use crate::Route;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;
use yew_router::prelude::*;

use common::RenameRequest;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Shortcuts {
    pub edit: String,
    pub save: String,
    pub cancel: String,
    pub search: String,
    pub toggle_drawer: String,
    pub copy_link: String,
    pub undo: String,
    pub redo: String,
    pub scroll_down: String,
    pub scroll_up: String,
    pub insert_datetime: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            edit: "Ctrl+E".to_string(),
            save: "Ctrl+S".to_string(),
            cancel: "Escape".to_string(),
            search: "<Ctrl+p><Ctrl+f>".to_string(),
            toggle_drawer: "Ctrl+B".to_string(),
            copy_link: "Ctrl+Shift+C".to_string(),
            undo: "u".to_string(),
            redo: "r, <Ctrl+r>".to_string(),
            scroll_down: "<Ctrl+d>".to_string(),
            scroll_up: "<Ctrl+u>".to_string(),
            insert_datetime: "<Ctrl+t><Ctrl+s>".to_string(),
        }
    }
}

impl Shortcuts {
    /// Load shortcuts from LocalStorage, migrating legacy default search binding if present.
    pub fn load() -> Self {
        let mut s = LocalStorage::get::<Shortcuts>("shortcuts").unwrap_or_default();
        if s.search.trim() == "Ctrl+K, Ctrl+P" {
            s.search = "<Ctrl+p><Ctrl+f>".to_string();
            let _ = LocalStorage::set("shortcuts", s.clone());
        }
        s
    }
}

/// Represents a single normalized key press step in a potentially multi-step chord.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyStep {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyStep {
    pub fn from_keyboard_event(e: &KeyboardEvent) -> Option<Self> {
        let raw_key = e.key();
        // Ignore pure modifier presses
        if matches!(
            raw_key.as_str(),
            "Control" | "Shift" | "Alt" | "Meta" | "CapsLock" | "OS"
        ) {
            return None;
        }
        let is_ctrl = e.ctrl_key() || e.meta_key();
        let is_shift = e.shift_key();
        let is_alt = e.alt_key();
        let key = normalize_key_name(&raw_key);
        Some(Self {
            key,
            ctrl: is_ctrl,
            shift: is_shift,
            alt: is_alt,
        })
    }
}

fn normalize_key_name(raw: &str) -> String {
    let trimmed = raw.trim();
    match trimmed.to_lowercase().as_str() {
        "esc" | "escape" => "escape".to_string(),
        "return" | "enter" => "enter".to_string(),
        "space" | " " => " ".to_string(),
        "bs" | "backspace" => "backspace".to_string(),
        "del" | "delete" => "delete".to_string(),
        "tab" => "tab".to_string(),
        "up" | "arrowup" => "arrowup".to_string(),
        "down" | "arrowdown" => "arrowdown".to_string(),
        "left" | "arrowleft" => "arrowleft".to_string(),
        "right" | "arrowright" => "arrowright".to_string(),
        other => other.to_string(),
    }
}

