mod bench;
pub mod model;
pub mod encryption;

// #[derive(Debug)]
// pub struct PasswordManagerOpenError;
//
// pub struct PasswordManager {
//     credentials: Credentials,
//     root: Group,
// }
//
// impl PasswordManager {
//     pub fn open(credentials: Credentials) -> Result<Self, PasswordManagerOpenError> {
//         // TODO: connect to db or something
//         let mut test_group = Group::new("root".to_string());
//         test_group
//             .add_entry(Entry::new(
//                 "test_entry".to_string(),
//                 "username".to_string(),
//                 "pw".to_string(),
//             ))
//             .expect("should be able to add any entry to empty group");
//         test_group
//             .add_entry(Entry::new(
//                 "another entry".to_string(),
//                 "another_username".to_string(),
//                 "123456".to_string(),
//             ))
//             .expect("should be able to add any entry");
//
//         return Ok(PasswordManager {
//             credentials,
//             root: test_group,
//         });
//     }
//
//     pub fn root(&self) -> &Group {
//         &self.root
//     }
//
//     /// TODO: more generic filtering
//     pub fn entries(&self, name: Option<&str>) -> Vec<&Entry> {
//         if let Some(name) = name {
//             let trimmed_name = name.trim();
//             return self
//                 .root
//                 .entries
//                 .iter()
//                 .filter(|entry| entry.entry_name.contains(trimmed_name))
//                 .collect();
//         }
//
//         return self.root.entries.iter().collect();
//     }
// }

pub struct Credentials {
    user: Box<str>,
    password: Box<str>,
}

impl Credentials {
    pub fn new(user: Box<str>, password: Box<str>) -> Self {
        Self { user, password }
    }
    // pub fn to_cipher_key(&self) ->

    pub fn user(&self) -> &str {
        self.user.as_ref()
    }
}

impl<'a> From<&'a Credentials> for [&'a [u8]; 2] {
    fn from(cred: &'a Credentials) -> Self {
        return [cred.user.as_bytes(), cred.password.as_bytes()];
    }
}

// pub struct Group {
//     group_name: String,
//     groups: Vec<Group>,
//     entries: Vec<EntryData>,
// }
//
// impl Group {
//     pub fn new(group_name: String) -> Self {
//         Self {
//             group_name,
//             groups: vec![],
//             entries: vec![],
//         }
//     }
//
//     fn add_entry(&mut self, entry: EntryData) -> Result<(), ()> {
//         // TODO: check duplicate entry name ?
//
//         self.entries.push(entry);
//
//         return Ok(());
//     }
//
//     pub fn name(&self) -> &str {
//         self.group_name.as_ref()
//     }
//
//     pub fn entries(&self) -> &[EntryData] {
//         self.entries.as_ref()
//     }
//
//     pub fn groups(&self) -> &[Group] {
//         self.groups.as_ref()
//     }
// }

