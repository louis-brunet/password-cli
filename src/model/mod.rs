use serde::{Deserialize, Serialize};

use crate::encryption::CipherKey;

pub mod database;

pub struct Credentials {
    user: Box<str>,
    password: Box<str>,
}

impl Credentials {
    pub fn new(user: Box<str>, password: Box<str>) -> Self {
        Self { user, password }
    }

    pub fn user(&self) -> &str {
        self.user.as_ref()
    }

    // pub fn hash(&self) -> [u8; 32] {
    //     let mut key_hasher = hmac_sha256::Hash::new();
    //     key_hasher.update(self.user.as_bytes());
    //     key_hasher.update(self.password.as_bytes());
    //     return key_hasher.finalize();
    // }
}

impl From<&Credentials> for CipherKey {
    fn from(credentials: &Credentials) -> Self {
        let mut key_hasher = hmac_sha256::Hash::new();
        key_hasher.update(credentials.user.as_bytes());
        key_hasher.update(credentials.password.as_bytes());
        let key_bytes = key_hasher.finalize();
        return CipherKey::new(key_bytes);
    }
}

pub struct EntryGroup {
    id: i64,
    data: EntryGroupData,
}

impl EntryGroup {
    pub fn new(id: i64, data: EntryGroupData) -> Self {
        Self { id, data }
    }

    pub fn data(&self) -> &EntryGroupData {
        &self.data
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryGroupData {
    group_name: Box<str>,
}

impl EntryGroupData {
    pub fn new(group_name: Box<str>) -> Self {
        Self { group_name }
    }

    pub fn name(&self) -> &str {
        self.group_name.as_ref()
    }
}

#[derive(Debug)]
pub struct Entry {
    id: i64,
    data: EntryData,
}

impl Entry {
    pub fn new(id: i64, data: EntryData) -> Self {
        Self { id, data }
    }

    pub fn data(&self) -> &EntryData {
        &self.data
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryData {
    entry_name: Box<str>,
    username: Box<str>,
    password: Box<str>,
}

impl EntryData {
    pub fn new(entry_name: Box<str>, username: Box<str>, password: Box<str>) -> Self {
        Self {
            entry_name,
            username,
            password,
        }
    }

    pub fn name(&self) -> &str {
        self.entry_name.as_ref()
    }

    pub fn username(&self) -> &str {
        self.username.as_ref()
    }

    pub fn password(&self) -> &str {
        self.password.as_ref()
    }
}