/// Splits a single alternative (e.g. `"<Ctrl+p><Ctrl+f>"` or `"Ctrl+P Ctrl+F"` or `"Ctrl+Shift+C"`)
/// into individual stroke tokens (e.g. `["<Ctrl+p>", "<Ctrl+f>"]` or `["Ctrl+P", "Ctrl+F"]`).
fn split_chord_strokes(alt: &str) -> Result<Vec<String>, String> {
    let mut strokes = Vec::new();
    let mut current = String::new();
    let mut in_angle = false;

    for ch in alt.chars() {
        match ch {
            '<' => {
                if in_angle {
                    return Err(format!("Nested '<' in shortcut '{}'", alt));
                }
                if !current.trim().is_empty() {
                    strokes.push(current.trim().to_string());
                    current.clear();
                }
                in_angle = true;
                current.push('<');
            }
            '>' => {
                if !in_angle {
                    return Err(format!("Unmatched '>' in shortcut '{}'", alt));
                }
                current.push('>');
                in_angle = false;
                strokes.push(current.trim().to_string());
                current.clear();
            }
            ' ' | '\t' if !in_angle => {
                if !current.trim().is_empty() {
                    strokes.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if in_angle {
        return Err(format!("Unclosed '<' in shortcut '{}'", alt));
    }

    if !current.trim().is_empty() {
        strokes.push(current.trim().to_string());
    }

    Ok(strokes)
}

/// Parses a single stroke token like `"<Ctrl+p>"`, `"<C-p>"`, `"Ctrl+Shift+C"`, `"u"`, or `"U"`
/// into a `KeyStep`.
pub fn parse_single_step(stroke: &str) -> Result<KeyStep, String> {
    let trimmed = stroke.trim();
    if trimmed.is_empty() {
        return Err("Empty keystroke".to_string());
    }

    let inner = if trimmed.starts_with('<') && trimmed.ends_with('>') {
        let inside = &trimmed[1..trimmed.len() - 1].trim();
        if inside.is_empty() {
            return Err("Empty '<>' in shortcut".to_string());
        }
        *inside
    } else {
        trimmed
    };

    // Single character like "u" or "U" or "/"
    if inner.chars().count() == 1 {
        let ch = inner.chars().next().unwrap();
        let is_upper = ch.is_ascii_uppercase();
        return Ok(KeyStep {
            key: ch.to_ascii_lowercase().to_string(),
            ctrl: false,
            shift: is_upper,
            alt: false,
        });
    }

    // Split by '+' or '-' (supporting both Ctrl+P and C-p)
    let mut parts: Vec<&str> = Vec::new();
    let mut start = 0;
    for (idx, ch) in inner.char_indices() {
        if (ch == '+' || ch == '-') && idx > start {
            parts.push(&inner[start..idx]);
            start = idx + 1;
        }
    }
    if start < inner.len() {
        parts.push(&inner[start..]);
    } else if inner.ends_with('+') || inner.ends_with('-') {
        return Err(format!("Missing key after modifier in '{}'", stroke));
    }

    if parts.is_empty() {
        return Err(format!("Invalid keystroke '{}'", stroke));
    }

    let mut req_ctrl = false;
    let mut req_shift = false;
    let mut req_alt = false;

    // All parts except the last must be modifiers
    for &mod_part in &parts[..parts.len() - 1] {
        match mod_part.trim().to_lowercase().as_str() {
            "ctrl" | "c" | "cmd" | "meta" | "m" | "control" => req_ctrl = true,
            "shift" | "s" => req_shift = true,
            "alt" | "a" | "opt" | "option" => req_alt = true,
            unknown => {
                return Err(format!(
                    "Unknown modifier '{}' in shortcut '{}'",
                    unknown, stroke
                ))
            }
        }
    }

    let last_raw = parts[parts.len() - 1].trim();
    if last_raw.is_empty() {
        return Err(format!("Missing key in '{}'", stroke));
    }

    let last_lower = last_raw.to_lowercase();
    if matches!(
        last_lower.as_str(),
        "ctrl" | "c" | "cmd" | "meta" | "shift" | "alt" | "control"
    ) && parts.len() == 1
        && last_raw.len() > 1
    {
        return Err(format!(
            "Shortcut '{}' cannot consist only of a modifier",
            stroke
        ));
    }

    // If no modifiers were given and it's a single uppercase ASCII letter, preserve shift
    if parts.len() == 1 && last_raw.chars().count() == 1 {
        let ch = last_raw.chars().next().unwrap();
        if ch.is_ascii_uppercase() {
            req_shift = true;
        }
    }

    let key = normalize_key_name(&last_lower);
    Ok(KeyStep {
        key,
        ctrl: req_ctrl,
        shift: req_shift,
        alt: req_alt,
    })
}

/// Parses a comma-separated shortcut definition (e.g. `"r, <Ctrl+r>"` or `"<Ctrl+p><Ctrl+f>"`)
/// into a list of chord sequences (`Vec<Vec<KeyStep>>`).
pub fn parse_shortcut_sequences(shortcut_def: &str) -> Result<Vec<Vec<KeyStep>>, String> {
    if shortcut_def.trim().is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }

    let mut sequences = Vec::new();
    for part in shortcut_def.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("Empty alternative in '{}'", shortcut_def));
        }
        let stroke_tokens = split_chord_strokes(part)?;
        if stroke_tokens.is_empty() {
            return Err(format!("No keystrokes found in '{}'", part));
        }
        let mut chord = Vec::with_capacity(stroke_tokens.len());
        for token in stroke_tokens {
            chord.push(parse_single_step(&token)?);
        }
        sequences.push(chord);
    }

    if sequences.is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }

    Ok(sequences)
}

