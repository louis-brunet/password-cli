use std::{fs, io::ErrorKind, path::Path};

use aes_gcm::{aead::consts::U12, Nonce};
use rusqlite::{Connection, OpenFlags};

use crate::encryption::{self, Cipher, DecryptedMessage};

use super::{Entry, EntryData};

pub struct Database<'a> {
    conn: Connection,
    cipher: &'a Cipher,
}

impl<'a> Database<'a> {
    pub fn open(path: impl AsRef<Path>, cipher: &'a Cipher) -> Result<Self, rusqlite::Error> {
        let mut is_new = false;
        let conn = Connection::open_with_flags(
            path.as_ref(),
            OpenFlags::default() & !OpenFlags::SQLITE_OPEN_CREATE,
        )
        .or_else(|err| {
            if let rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::CannotOpen,
                    ..
                },
                ..,
            ) = err
            {
                is_new = true;
                Connection::open(path.as_ref())
            } else {
                Err(err)
            }
        })?;

        let db = Database { conn, cipher };
        if is_new {
            db.init_test_tables();
        }

        return Ok(db);
    }

    pub fn root_group_id(&self) -> i64 {
        return self
            .conn
            .query_row(
                "
SELECT id 
FROM EntryGroup e 
WHERE e.id NOT IN (
    SELECT child_id FROM EntryGroupParent
);",
                [],
                |row| row.get(0),
            )
            .expect("there should be a group with no parent");
    }

    pub fn entry(&self, row_id: i64) -> Result<Entry, ()> {
        let entry_data = self.decrypt_blob("Entry", "data", row_id).unwrap();
        let deserialized = ciborium::from_reader(entry_data.as_slice()).unwrap();
        return Ok(Entry::new(row_id, deserialized));
    }

    pub fn insert_entry(&self, group_id: i64, entry: &EntryData) -> Result<i64, ()> {
        let mut serialized: Vec<u8> = vec![];
        ciborium::into_writer(entry, &mut serialized).unwrap();
        let mut stmt = self
            .conn
            .prepare("INSERT INTO Entry(group_id, data) VALUES (?, ?);")
            .unwrap();
        stmt.execute(rusqlite::params![
            group_id,
            encrypt_blob(serialized.as_slice(), self.cipher)
        ])
        .unwrap();

        //         self.conn
        //             .execute(
        //                 "
        // INSERT INTO Entry(group_id, data) VALUES (?, ?);
        // ",
        //                 rusqlite::params![group_id, encrypt_blob(serialized.as_slice(), self.cipher),],
        //             )
        //             .unwrap(); //TODO: error handling

        return Ok(self.conn.last_insert_rowid());
    }

    pub fn entries(&self, name_filter: Option<&str>) -> Result<Vec<Entry>, ()> {
        let mut stmt = self.conn.prepare("SELECT id, data FROM Entry").unwrap();
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();

        return Ok(rows
            .filter_map(|mapped_row| {
                if let Ok((entry_id, blob_buf)) = mapped_row {
                    let decrypted = decrypt_blob(Vec::as_slice(&blob_buf), self.cipher).unwrap();
                    let entry_data: EntryData =
                        ciborium::from_reader(decrypted.as_slice()).unwrap();

                    if let Some(name_filter) = name_filter {
                        if !entry_data.name().contains(name_filter) {
                            return None;
                        }
                    }
                    return Some(Entry::new(entry_id, entry_data));
                } else {
                    return None;
                }
            })
            .collect());
    }

    pub fn insert_entry_group(&self, parent_group_id: i64) -> Result<i64, ()> {
        todo!()
    }

    pub fn init_test_tables(&self) {
        let creation_script_path = std::env::var("DB_CREATION_SCRIPT")
            .expect("missing DB_CREATION_SCRIPT environment variable");

        self.conn
            .execute_batch(
                fs::read_to_string(creation_script_path)
                    .expect("could not read DB creation script")
                    .as_str(),
            )
            .unwrap();
        log::trace!("created tables");

        // Insert test data
        self.conn
            .execute(
                "
INSERT INTO EntryGroup(data) VALUES (:data);
",
                rusqlite::named_params! { ":data": b"test root group data" },
            )
            .unwrap();

        let group_id = self.conn.last_insert_rowid();
        log::trace!("inserted group (id={})", group_id);

        let entry_one_blob_id = self
            .insert_entry(
                group_id,
                &EntryData::new("abcdef".into(), "fghijkl".into(), "password".into()),
            )
            .unwrap();

        let inserted_entry_one_blob = self.entry(entry_one_blob_id).unwrap();
        log::trace!(
            "inserted encrypted entry with one data blob: {:?}",
            inserted_entry_one_blob
        );

        // bench::bench(
        //     || {
        //         self.entry_with_read(entry_one_blob_id);
        //     },
        //     "db.entry_with_read()",
        // );
        //
        // bench::bench(
        //     || {
        //         self.entry(entry_one_blob_id);
        //     },
        //     "db.entry()",
        // );
        //
        // let insert_counts = [1, 10, 50];
        //
        // let entry = Entry::new("a".into(), "b".into(), "c".into());
        // bench::bench_with_counts(
        //     || {
        //         self.insert_entry(group_id, &entry).unwrap();
        //     },
        //     "db.insert_entry() (no alloc)",
        //     &insert_counts,
        // );
        // self.conn
        //     .execute("DELETE FROM Entry WHERE 1 = 1;", [])
        //     .unwrap();
        //
        // bench::bench_with_counts(
        //     || {
        //         let entry = Entry::new("a".into(), "b".into(), "c".into());
        //         self.insert_entry(group_id, &entry).unwrap();
        //     },
        //     "db.insert_entry() (alloc)",
        //     &insert_counts,
        // );
        // self.conn
        //     .execute("DELETE FROM Entry WHERE 1 = 1;", [])
        //     .unwrap();
    }

    fn decrypt_blob(&self, table: &str, column: &str, row_id: i64) -> Result<DecryptedMessage, ()> {
        let query = format!("SELECT \"{}\" FROM \"{}\" WHERE id = ?", column, table);
        let blob_buf: Vec<u8> = self
            .conn
            .query_row(query.as_str(), rusqlite::params![row_id], |row| row.get(0))
            .unwrap();

        return decrypt_blob(blob_buf.as_slice(), self.cipher);
        // let blob = self
        //     .conn
        //     .blob_open(DatabaseName::Main, table, column, row_id, true)
        //     .unwrap();
        // return decrypt_blob(&blob, self.cipher);
    }
}

/// Encrypt the data and return a vector of bytes containing the nonce
/// concatenated with the encrypted data.
///
/// TODO: error handling
fn encrypt_blob(data: &[u8], cipher: &Cipher) -> Vec<u8> {
    let msg = cipher.encrypt(data).unwrap();
    return msg
        .nonce()
        .iter()
        .chain(msg.ciphertext().iter())
        .copied()
        .collect();
}

/// TODO: error handling
fn decrypt_blob(blob_buf: &[u8], cipher: &Cipher) -> Result<Vec<u8>, ()> {
    let (nonce_buf, payload_buf) = blob_buf.split_at(encryption::NONCE_SIZE);
    let nonce_buf: &[u8; encryption::NONCE_SIZE] = nonce_buf.try_into().unwrap();
    let nonce: &Nonce<U12> = nonce_buf.into();

    let decrypted_data = cipher.decrypt(payload_buf.as_ref(), nonce).unwrap();

    return Ok(decrypted_data);
}
