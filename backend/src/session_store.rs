use async_trait::async_trait;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{self, SessionStore};

pub const TEMP_PREFIX: &str = ".tmp-";

#[derive(Clone, Debug)]
pub struct FileSessionStore {
    dir: PathBuf,
}

impl FileSessionStore {
    pub fn new<P: AsRef<Path>>(dir: P) -> std::io::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }

        Ok(Self { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn session_path(&self, id: &Id) -> PathBuf {
        self.dir.join(format!("{}.json", id))
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let dir = self.dir.clone();
        let mut rec = record.clone();
        let (rec, res) = tokio::task::spawn_blocking(move || {
            let mut attempts = 0;
            while attempts < 10 {
                let path = dir.join(format!("{}.json", rec.id));
                if !path.exists() {
                    break;
                }
                rec.id = Id::default();
                attempts += 1;
            }
            if dir.join(format!("{}.json", rec.id)).exists() {
                return (
                    rec,
                    Err(session_store::Error::Backend(
                        "Failed to generate unique session ID after 10 attempts".into(),
                    )),
                );
            }
            let res = save_record(&dir, &rec);
            (rec, res)
        })
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        res?;
        *record = rec;
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let dir = self.dir.clone();
        let rec = record.clone();
        tokio::task::spawn_blocking(move || save_record(&dir, &rec))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let path = self.session_path(session_id);
        tokio::task::spawn_blocking(move || load_record(&path))
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let path = self.session_path(session_id);
        tokio::task::spawn_blocking(move || {
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            Ok(())
        })
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?
    }
}

fn save_record(dir: &Path, record: &Record) -> session_store::Result<()> {
    let final_path = dir.join(format!("{}.json", record.id));
    let temp_path = dir.join(format!("{}{}-{}.json", TEMP_PREFIX, record.id, Id::default()));

    let mut record_to_save = record.clone();

    // Check if stay_signed_in is true
    let is_stay_signed_in = record_to_save
        .data
        .get("stay_signed_in")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !is_stay_signed_in {
        // Capped at 7 days of inactivity
        let cap = OffsetDateTime::now_utc() + time::Duration::days(7);
        if record_to_save.expiry_date > cap {
            record_to_save.expiry_date = cap;
        }
    }

    let json_bytes = serde_json::to_vec(&record_to_save)
        .map_err(|e| session_store::Error::Encode(e.to_string()))?;

    {
        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = file.set_permissions(std::fs::Permissions::from_mode(0o600));
        }

        use std::io::Write;
        file.write_all(&json_bytes)
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        file.sync_all()
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
    }

    std::fs::rename(&temp_path, &final_path)
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

    Ok(())
}

fn load_record(path: &Path) -> session_store::Result<Option<Record>> {
    if !path.exists() {
        return Ok(None);
    }

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(session_store::Error::Backend(e.to_string())),
    };

    let record: Record = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(_) => {
            // Corrupt file, clean it up
            let _ = std::fs::remove_file(path);
            return Ok(None);
        }
    };

    if record.expiry_date <= OffsetDateTime::now_utc() {
        let _ = std::fs::remove_file(path);
        return Ok(None);
    }

    Ok(Some(record))
}

