use core::fmt;
use serde::{Deserialize, Serialize};

use crate::generators;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entry {
    pub id: String,
    pub service: String,
    pub email: String,
    pub password: String,
    pub username: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct RawEntry {
    pub id: String,
    pub service: Vec<u8>,
    pub email: Vec<u8>,
    pub password: Vec<u8>,
    pub username: Vec<u8>,
    pub url: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct ImportEntry {
    pub service: String,
    pub email: String,
    pub password: String,
    pub username: String,
    pub url: String,
}

impl From<ImportEntry> for Entry {
    fn from(value: ImportEntry) -> Self {
        Entry::new(
            generators::gen_key(12),
            value.service,
            value.email,
            value.password,
            value.username,
            value.url,
        )
    }
}

impl Entry {
    pub fn new(
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
