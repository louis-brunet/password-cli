use std::{fs, path::Path};

use aes_gcm::{aead::consts::U12, Nonce};
use fallible_iterator::FallibleIterator;
use rusqlite::{Connection, OpenFlags};

use crate::encryption::{self, Cipher, DecryptedMessage};

use super::{Entry, EntryData, EntryGroup, EntryGroupData};

pub struct Database<'a> {
    conn: Connection,
    cipher: &'a Cipher,
}

impl<'a> Database<'a> {
    const ROOT_GROUP_ID: i64 = 1;
    const CHALLENGE: &[u8; 32] = &[
        71, 241, 152, 110, 194, 42, 171, 124, 37, 122, 211, 128, 57, 254, 234, 253, 82, 237, 131,
        16, 141, 243, 50, 140, 6, 250, 169, 73, 249, 69, 19, 174,
    ];

    pub fn create(path: impl AsRef<Path>, cipher: &'a Cipher) -> Result<Self, ()> {
        if path.as_ref().exists() {
            return Err(());
        }

        let conn = Connection::open(path.as_ref()).map_err(|_| ())?;
        let db = Database { conn, cipher };
        let challenge = Self::CHALLENGE;
        db.init_test_tables();

        db.conn
            .execute(
                "INSERT INTO Metadata(challenge) VALUES (?1);",
                [encrypt_blob(challenge.as_slice(), db.cipher)],
            )
            .unwrap();

        return Ok(db);
    }

