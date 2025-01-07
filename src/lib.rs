mod encryption;
mod entry;
mod generators;
mod storage;
pub mod tui;

use crossterm::style::Stylize;
pub use encryption::Encrypter;
pub use entry::Entry;
use rpassword::prompt_password;
pub use storage::Storage;

use anyhow::{anyhow, Result};
use std::{fs, io::Write, path::PathBuf};

pub const VERSION: &str = "V1.0.0";
pub const ABOUT: &str = "Candado a Local Encrypted Password Manager & Secret Generator";
pub const PREFIX: &str = "Candado \u{f023}";

//------------------------------------------
// Manager
//------------------------------------------

pub fn init() -> Result<()> {
    println!("Initializing new Vault!");

    if Encrypter::load_keyfile_path().is_ok() {
        println!("WARNING there is already a existing Vault!");
        println!("continuing will permantly delete the exsting vault.");
        print!("are you sure? [y/n]: ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;

        match buffer.trim() {
            "y" | "Y" | "yes" | "YES" => {
                println!("Deleting vault...");
                let path = format!("{}/.candado", std::env::var("HOME")?);
                fs::remove_dir_all(path)?;
            }
            _ => {
                return Err(anyhow!("Aborted."));
            }
        }
    }

    let password = prompt_password(format!("{} Enter Master: ", PREFIX.green()))?;
    println!("Generating new keyfile...");
    Encrypter::new(&password)?;
    Ok(())
}

pub fn ls(encrypter: Encrypter) -> Result<Vec<Entry>> {
    Storage::init(&encrypter)?.list()
}

pub fn rm(encrypter: Encrypter, id: &str) -> Result<()> {
    Storage::init(&encrypter)?.remove(id)
}

pub fn read(encrypter: Encrypter, id: &str) -> Result<Entry, anyhow::Error> {
    Storage::init(&encrypter)?.read(id)
}

pub fn add(
    encrypter: Encrypter,
    service: String,
    email: String,
    password: Option<String>,
    username: Option<String>,
    url: Option<String>,
) -> Result<(), anyhow::Error> {
    let password = if let Some(pass) = password {
        pass.to_owned()
    } else {
        generators::gen_passphrase(4, &None)
    };

    let entry = Entry::new(
        generators::gen_key(12),
        service,
        email,
        password,
        username.unwrap_or("".to_string()),
        url.unwrap_or("".to_string()),
    );
    Storage::init(&encrypter)?.write(entry)
}

pub fn update(
    encrypter: Encrypter,
    id: &str,
    service: Option<String>,
    email: Option<String>,
    password: Option<String>,
    username: Option<String>,
    url: Option<String>,
) -> Result<()> {
    let storage = Storage::init(&encrypter)?;
    let mut entry = storage.read(id)?;
    entry.service = service.unwrap_or(entry.service);
    entry.email = email.unwrap_or(entry.email);
    entry.password = password.unwrap_or(entry.password);
    entry.username = username.unwrap_or(entry.username);
    entry.url = url.unwrap_or(entry.url);
    storage.update(entry)
}

pub fn find(encrypter: Encrypter, query: &str) -> Result<Vec<Entry>> {
    Storage::init(&encrypter)?.find(query)
}

pub fn import(encrypter: Encrypter, file: PathBuf) -> Result<()> {
    Storage::init(&encrypter)?.import(file)
}

pub fn export(encrypter: Encrypter, file: PathBuf) -> Result<()> {
    Storage::init(&encrypter)?.export(file)
}

pub fn login() -> Result<Encrypter, anyhow::Error> {
    let password = prompt_password(format!("{} Enter Master: ", PREFIX.green()))?;
    Encrypter::unlock(&password)
}

pub fn logout() -> Result<(), anyhow::Error> {
    todo!("Logout not implemented")
}

//------------------------------------------
// Generators
//------------------------------------------

pub fn password(length: u32) -> String {
    generators::gen_password(length)
}

pub fn token(length: u32) -> String {
    generators::gen_token(length)
}

pub fn key(length: u32) -> String {
    generators::gen_key(length)
}

pub fn passphrase(length: u32, wordlist: &Option<PathBuf>) -> String {
    generators::gen_passphrase(length, &wordlist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_password() {
        let result = password(4);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_gen_token() {
        let result = token(4);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_gen_key() {
        let result = key(4);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_gen_passhrase() {
        let result = passphrase(4, &None);
        assert_eq!(result.split(" ").collect::<Vec<&str>>().len(), 4);
    }
}
