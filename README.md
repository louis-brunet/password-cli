## Model

Entry(name: string, username: string, password: string)

Group(name: string)

```
Entry  *<------>1     Group
Group  *<------>0..1  Group
```

User data is encrypted. 


```sql
CREATE TABLE Entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES EntryGroup(id),
    data BLOB NOT NULL
);
```

