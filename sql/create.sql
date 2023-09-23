BEGIN;

CREATE TABLE Metadata (
    challenge BLOB NOT NULL
);

CREATE TABLE EntryGroup (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data BLOB NOT NULL
);

CREATE TABLE Entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES EntryGroup(id),
    data BLOB NOT NULL
);

CREATE TABLE EntryGroupParent (
    parent_id INTEGER NOT NULL REFERENCES EntryGroup(id),
    child_id INTEGER NOT NULL REFERENCES EntryGroup(id),

    PRIMARY KEY(parent_id, child_id)
);

COMMIT;