/// Validates all fields in `Shortcuts` and checks for duplicate bindings in the same scope.
pub fn validate_all_shortcuts(s: &Shortcuts) -> Result<(), String> {
    let global_fields = [
        ("Open Document / Search", &s.search),
        ("Edit Page", &s.edit),
        ("Save Page", &s.save),
        ("Cancel / Exit Edit", &s.cancel),
        ("Toggle Side Panel", &s.toggle_drawer),
        ("Copy Link", &s.copy_link),
    ];

    let editor_fields = [
        ("Undo (Vim Normal)", &s.undo),
        ("Redo (Vim Normal)", &s.redo),
        ("Scroll Half-Page Down", &s.scroll_down),
        ("Scroll Half-Page Up", &s.scroll_up),
        ("Insert Date/Time", &s.insert_datetime),
    ];

    let mut global_seen: Vec<(&str, Vec<KeyStep>)> = Vec::new();
    for (label, val) in &global_fields {
        let seqs =
            parse_shortcut_sequences(val).map_err(|e| format!("Invalid '{}': {}", label, e))?;
        for seq in seqs {
            for (existing_label, existing_seq) in &global_seen {
                if existing_label != label && *existing_seq == seq {
                    return Err(format!(
                        "Conflict: '{}' and '{}' share the same keybinding",
                        label, existing_label
                    ));
                }
            }
            global_seen.push((label, seq));
        }
    }

    let mut editor_seen: Vec<(&str, Vec<KeyStep>)> = Vec::new();
    // Include search in editor normal-mode conflict check since search is also active in Vim Normal mode
    if let Ok(search_seqs) = parse_shortcut_sequences(&s.search) {
        for seq in search_seqs {
            editor_seen.push(("Open Document / Search", seq));
        }
    }

    for (label, val) in &editor_fields {
        let seqs =
            parse_shortcut_sequences(val).map_err(|e| format!("Invalid '{}': {}", label, e))?;
        for seq in seqs {
            for (existing_label, existing_seq) in &editor_seen {
                if existing_label != label && *existing_seq == seq {
                    return Err(format!(
                        "Conflict: '{}' and '{}' share the same keybinding",
                        label, existing_label
                    ));
                }
            }
            editor_seen.push((label, seq));
        }
    }

    Ok(())
}

/// Legacy single-event helper preserved for backward compatibility in simple 1-step checks.
#[allow(dead_code)]
pub fn matches_shortcut(e: &KeyboardEvent, shortcut_def: &str) -> bool {
    if let Some(step) = KeyStep::from_keyboard_event(e) {
        if let Ok(seqs) = parse_shortcut_sequences(shortcut_def) {
            return seqs.iter().any(|seq| seq.len() == 1 && seq[0] == step);
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobalAction {
    Cancel,
    Save,
    Search,
    ToggleDrawer,
    Edit,
    CopyLink,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChordMatchResult {
    None,
    PrefixPending,
    Matched(GlobalAction),
}

/// Pure-Rust stateful multi-key chord matcher with timeout expiration and prefix fallback.
pub struct ChordState {
    pub buffer: Vec<KeyStep>,
    pub last_timestamp_ms: f64,
    pub timeout_ms: f64,
}

impl Default for ChordState {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            last_timestamp_ms: 0.0,
            timeout_ms: 1500.0,
        }
    }
}

impl ChordState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, step: KeyStep, now_ms: f64, shortcuts: &Shortcuts) -> ChordMatchResult {
        if !self.buffer.is_empty() && (now_ms - self.last_timestamp_ms > self.timeout_ms) {
            self.buffer.clear();
        }
        self.last_timestamp_ms = now_ms;
        self.buffer.push(step.clone());

        let result = Self::evaluate_buffer(&self.buffer, shortcuts);
        match result {
            ChordMatchResult::Matched(action) => {
                self.buffer.clear();
                ChordMatchResult::Matched(action)
            }
            ChordMatchResult::PrefixPending => ChordMatchResult::PrefixPending,
            ChordMatchResult::None => {
                // If buffer had > 1 elements and failed, retry with just the new keystep alone
                if self.buffer.len() > 1 {
                    self.buffer.clear();
                    self.buffer.push(step);
                    match Self::evaluate_buffer(&self.buffer, shortcuts) {
                        ChordMatchResult::Matched(action) => {
                            self.buffer.clear();
                            ChordMatchResult::Matched(action)
                        }
                        ChordMatchResult::PrefixPending => ChordMatchResult::PrefixPending,
                        ChordMatchResult::None => {
                            self.buffer.clear();
                            ChordMatchResult::None
                        }
                    }
                } else {
                    self.buffer.clear();
                    ChordMatchResult::None
                }
            }
        }
    }

    fn evaluate_buffer(buffer: &[KeyStep], shortcuts: &Shortcuts) -> ChordMatchResult {
        let bindings = [
            (GlobalAction::Cancel, &shortcuts.cancel),
            (GlobalAction::Save, &shortcuts.save),
            (GlobalAction::Search, &shortcuts.search),
            (GlobalAction::ToggleDrawer, &shortcuts.toggle_drawer),
            (GlobalAction::Edit, &shortcuts.edit),
            (GlobalAction::CopyLink, &shortcuts.copy_link),
        ];

        let mut has_prefix = false;

        for (action, def) in &bindings {
            if let Ok(seqs) = parse_shortcut_sequences(def) {
                for seq in seqs {
                    if seq == buffer {
                        return ChordMatchResult::Matched(*action);
                    }
                    if seq.len() > buffer.len() && seq.starts_with(buffer) {
                        has_prefix = true;
                    }
                }
            }
        }

        if has_prefix {
            ChordMatchResult::PrefixPending
        } else {
            ChordMatchResult::None
        }
    }
}