pub fn clean_expired_sessions(dir: &Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let now = OffsetDateTime::now_utc();
    let temp_max_age = std::time::Duration::from_secs(10 * 60);

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if file_name.starts_with(TEMP_PREFIX) {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > temp_max_age {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        } else if file_name.ends_with(".json") {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(record) = serde_json::from_slice::<Record>(&bytes) {
                    if record.expiry_date <= now {
                        let _ = std::fs::remove_file(&path);
                    }
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

pub fn spawn_cleanup_task(
    store: FileSessionStore,
    interval: std::time::Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        loop {
            interval_timer.tick().await;
            let dir = store.dir.clone();
            let _ = tokio::task::spawn_blocking(move || {
                clean_expired_sessions(&dir);
            })
            .await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use tower_sessions::session::Id;

    #[tokio::test]
    async fn test_create_load_delete() {
        let dir = tempdir().unwrap();
        let store = FileSessionStore::new(dir.path()).unwrap();

        let mut data = HashMap::new();
        data.insert("user".to_string(), serde_json::json!({"username": "test"}));

        let mut record = Record {
            id: Id::default(),
            data: data.clone(),
            expiry_date: OffsetDateTime::now_utc() + time::Duration::days(1),
        };

        store.create(&mut record).await.unwrap();

        let loaded = store.load(&record.id).await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, record.id);
        assert_eq!(loaded.data.get("user"), data.get("user"));

        // Delete
        store.delete(&record.id).await.unwrap();
        let loaded_after = store.load(&record.id).await.unwrap();
        assert!(loaded_after.is_none());
    }

    #[tokio::test]
    async fn test_expired_session_returns_none_and_deletes() {
        let dir = tempdir().unwrap();
        let store = FileSessionStore::new(dir.path()).unwrap();

        let record = Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: OffsetDateTime::now_utc() - time::Duration::seconds(10),
        };

        // Write directly to disk
        let session_file = store.session_path(&record.id);
        std::fs::write(&session_file, serde_json::to_vec(&record).unwrap()).unwrap();
        assert!(session_file.exists());

        // Load should detect expiry, delete file, and return None
        let loaded = store.load(&record.id).await.unwrap();
        assert!(loaded.is_none());
        assert!(!session_file.exists());
    }

    #[tokio::test]
    async fn test_stay_signed_in_versus_not() {
        let dir = tempdir().unwrap();
        let store = FileSessionStore::new(dir.path()).unwrap();

        // Case 1: stay_signed_in is false, initial expiry is 14 days
        let mut data_not_stay = HashMap::new();
        data_not_stay.insert("stay_signed_in".to_string(), serde_json::json!(false));
        let record_not_stay = Record {
            id: Id::default(),
            data: data_not_stay,
            expiry_date: OffsetDateTime::now_utc() + time::Duration::days(14),
        };

        store.save(&record_not_stay).await.unwrap();
        let loaded_not_stay = store.load(&record_not_stay.id).await.unwrap().unwrap();
        let max_expected = OffsetDateTime::now_utc() + time::Duration::days(7);
        assert!(loaded_not_stay.expiry_date <= max_expected + time::Duration::seconds(2));

        // Case 2: stay_signed_in is true, initial expiry is 90 days
        let mut data_stay = HashMap::new();
        data_stay.insert("stay_signed_in".to_string(), serde_json::json!(true));
        let record_stay = Record {
            id: Id::default(),
            data: data_stay,
            expiry_date: OffsetDateTime::now_utc() + time::Duration::days(90),
        };

        store.save(&record_stay).await.unwrap();
        let loaded_stay = store.load(&record_stay.id).await.unwrap().unwrap();
        let min_expected = OffsetDateTime::now_utc() + time::Duration::days(89);
        assert!(loaded_stay.expiry_date >= min_expected);
    }

    #[tokio::test]
    async fn test_corrupt_file_handling() {
        let dir = tempdir().unwrap();
        let store = FileSessionStore::new(dir.path()).unwrap();

        let id = Id::default();
        let file_path = store.session_path(&id);
        std::fs::write(&file_path, b"not valid json").unwrap();
        assert!(file_path.exists());

        let loaded = store.load(&id).await.unwrap();
        assert!(loaded.is_none());
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_clean_expired_sessions() {
        let dir = tempdir().unwrap();
        let store = FileSessionStore::new(dir.path()).unwrap();

        // 1. Valid session
        let valid_rec = Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: OffsetDateTime::now_utc() + time::Duration::days(1),
        };
        store.save(&valid_rec).await.unwrap();

        // 2. Expired session
        let expired_rec = Record {
            id: Id::default(),
            data: HashMap::new(),
            expiry_date: OffsetDateTime::now_utc() - time::Duration::days(1),
        };
        let expired_file = store.session_path(&expired_rec.id);
        std::fs::write(&expired_file, serde_json::to_vec(&expired_rec).unwrap()).unwrap();

        // 3. Clean
        clean_expired_sessions(dir.path());

        assert!(store.session_path(&valid_rec.id).exists());
        assert!(!expired_file.exists());
    }
}
