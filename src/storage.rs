use std::{fs::{self, File}, io::Write, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

use crate::{entry::{Entry, ImportEntry, RawEntry}, Encrypter};
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
    pub fn init(encrypter: &'unlocked Encrypter) -> Result<Self> {
        // Linux
        #[cfg(target_os = "linux")]
        let path = format!("{}/.candado/candado.db", std::env::var("HOME")?);

        // windows
        // #[cfg(target_os = "windows")]
        // let path = format!("{}/.candado/.candado.db", std::env::var("USERHOME")?);

        let db_path = Path::new(&path);
        if !db_path.exists() {
            File::create(db_path)?;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(db_path, permissions)?;
        }

        let conn = Connection::open(db_path)?;
        // conn.pragma_update(None, "key", &encrypter.derived_key)?;
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

    pub fn write(&self, entry: Entry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO candado (entry_id, service, email, password, username, url) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id, 
                self.encrypter.encrypt(&entry.service)?, 
                self.encrypter.encrypt(&entry.email)?, 
                self.encrypter.encrypt(&entry.password)?, 
                self.encrypter.encrypt(&entry.username)?, 
                self.encrypter.encrypt(&entry.url)?,
            ],
        )?;
        Ok(())
    }


    pub fn remove(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM candado WHERE entry_id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn read(&self, id: &str) -> Result<Entry> {
        let mut stmt = self.conn.prepare("SELECT * FROM candado WHERE entry_id=?1")?;
        let entry = stmt.query_row(params![id], |row| {
            Ok(RawEntry {
                id: row.get(1)?,
                service: row.get(2)?,
                email: row.get(3)?,
                password: row.get(4)?,
                username: row.get(5)?,
                url: row.get(6)?,
            })
        })?;
        let entry = Entry::new(
            entry.id, 
            self.encrypter.decrypt(&entry.service)?, 
            self.encrypter.decrypt(&entry.email)?, 
            self.encrypter.decrypt(&entry.password)?, 
            self.encrypter.decrypt(&entry.username)?, 
            self.encrypter.decrypt(&entry.url)?,
        );
        Ok(entry)
    }

    pub fn update(&self, entry: Entry) -> Result<()> {
        self.conn.execute(
            "UPDATE candado SET service=?2, email=?3, password=?4, username=?5, url=?6 WHERE entry_id=?1",
            params![
                entry.id, 
                self.encrypter.encrypt(&entry.service)?, 
                self.encrypter.encrypt(&entry.email)?, 
                self.encrypter.encrypt(&entry.password)?, 
                self.encrypter.encrypt(&entry.username)?, 
                self.encrypter.encrypt(&entry.url)?,
            ],
        )?;
        Ok(())
    }

    pub fn find(&self, query: &str) -> Result<Vec<Entry>> {
        let matcher = SkimMatcherV2::default();
        let entries = self.list()?;
        let result: Vec<Entry> = entries.iter().filter_map(|entry| {
            matcher.fuzzy_match(&format!("{}", entry), query).map(|_| entry.clone())
        }).collect();
        Ok(result)
    }

    pub fn list(&self) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare("SELECT * FROM candado")?;
        let enries = stmt.query_map([], |row| {
            Ok(RawEntry {
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
            result.push(Entry::new(
                entry.id, 
                self.encrypter.decrypt(&entry.service)?, 
                self.encrypter.decrypt(&entry.email)?, 
                self.encrypter.decrypt(&entry.password)?, 
                self.encrypter.decrypt(&entry.username)?, 
                self.encrypter.decrypt(&entry.url)?,
            ));
        }
        Ok(result)
    }

    pub fn load_json(path: PathBuf) -> Result<SupportedFile> {
        Ok(SupportedFile::JSON(fs::read_to_string(path)?))
    }
    
    pub fn load_sqldump(path: PathBuf) -> Result<SupportedFile> {
        Ok(SupportedFile::SQL(fs::read_to_string(path)?))
    }

    /// (optional add different import formats sql.dump
    /// for now lets support json)
    pub fn import(&mut self, filepath: PathBuf) -> Result<()> {
        let file = if let Some(path) = filepath.extension() {
            // verify corret file
            match path.to_str() {
                Some("json") => Storage::load_json(filepath)?,
                Some("sql") => Storage::load_sqldump(filepath)?,
                _ => return Err(anyhow!("File not supported")),
            }
        } else {
            return Err(anyhow!("Inalid filetype"));
        };

        // convert to Entry
        match file {
            SupportedFile::JSON(data) => {
                let entries: Vec<ImportEntry> = serde_json::from_str(&data)?;
                let total = entries.len();
                for (i, import) in entries.into_iter().enumerate() {
                    let percent = ((i + 1) as f64 / total as f64) * 100.0;
                    // Create the loading bar string
                    let bar = "=".repeat(percent.ceil() as usize) + &" ".repeat((100.0 - percent).ceil() as usize);
                    // Print the loading bar on the same line

                    print!("\r[{}] {:.0}% | [{}/{}]", bar, percent, i + 1, total);
                    // Flush the output to make it appear immediately
                    std::io::stdout().flush().unwrap();
                    // Simulate work by sleeping
                    let entry = Entry::from(import);
                    self.write(entry)?;
                }
                println!("");
            }
            SupportedFile::SQL(_data) => todo!(".sql imports are not supported yet")
        }
        Ok(())
    }

    /// (optional add different export formats sql.dump
    /// for now lets support json)
    pub fn export(&self, path: PathBuf) -> Result<()> {
        let entries = self.list()?;
        let mut file = File::options().write(true).create(true).open(path)?;
        let objects = serde_json::to_string_pretty(&entries)?;
        writeln!(file, "{}", objects)?;
        Ok(())
    }
}