    pub fn open(path: impl AsRef<Path>, cipher: &'a Cipher) -> Result<Self, rusqlite::Error> {
        // let mut is_new = false;
        let conn = Connection::open_with_flags(
            path.as_ref(),
            OpenFlags::default() & !OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        // .or_else(|err| {
        //     if let rusqlite::Error::SqliteFailure(
        //         rusqlite::ffi::Error {
        //             code: rusqlite::ffi::ErrorCode::CannotOpen,
        //             ..
        //         },
        //         ..,
        //     ) = err
        //     {
        //         is_new = true;
        //         Connection::open(path.as_ref())
        //     } else {
        //         Err(err)
        //     }
        // })?;

        let db = Database { conn, cipher };
        let challenge = Self::CHALLENGE;
        // if is_new {
        //     db.init_test_tables();
        //
        //     db.conn
        //         .execute(
        //             "INSERT INTO Metadata(challenge) VALUES (?1);",
        //             [encrypt_blob(challenge.as_slice(), db.cipher)],
        //         )
        //         .unwrap();
        // } else {
        // verify that the given cipher can decrypt this db
        let encrypted_challenge: Vec<u8> =
            db.conn
                .query_row("SELECT challenge FROM Metadata", [], |row| row.get(0))?;

        let decrypted_challenge = decrypt_blob(encrypted_challenge.as_slice(), db.cipher).unwrap();

        assert_eq!(decrypted_challenge.as_slice(), challenge.as_slice());
        // }

        return Ok(db);
    }

    pub fn root_group_id(&self) -> i64 {
        return Self::ROOT_GROUP_ID;
        //         return self
        //             .conn
        //             .query_row(
        //                 "
        // SELECT id
        // FROM EntryGroup e
        // WHERE e.id NOT IN (
        //     SELECT child_id FROM EntryGroupParent
        // );",
        //                 [],
        //                 |row| row.get(0),
        //             )
        //             .expect("there should be a group with no parent");
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
            encrypt_blob(serialized.as_slice(), self.cipher),
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

    pub fn entries(
        &self,
        name_filter: Option<&str>,
        parent_group_id: Option<i64>,
    ) -> Result<Vec<Entry>, ()> {
        let mut stmt;
        let rows = if let Some(parent_id) = parent_group_id {
            stmt = self
                .conn
                .prepare(
                    "
SELECT id, data FROM Entry
WHERE group_id = ?1;
",
                )
                .unwrap();
            stmt.query([parent_id]).unwrap()
        } else {
            stmt = self.conn.prepare("SELECT id, data FROM Entry").unwrap();
            stmt.query([]).unwrap()
        };
        let rows = rows.map(|row| Ok((row.get(0)?, row.get(1)?)));
        // stmt.query_map([parent_id], |row| Ok((row.get(0)?, row.get(1)?))).unwrap()

        return Ok(rows
            .filter_map(|(entry_id, blob_buf)| {
                // if let Ok((entry_id, blob_buf)) = mapped_row {
                let decrypted = decrypt_blob(Vec::as_slice(&blob_buf), self.cipher).unwrap();
                let entry_data: EntryData = ciborium::from_reader(decrypted.as_slice()).unwrap();

                if let Some(name_filter) = name_filter {
                    if !entry_data.name().contains(name_filter) {
                        return Ok(None);
                    }
                }
                return Ok(Some(Entry::new(entry_id, entry_data)));
                // } else {
                //     return None;
                // }
            })
            .collect()
            .unwrap());
    }

    pub fn groups(&self, parent_group_id: Option<i64>) -> Result<Vec<EntryGroup>, ()> {
        //         let (mut stmt, params) = if let Some(parent_id) = parent_group_id {
        //             (
        //                 self.conn
        //                     .prepare("
        // SELECT id, data FROM EntryGroup
        // WHERE id IN (
        //     SELECT child_id FROM EntryGroupParent
        //     WHERE parent_id = ?1
        // );")
        //                     .unwrap(),
        //                 vec![parent_id],
        //             )
        //             // stmt.query([]).unwrap()
        //             // todo!()
        //         } else {
        //             (
        //                 self.conn
        //                     .prepare("SELECT id, data FROM EntryGroup")
        //                     .unwrap(),
        //                 vec![],
        //             )
        //             // stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap()
        //         };
        //         let rows = stmt.query(params.as_slice()).unwrap();
        let mut stmt;
        let rows = if let Some(parent_id) = parent_group_id {
            stmt = self
                .conn
                .prepare(
                    "
SELECT id, data FROM EntryGroup
WHERE id IN (
    SELECT child_id FROM EntryGroupParent
    WHERE parent_id = ?1
);",
                )
                .unwrap();
            stmt.query([parent_id])
        } else {
            stmt = self
                .conn
                .prepare("SELECT id, data FROM EntryGroup;")
                .unwrap();
            stmt.query([])
        };

        let groups = rows
            .unwrap()
            .map(|row| {
                let group_id = row.get(0).unwrap();
                let data_blob = row.get(1).unwrap();

                let decrypted = decrypt_blob(Vec::as_slice(&data_blob), self.cipher).unwrap();
                let group_data: EntryGroupData =
                    ciborium::from_reader(decrypted.as_slice()).unwrap();

                return Ok(EntryGroup::new(group_id, group_data));
            })
            .collect()
            .unwrap();

        return Ok(groups);
    }

    // TODO: use Option<i64> for parent id to not make querying DB beforehand
    //   necessary, but that would mean not being able to create a root-level
    //   group with this method.
    pub fn insert_entry_group(
        &mut self,
        parent_group_id: i64,
        group_data: &EntryGroupData,
    ) -> Result<i64, ()> {
        let mut serialized: Vec<u8> = vec![];
        let tx = self.conn.transaction().unwrap();

        ciborium::into_writer(group_data, &mut serialized).unwrap();
        tx.execute(
            "INSERT INTO EntryGroup(data) VALUES (?1);",
            rusqlite::params![encrypt_blob(serialized.as_slice(), self.cipher),],
        )
        .unwrap();
        let entry_group_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO EntryGroupParent(parent_id, child_id) VALUES (?1, ?2);",
            rusqlite::params![parent_group_id, entry_group_id,],
        )
        .unwrap();

        tx.commit().unwrap();

        return Ok(entry_group_id);
    }

    // TODO: refactor copied code from Database::insert_entry_group
    pub fn insert_root_entry_group(&self, group_data: &EntryGroupData) -> Result<i64, ()> {
        let mut serialized: Vec<u8> = vec![];
        ciborium::into_writer(group_data, &mut serialized).unwrap();
        self.conn
            .execute(
                "INSERT INTO EntryGroup(id, data) VALUES (?1, ?2);", // TODO: use fixed id =1 or-1 or
                // something
                rusqlite::params![
                    Self::ROOT_GROUP_ID,
                    encrypt_blob(serialized.as_slice(), self.cipher),
                ],
            )
            .unwrap();

        return Ok(self.conn.last_insert_rowid());
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

        let group_id = self
            .insert_root_entry_group(&EntryGroupData::new("root group name".into()))
            .unwrap();

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

        self.insert_entry(
            group_id,
            &EntryData::new("entry name".into(), "u".into(), "password123".into()),
        )
        .unwrap();

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

#[cfg(test)]
mod test {
    use crate::{encryption::Cipher, model::database::{encrypt_blob, decrypt_blob}};

    #[test]
    fn encrypt_then_decrypt_blob() {
        let blob_data= b"some data";
        let cipher_secret = b"secret key";
        let test_cipher = Cipher::new(&cipher_secret.as_slice().into());
        let encrypted = encrypt_blob(blob_data, &test_cipher);
        let decrypted = decrypt_blob(encrypted.as_slice(), &test_cipher).unwrap();

        assert_eq!(decrypted.as_slice(), blob_data);
    }
}

