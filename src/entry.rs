use anyhow::Result;
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::{generators, Encrypter};

#[derive(Debug, Serialize, Default, Clone)]
pub struct Entry {
    pub id: String,
    pub service: String,
    pub email: String,
    pub password: String,
    pub username: String,
    pub url: String,
}

pub struct EncryptedEntry {
    pub id: Vec<u8>,
    pub service: Vec<u8>,
    pub email: Vec<u8>,
    pub password: Vec<u8>,
    pub username: Vec<u8>,
    pub url: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct ImportedEntry {
    pub service: String,
    pub email: String,
    pub password: String,
    pub username: String,
    pub url: String,
}

impl From<ImportedEntry> for Entry {
    fn from(value: ImportedEntry) -> Self {
        Entry::new(
            value.service,
            value.email,
            Some(value.password),
            Some(value.username),
            Some(value.url),
        )
    }
}

pub trait Decrypt {
    fn decrypt(&self, encrypter: &Encrypter) -> Result<Entry>;
}

impl Decrypt for EncryptedEntry {
    fn decrypt(&self, encrypter: &Encrypter) -> Result<Entry> {
        Ok(Entry::init(
            encrypter.decrypt(&self.id)?,
            encrypter.decrypt(&self.service)?,
            encrypter.decrypt(&self.email)?,
            encrypter.decrypt(&self.password)?,
            encrypter.decrypt(&self.username)?,
            encrypter.decrypt(&self.url)?,
        ))
    }
}

pub trait Encrypt {
    fn encrypt(&self, encrypter: &Encrypter) -> Result<EncryptedEntry>;
}

impl Encrypt for Entry {
    fn encrypt(&self, encrypter: &Encrypter) -> Result<EncryptedEntry> {
        Ok(EncryptedEntry::init(
            encrypter.encrypt(&self.id)?,
            encrypter.encrypt(&self.service)?,
            encrypter.encrypt(&self.email)?,
            encrypter.encrypt(&self.password)?,
            encrypter.encrypt(&self.username)?,
            encrypter.encrypt(&self.url)?,
        ))
    }
}

impl EncryptedEntry {
    pub fn init(
        id: Vec<u8>,
        service: Vec<u8>,
        email: Vec<u8>,
        password: Vec<u8>,
        username: Vec<u8>,
        url: Vec<u8>,
    ) -> Self {
        Self {
            id,
            service,
            email,
            password,
            username,
            url,
        }
    }
}

impl Entry {
    pub fn init(
        id: String,
        service: String,
        email: String,
        password: String,
        username: String,
        url: String,
    ) -> Self {
        Self {
            id,
            service,
            email,
            password,
            username,
            url,
        }
    }

    pub fn new(
        service: String,
        email: String,
        password: Option<String>,
        username: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            id: generators::gen_key(12),
            service,
            email,
            password: password.unwrap_or(generators::gen_passphrase(4, &None)),
            username: username.unwrap_or(String::new()),
            url: url.unwrap_or(String::new()),
        }
    }

    pub fn overite(
        &mut self,
        service: Option<String>,
        email: Option<String>,
        password: Option<String>,
        username: Option<String>,
        url: Option<String>,
    ) {
        macro_rules! update_if_some {
            ($self:ident, $($field:ident, $value:expr),*) => {
                $(
                    if let Some(val) = $value {
                        $self.$field = val;
                    }
                )*
            };
        }
        update_if_some!(self, service, service);
        update_if_some!(self, email, email);
        update_if_some!(self, password, password);
        update_if_some!(self, username, username);
        update_if_some!(self, url, url);
    }

    pub const fn ref_array(&self) -> [&String; 6] {
        [
            &self.id,
            &self.service,
            &self.email,
            &self.password,
            &self.username,
            &self.url,
        ]
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {} | {}",
            self.id, self.service, self.email, self.password, self.username, self.url
        )
    }
}
