use common::auth::{encrypt_users, hash_password, User};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} add-user <users_file> [secret_key]", args[0]);
        eprintln!("       (If secret_key is not provided, it is read from AUTH_SECRET env var)");
        eprintln!("       (Credentials are read from stdin/prompt)");
        std::process::exit(1);
    }

    let command = &args[1];
    if command != "add-user" {
        eprintln!("Unknown command: {}", command);
        std::process::exit(1);
    }

    let users_file_path = &args[2];

    let secret_key = if args.len() > 3 {
        args[3].clone()
    } else {
        env::var("AUTH_SECRET").expect("AUTH_SECRET env var not set and key not provided")
    };

    // Prompt for username
    print!("Enter username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();

    // Prompt for password
    print!("Enter password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    let (hash, salt) = hash_password(password);
    let new_user = User {
        username: username.clone(),
        password_hash: hash,
        salt,
    };

    let mut users = Vec::new();
    if Path::new(users_file_path).exists() {
        let content = fs::read_to_string(users_file_path).unwrap_or_default();
        if !content.is_empty() {
             match common::auth::decrypt_users(&content, &secret_key) {
                Ok(existing_users) => {
                    users = existing_users;
                    // Check if user exists
                    if users.iter().any(|u| u.username == username) {
                        eprintln!("User {} already exists. Updating password.", username);
                        users.retain(|u| u.username != username);
                    }
                },
                Err(_) => {
                    eprintln!("Warning: Could not decrypt existing users file. Overwriting.");
                }
             }
        }
    }

    users.push(new_user);

    let encrypted_content = encrypt_users(&users, &secret_key).expect("Failed to encrypt users");
    fs::write(users_file_path, encrypted_content).expect("Failed to write users file");

    println!("User {} added/updated successfully in {}", username, users_file_path);
}
