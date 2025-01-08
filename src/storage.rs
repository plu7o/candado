use std::{fs::{self, File, Permissions}, io::Write, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

use crate::{entry::{Decrypt, Encrypt, EncryptedEntry, Entry, ImportedEntry}, Encrypter, PROGRAM_FOLDER};
use anyhow::{anyhow, Result};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rusqlite::{params, Connection};

pub enum SupportedFile {
    JSON(String),
    SQL(String),
}

pub struct Storage<'unlocked> {
    conn: Connection,
    encrypter: &'unlocked Encrypter,
}

impl<'unlocked> Storage<'unlocked> {
    /// Initialize storage
    ///
    /// # Params
    /// * Needs a Encrypter instace to encrypt & decrypt entries
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to create and set permission of db
    /// * fails to connect to db
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// 
    pub fn init(encrypter: &'unlocked Encrypter) -> Result<Self> {
        // Linux
        #[cfg(target_os = "linux")]
        let db_path = format!("{}/{}/candado.db", std::env::var("HOME")?, PROGRAM_FOLDER);

        // NOTE: Support for Mac os and Windows will be added in the future
        // MacOs
        // #[cfg(target_os = "macos")]
        // let db_path = format!("{}/.candado/candado.db", std::env::var("HOME")?);
        // windows
        // #[cfg(target_os = "windows")]
        // let db_path = format!("{}/.candado/.candado.db", std::env::var("USERHOME")?);

        let db_path = Path::new(&db_path);
        if !db_path.exists() {
            File::create(db_path)?.set_permissions(Permissions::from_mode(0o600))?;
        }

        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS candado (
            id INTEGER PRIMARY KEY,
            entry_id TEXT NOT NULL,
            service BLOB NOT NULL,
            email BLOB NOT NULL,
            password BLOB NOT NULL,
            username BLOB NOT NULL,
            url BLOB NOT NULL
        )",
            [],
        )?;
        let storage = Self { conn, encrypter };
        Ok(storage)
    }

    /// write a single entry to the vault
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to write to db
    /// * can't encrypt entry
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let entry = Entry::default;
    /// let result = storage.write(entry); 
    /// 
    pub fn write<T: Encrypt>(&self, entry: T) -> Result<()> {
        let entry = entry.encrypt(&self.encrypter)?;
        self.conn.execute(
            "INSERT INTO candado (entry_id, service, email, password, username, url) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id, 
                entry.service, 
                entry.email, 
                entry.password, 
                entry.username, 
                entry.url,
            ],
        )?;
        Ok(())
    }

    /// removes a single entry by id from the vault
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to write to db
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let result = storage.delete("jkdfnF54ms");
    /// 
    pub fn remove(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM candado WHERE entry_id=?1",
            params![id],
        )?;
        Ok(())
    }

    /// read a single entry by id from the vault
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to read from db
    /// * can't decrypt the entries
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let entry: Entry = storage.read("jkdfnF54ms")?;
    /// 
    pub fn read(&self, id: &str) -> Result<Entry> {
        let mut stmt = self.conn.prepare("SELECT * FROM candado WHERE entry_id=?1")?;
        let entry = stmt.query_row(params![id], |row| {
            Ok(EncryptedEntry{
                id: row.get(1)?,
                service: row.get(2)?,
                email: row.get(3)?,
                password: row.get(4)?,
                username: row.get(5)?,
                url: row.get(6)?,
            })
        })?;
        let entry = entry.decrypt(&self.encrypter)?;
        Ok(entry)
    }

    /// updates an entry in the vault
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to write to db
    /// * can't encrypt the entry
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let entries: Vec<Entry> = storage.find("some service")?;
    /// 
    pub fn update<T: Encrypt>(&self, entry: T) -> Result<()> {
        let entry = entry.encrypt(&self.encrypter)?;
        self.conn.execute(
            "UPDATE candado SET service=?2, email=?3, password=?4, username=?5, url=?6 WHERE entry_id=?1",
            params![
                entry.id, 
                entry.service, 
                entry.email, 
                entry.password, 
                entry.username, 
                entry.url,
            ],
        )?;
        Ok(())
    }

    /// gets a list of decrypted entries from vault matching the query
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to read db
    /// * can't decrypt entries
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let entries: Vec<Entry> = storage.find("some service")?;
    /// 
    pub fn find(&self, query: &str) -> Result<Vec<Entry>> {
        let matcher = SkimMatcherV2::default();
        let entries = self.list()?;
        let result: Vec<Entry> = entries.iter().filter_map(|entry| {
            matcher.fuzzy_match(&format!("{}", entry), query).map(|_| entry.clone())
        }).collect();
        Ok(result)
    }

    /// gets a list of decrypted entries from vault
    /// 
    /// # Panics
    /// This function will panic if:
    /// * fails to read db
    /// * can't decrypt entries
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// let entries: Vec<Entry> = storage.list()?;
    /// 
    pub fn list(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare("SELECT * FROM candado")?;
        let enries = stmt.query_map([], |row| {
            Ok(EncryptedEntry {
                id: row.get(1)?,
                service: row.get(2)?,
                email: row.get(3)?,
                password: row.get(4)?,
                username: row.get(5)?,
                url: row.get(6)?,
            })
        })?;
        let mut result: Vec<Entry> = vec![];
        for entry in enries {
            let entry = entry.unwrap();
            result.push(entry.decrypt(&self.encrypter)?);
        }
        Ok(result)
    }

    pub fn load_json(source: PathBuf) -> Result<SupportedFile> {
        Ok(SupportedFile::JSON(fs::read_to_string(source)?))
    }

    /// Imports entries from .json file
    /// Will add support for other import file formats in future releasea
    ///
    /// Need
    ///
    /// # Panics
    /// This function will panic if:
    /// * File format is not supported or with no extension is provided
    /// * can't deserialize entries
    /// * cant't write to db
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// storage.import("backup.json")?;
    ///
    pub fn import(&mut self, filepath: PathBuf) -> Result<()> {
        // verify corret file
        let file = if let Some(extention) = filepath.extension() {
            match extention.to_str() {
                Some("json") => Storage::load_json(filepath)?,
                _ => return Err(anyhow!("File not supported")),
            }
        } else {
            return Err(anyhow!("Inalid filetype"));
        };

        match file {
            SupportedFile::JSON(data) => {
                let entries: Vec<ImportedEntry> = serde_json::from_str(&data)?;
                let total = entries.len();

                for (i, import) in entries.into_iter().enumerate() {
                    // loading bar
                    let percent = ((i + 1) as f64 / total as f64) * 100.0;
                    let bar = "=".repeat(percent.ceil() as usize) + &" ".repeat((100.0 - percent).ceil() as usize);
                    print!("\r[{}] {:.0}% | [{}/{}]", bar, percent, i + 1, total);
                    std::io::stdout().flush().unwrap();
                    self.write(Entry::from(import))?;
                }
                println!("");
            }
            _ => todo!("import of this type are not supported yet")
        }
        Ok(())
    }

    /// Exports decrypted entries to .json file
    /// Will add support for other export file formats in future release
    ///
    /// # Panics
    /// This function will panic if:
    /// * can't load all entries
    /// * cant't open/write file
    /// * can't serialize entries
    ///
    /// # Basic usage:
    /// 
    /// let password: &str = "password";
    /// let enc = Encrypter::unlock(password)?;
    /// let storage = Storage::init(&encrypter)?;
    /// storage.export("backup.json")?;
    ///
    pub fn export(&self, path: PathBuf) -> Result<()> {
        let entries = self.list()?; // get all entries
        let mut file = File::options().write(true).create(true).open(path)?;
        let objects = serde_json::to_string_pretty(&entries)?;
        writeln!(file, "{}", objects)?;
        Ok(())
    }
}

