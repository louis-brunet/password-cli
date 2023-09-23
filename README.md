## Model

Entry(name: string, username: string, password: string)

Group(name: string)

```
Entry  *<------>1     Group
Group  *<------>0..1  Group
```

User data is symmetrically encrypted with the user's master password. 

```sql
CREATE TABLE Entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES EntryGroup(id),
    data BLOB NOT NULL -- encrypted
);

CREATE TABLE EntryGroup (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data BLOB NOT NULL -- encrypted
);
```

# Basic usage

```bash
cargo run -- <COMMAND> [-c <CREDENTIALS_FILE>]
```

If no credentials file is given, the username and password are read from stdin.
By default, the input password is hidden with [rpassword](https://crates.io/crates/rpassword).
Use `--show-password` to override this if needed.

The database file name is derived from the username.

See `cargo run -- --help` for full usage help.

## Create an encrypted database

```bash
cargo run -- create
```

## Add an entry

```bash
cargo run -- add entry
```

The user is prompted for the entry's data: parent group id, entry name, 
username, password.

## Get all data for each entry

```bash
cargo run -- get entries --all
```