fn normalize_wiki_file_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with('/') {
        return format!("{}index.md", trimmed);
    }
    if std::path::Path::new(trimmed).extension().is_none() {
        format!("{}.md", trimmed)
    } else {
        trimmed.to_string()
    }
}

#[hook]
pub fn use_create_file(current_volume: String) -> Callback<()> {
    let navigator_opt = use_navigator();

    Callback::from(move |_| {
        if let Some(navigator) = &navigator_opt {
            if let Some(path) = gloo_dialogs::prompt("Enter file path (e.g. folder/note.md):", None)
            {
                let normalized = normalize_wiki_file_path(&path);
                if !normalized.is_empty() {
                    navigator.push(&Route::Wiki {
                        volume: current_volume.clone(),
                        path: normalized,
                    });
                }
            }
        } else {
            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(
                "use_create_file must be used inside a BrowserRouter",
            ));
        }
    })
}

#[derive(Clone, PartialEq)]
pub struct KeyHandlerProps {
    pub shortcuts: Shortcuts,
    pub on_search: Callback<()>,
    pub on_edit: Callback<()>,
    pub on_save: Callback<()>,
    pub on_cancel: Callback<()>,
    pub on_toggle_drawer: Callback<()>,
    pub on_copy_link: Callback<()>,
}

#[hook]
pub fn use_key_handler(props: KeyHandlerProps) {
    let initial_props = props.clone();
    let props_ref = use_mut_ref(move || initial_props);
    *props_ref.borrow_mut() = props;

    let chord_state_ref = use_mut_ref(ChordState::new);

    {
        let props_ref = props_ref.clone();
        let chord_state_ref = chord_state_ref.clone();

        use_effect_with((), move |_| {
            let window = gloo_utils::window();
            let props_ref = props_ref.clone();
            let chord_state_ref = chord_state_ref.clone();

            let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                let props = props_ref.borrow();

                // If focus is on an input, textarea, or contenteditable element,
                // let the element handle ALL keystrokes — including Escape (vim Normal mode),
                // Ctrl+S (CodeMirror save keymap), etc.
                if let Some(target) = e.target() {
                    if target.dyn_ref::<web_sys::HtmlInputElement>().is_some() {
                        chord_state_ref.borrow_mut().buffer.clear();
                        return;
                    }
                    if target.dyn_ref::<web_sys::HtmlTextAreaElement>().is_some() {
                        chord_state_ref.borrow_mut().buffer.clear();
                        return;
                    }
                    if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                        if el.is_content_editable() {
                            chord_state_ref.borrow_mut().buffer.clear();
                            return;
                        }
                    }
                }

                let Some(step) = KeyStep::from_keyboard_event(&e) else {
                    return;
                };

                let now_ms = js_sys::Date::now();
                let match_res = chord_state_ref
                    .borrow_mut()
                    .feed(step, now_ms, &props.shortcuts);

                match match_res {
                    ChordMatchResult::PrefixPending => {
                        // Prevent browser default (e.g., Ctrl+P opening Print dialog) on first stroke of a chord
                        e.prevent_default();
                    }
                    ChordMatchResult::Matched(action) => {
                        e.prevent_default();
                        match action {
                            GlobalAction::Cancel => props.on_cancel.emit(()),
                            GlobalAction::Save => props.on_save.emit(()),
                            GlobalAction::Search => props.on_search.emit(()),
                            GlobalAction::ToggleDrawer => props.on_toggle_drawer.emit(()),
                            GlobalAction::Edit => props.on_edit.emit(()),
                            GlobalAction::CopyLink => props.on_copy_link.emit(()),
                        }
                    }
                    ChordMatchResult::None => {}
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);

            let _ = window
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref());

            move || {
                let _ = window.remove_event_listener_with_callback(
                    "keydown",
                    on_keydown.as_ref().unchecked_ref(),
                );
            }
        });
    }
}

