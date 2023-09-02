use serde::{Deserialize, Serialize};

pub mod database;

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryGroupData {
    group_name: Box<str>,
}

impl EntryGroupData {
    pub fn new(group_name: Box<str>) -> Self {
        Self { group_name }
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
