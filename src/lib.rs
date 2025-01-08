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

use anyhow::Result;
use std::path::PathBuf;

pub const VERSION: &str = "V1.0.2";
pub const ABOUT: &str = "Candado a Local Encrypted Password Manager & Secret Generator";
pub const PREFIX: &str = "Candado \u{f023}";
pub const PROGRAM_FOLDER: &str = ".candado";

//------------------------------------------
// Manager
//------------------------------------------

pub fn init() -> Result<()> {
    let password = prompt_password(format!("{} Enter new master: ", PREFIX.green()))?;
    Encrypter::init(&password)
}

pub fn unlock() -> Result<Encrypter> {
    let password = prompt_password(format!("{} Enter Master: ", PREFIX.green()))?;
    Encrypter::unlock(&password)
}

pub fn ls(encrypter: Encrypter) -> Result<Vec<Entry>> {
    Storage::init(&encrypter)?.list()
}

pub fn rm(encrypter: Encrypter, id: &str) -> Result<()> {
    Storage::init(&encrypter)?.remove(id)
}

pub fn read(encrypter: Encrypter, id: &str) -> Result<Entry> {
    Storage::init(&encrypter)?.read(id)
}

pub fn add(
    encrypter: Encrypter,
    service: String,
    email: String,
    password: Option<String>,
    username: Option<String>,
    url: Option<String>,
) -> Result<()> {
    Storage::init(&encrypter)?.write(Entry::new(service, email, password, username, url))
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
    entry.overite(service, email, password, username, url);
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

    // TODO: Add tests!

    #[test]
    fn test_init() {}

    #[test]
    fn test_unlock() {}

    #[test]
    fn test_ls() {}

    #[test]
    fn test_rm() {}

    #[test]
    fn test_read() {}

    #[test]
    fn test_add() {}

    #[test]
    fn test_update() {}

    #[test]
    fn test_find() {}

    #[test]
    fn import() {}

    #[test]
    fn export() {}

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