#[hook]
pub fn use_rename_file(current_volume: String, current_path: String) -> Callback<()> {
    let navigator = use_navigator();

    Callback::from(move |_| {
        if let Some(raw_new_path) =
            gloo_dialogs::prompt("Enter new file path:", Some(&current_path))
        {
            let new_path = normalize_wiki_file_path(&raw_new_path);
            if new_path != current_path && !new_path.is_empty() {
                let path = current_path.clone();
                let volume = current_volume.clone();
                let navigator = navigator.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/rename/{}/{}", volume, path);
                    let payload = RenameRequest {
                        new_path: new_path.clone(),
                    };
                    let resp = Request::post(&url).json(&payload).unwrap().send().await;
                    match resp {
                        Ok(r) if r.status() == 401 => {
                            let current_path = gloo_utils::window()
                                .location()
                                .pathname()
                                .unwrap_or_default();
                            if current_path != "/login" {
                                let _ = gloo_utils::window().location().set_href("/login");
                            }
                        }
                        Ok(r) if r.ok() => {
                            if let Some(nav) = navigator {
                                nav.push(&Route::Wiki {
                                    volume,
                                    path: new_path,
                                });
                            }
                        }
                        Ok(r) => {
                            let text = r.text().await.unwrap_or_default();
                            gloo_dialogs::alert(&format!("Failed to rename: {}", text));
                        }
                        Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                    }
                });
            }
        }
    })
}

