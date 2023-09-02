## Model

Entry(name: string, username: string, password: string)
Group(name: string)
User(username: string, password_hash: string)

```
Entry  *<------>1     Group
Group  *<------>0..1  Group
Group  1..*<--->1     User
```

User master passwords are hashed.
User data is encrypted. 


```sql
CREATE TABLE entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name BLOB,
    username BLOB,
    password BLOB,
    -- name VARCHAR2(50),
    -- username VARCHAR2(100),
    -- password VARCHAR2(100)
);
```