#[hook]
pub fn use_delete_file(current_volume: String, current_path: String) -> Callback<()> {
    Callback::from(move |_| {
        if gloo_dialogs::confirm(&format!(
            "Are you sure you want to delete {}?",
            current_path
        )) {
            let path = current_path.clone();
            let volume = current_volume.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/wiki/{}/{}", volume, path);
                let resp = Request::delete(&url).send().await;
                match resp {
                    Ok(r) if r.status() == 401 => {
                        let current_path = gloo_utils::window()
                            .location()
                            .pathname()
                            .unwrap_or_default();
                        if current_path != "/login" {
                            let _ = gloo_utils::window().location().set_href("/login");
                        }
                    }
                    Ok(r) if r.ok() => {
                        let _ = gloo_utils::window()
                            .location()
                            .set_href(&format!("/wiki/{}", volume));
                    }
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        gloo_dialogs::alert(&format!("Failed to delete: {}", text));
                    }
                    Err(e) => gloo_dialogs::alert(&format!("Network error: {}", e)),
                }
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vim_and_standard_chords() {
        let seqs = parse_shortcut_sequences("<Ctrl+p><Ctrl+f>").unwrap();
        assert_eq!(seqs.len(), 1);
        assert_eq!(
            seqs[0],
            vec![
                KeyStep {
                    key: "p".to_string(),
                    ctrl: true,
                    shift: false,
                    alt: false,
                },
                KeyStep {
                    key: "f".to_string(),
                    ctrl: true,
                    shift: false,
                    alt: false,
                },
            ]
        );

        let seqs2 = parse_shortcut_sequences("Ctrl+P Ctrl+F, <C-k>").unwrap();
        assert_eq!(seqs2.len(), 2);
        assert_eq!(seqs2[0], seqs[0]);
        assert_eq!(
            seqs2[1],
            vec![KeyStep {
                key: "k".to_string(),
                ctrl: true,
                shift: false,
                alt: false,
            }]
        );
    }

    #[test]
    fn test_parse_undo_redo_single_keys() {
        let undo_seqs = parse_shortcut_sequences("u").unwrap();
        assert_eq!(
            undo_seqs,
            vec![vec![KeyStep {
                key: "u".to_string(),
                ctrl: false,
                shift: false,
                alt: false,
            }]]
        );

        let redo_seqs = parse_shortcut_sequences("r, <Ctrl+r>").unwrap();
        assert_eq!(redo_seqs.len(), 2);
        assert_eq!(
            redo_seqs[0],
            vec![KeyStep {
                key: "r".to_string(),
                ctrl: false,
                shift: false,
                alt: false,
            }]
        );
        assert_eq!(
            redo_seqs[1],
            vec![KeyStep {
                key: "r".to_string(),
                ctrl: true,
                shift: false,
                alt: false,
            }]
        );
    }

    #[test]
    fn test_chord_state_machine_and_prefix_fallback() {
        let shortcuts = Shortcuts::default();
        let mut state = ChordState::new();

        let ctrl_p = KeyStep {
            key: "p".to_string(),
            ctrl: true,
            shift: false,
            alt: false,
        };
        let ctrl_f = KeyStep {
            key: "f".to_string(),
            ctrl: true,
            shift: false,
            alt: false,
        };
        let ctrl_e = KeyStep {
            key: "e".to_string(),
            ctrl: true,
            shift: false,
            alt: false,
        };

        // 1. Standard two-step chord <Ctrl+p><Ctrl+f>
        assert_eq!(
            state.feed(ctrl_p.clone(), 100.0, &shortcuts),
            ChordMatchResult::PrefixPending
        );
        assert_eq!(
            state.feed(ctrl_f.clone(), 250.0, &shortcuts),
            ChordMatchResult::Matched(GlobalAction::Search)
        );

        // 2. Pressing <Ctrl+p> twice then <Ctrl+f> should still match Search!
        assert_eq!(
            state.feed(ctrl_p.clone(), 500.0, &shortcuts),
            ChordMatchResult::PrefixPending
        );
        assert_eq!(
            state.feed(ctrl_p.clone(), 600.0, &shortcuts),
            ChordMatchResult::PrefixPending
        );
        assert_eq!(
            state.feed(ctrl_f.clone(), 700.0, &shortcuts),
            ChordMatchResult::Matched(GlobalAction::Search)
        );

        // 3. Pressing <Ctrl+p> then Ctrl+E should fallback to matching Edit!
        assert_eq!(
            state.feed(ctrl_p.clone(), 1000.0, &shortcuts),
            ChordMatchResult::PrefixPending
        );
        assert_eq!(
            state.feed(ctrl_e, 1100.0, &shortcuts),
            ChordMatchResult::Matched(GlobalAction::Edit)
        );

        // 4. Timeout expiration (> 1500ms)
        assert_eq!(
            state.feed(ctrl_p, 2000.0, &shortcuts),
            ChordMatchResult::PrefixPending
        );
        assert_eq!(
            state.feed(ctrl_f, 4000.0, &shortcuts),
            ChordMatchResult::None
        );
    }

    #[test]
    fn test_validation_and_serde_defaults() {
        let def = Shortcuts::default();
        assert!(validate_all_shortcuts(&def).is_ok());

        // Legacy JSON missing new fields should deserialize with defaults, not empty strings
        let legacy_json = r#"{"edit":"Ctrl+E","save":"Ctrl+S","cancel":"Escape","search":"Ctrl+K, Ctrl+P","toggle_drawer":"Ctrl+B","copy_link":"Ctrl+Shift+C"}"#;
        let parsed: Shortcuts = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(parsed.undo, "u");
        assert_eq!(parsed.redo, "r, <Ctrl+r>");
        assert_eq!(parsed.insert_datetime, "<Ctrl+t><Ctrl+s>");

        // Invalid syntax
        let mut bad = def.clone();
        bad.search = "<Ctrl+p".to_string();
        assert!(validate_all_shortcuts(&bad).is_err());

        // Duplicate binding
        let mut dup = def.clone();
        dup.edit = "Ctrl+B".to_string();
        assert!(validate_all_shortcuts(&dup).is_err());
    }

    #[test]
    fn test_normalize_wiki_file_path() {
        assert_eq!(normalize_wiki_file_path("note"), "note.md");
        assert_eq!(normalize_wiki_file_path("/folder/note"), "folder/note.md");
        assert_eq!(normalize_wiki_file_path("folder/note.md"), "folder/note.md");
        assert_eq!(normalize_wiki_file_path("folder/"), "folder/index.md");
        assert_eq!(normalize_wiki_file_path("   "), "");
    }
}
