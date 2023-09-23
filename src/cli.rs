use std::{
    env,
    io::{self, Write},
    num::IntErrorKind,
    path::PathBuf,
};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::{
    encryption::Cipher,
    model::{database::Database, Credentials, EntryData, EntryGroupData},
};

/// Command-line password manager
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    command: Commands,

    /// credentials file containing the username and password
    #[arg(short, long)]
    credentials_file: Option<PathBuf>,

    /// do not hide password inputs on stdin (not recommended if running from a
    /// terminal)
    #[arg(long)]
    show_password: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// create a new encrypted database
    #[command(visible_alias = "c")]
    Create,

    /// retrieve entry or group data
    #[command(visible_alias = "g")]
    Get {
        #[command(subcommand)]
        command: GetCommand,

        #[arg(short, long, default_value_t = String::from("\t"))]
        separator: String,
    },

    /// add an entry or a group
    #[command(visible_alias = "a")]
    Add {
        #[command(subcommand)]
        command: AddCommand,
    },

    /// generate miscellaneous files, print them to stdout
    Gen {
        /// generate zsh completion script
        #[command(subcommand)]
        target: GenTarget,
    },
}

#[derive(Subcommand, Debug)]
enum GenTarget {
    /// print zsh completion to stdout
    Zsh { cmd_name: PathBuf },
}

#[derive(Subcommand, Debug)]
enum GetCommand {
    #[command(visible_alias = "e")]
    Entries {
        /// filters the results to those with the given parent group id
        #[arg(short = 'g', long)]
        parent_group: Option<i64>,

        /// filters the results to those matching the given name
        #[arg(short, long)]
        name: Option<String>,

        /// get the username from each entry
        #[arg(short, long)]
        username: bool,

        /// get the password from each entry
        #[arg(short, long)]
        password: bool,

        /// get the id from each entry
        #[arg(short, long)]
        id: bool,

        /// get all values associated with each entry (name, username, password, etc.)
        #[arg(short, long)]
        all: bool,
    },

    #[command(visible_alias = "g")]
    Groups {
        /// filters the results to those with the given parent group id
        #[arg(short = 'g', long)]
        parent_group: Option<i64>,
    },
}

#[derive(Subcommand, Debug)]
enum AddCommand {
    /// add an entry
    #[command(visible_alias = "e")]
    Entry {},

    /// add a group
    #[command(visible_alias = "g")]
    Group {},
}

pub fn run(args: CliArgs) -> Result<(), ()> {
    let show_password = args.show_password;

    let username: Box<str>;
    let password: Box<str>;

    if let Some(path) = args.credentials_file {
        let file_content = std::fs::read_to_string(path).expect("could not read credentials file");
        let mut file_lines = file_content.lines();
        username = file_lines.next().expect("missing username").into();
        password = file_lines.next().expect("missing password").into();
    } else {
        let mut stderr = io::stderr();
        stderr.write_all(b"Username: ").unwrap();
        stderr.flush().unwrap();

        match std::io::stdin().lines().next() {
            Some(line) => username = line.expect("TODO: why can this error ?").into(),
            None => return Err(()), // EOF
        }

        stderr.write_all(b"Password: ").unwrap();
        stderr.flush().unwrap();
        password = read_password(show_password).into();
    }

    let credentials = Credentials::new(username, password);
    let cipher = Cipher::new(&(&credentials).into());

    // TODO: use clap "env" feature
    let db_dir = env::var("DB_DIR").expect("missing env var DB_DIR");
    let db_suffix = env::var("DB_SUFFIX").expect("missing env var DB_SUFFIX");
    let db_path = format!("{}{}{}", db_dir, credentials.user(), db_suffix);

    match args.command {
        Commands::Create => {
            Database::create(db_path, &cipher).expect("could not create database");

            return Ok(());
        }

        Commands::Get { command, separator } => match command {
            GetCommand::Entries {
                parent_group,
                name,
                username,
                password,
                id,
                all,
            } => {
                let db =
                    Database::open(db_path, &cipher).expect("could not open database connection");
                let matched = db.entries(name.as_deref(), parent_group).unwrap();
                let id = id || all;
                let password = password || all;
                let username = username || all;

                for entry in matched {
                    if id {
                        print!("{}{}", entry.id(), separator);
                    }

                    let entry_data = entry.data();
                    print!("{}", entry_data.name());

                    if username {
                        print!("{}{}", separator, entry_data.username());
                    }
                    if password {
                        print!("{}{}", separator, entry_data.password());
                    }
                    println!();
                }

                return Ok(());
            }

            GetCommand::Groups { parent_group } => {
                let db =
                    Database::open(db_path, &cipher).expect("could not open database connection");
                let matched = db.groups(parent_group).unwrap();

                for group in matched {
                    println!("{}{}{}", group.id(), separator, group.data().name());
                }

                return Ok(());
            }
        },

        Commands::Add { command } => {
            match command {
                AddCommand::Entry {} => {
                    let db = Database::open(db_path, &cipher)
                        .expect("could not open database connection");
                    let stdin = std::io::stdin();
                    let mut stdout = std::io::stdout();
                    let mut parent_group = String::new();
                    let mut entry_name = String::new();
                    let mut username = String::new();

                    println!("Adding entry");
                    print!("- parent group id (leave empty for root): ");
                    stdout.flush().ok();
                    // TODO: handle number of bytes read (EOF)
                    let _bytes_read = stdin.read_line(&mut parent_group).unwrap();
                    let parent_group: i64 = parent_group
                        .trim()
                        .parse::<i64>()
                        .or_else(|err| {
                            if let IntErrorKind::Empty = err.kind() {
                                Ok(db.root_group_id())
                            } else {
                                Err(err)
                            }
                        })
                        .unwrap();

                    print!("- entry name: ");
                    stdout.flush().ok();
                    stdin.read_line(&mut entry_name).unwrap();
                    let entry_name = entry_name.trim();
                    // validate entry name
                    if !entry_name
                        .chars()
                        .all(|ch| ch == ' ' || ch.is_ascii_graphic())
                    {
                        todo!("handle invalid input \"{}\"", entry_name);
                    }

                    print!("- entry username: ");
                    stdout.flush().ok();
                    stdin.read_line(&mut username).unwrap();
                    let username = username.trim();
                    // TODO: validate

                    print!("- entry password: ");
                    stdout.flush().ok();
                    let password = read_password(show_password);
                    let password = password.trim();
                    // TODO: validate

                    let entry_data =
                        EntryData::new(entry_name.into(), username.into(), password.into());
                    db.insert_entry(parent_group, &entry_data).unwrap();

                    return Ok(());
                }

                AddCommand::Group {} => {
                    let mut db = Database::open(db_path, &cipher)
                        .expect("could not open database connection");
                    let stdin = std::io::stdin();
                    let mut stdout = std::io::stdout();
                    let mut parent_group = String::new();
                    let mut group_name = String::new();

                    println!("Adding group");
                    print!("- parent group id (leave empty for root): ");
                    stdout.flush().ok();
                    // TODO: handle number of bytes read (EOF)
                    let _bytes_read = stdin.read_line(&mut parent_group).unwrap();
                    let parent_group_id: i64 = parent_group
                        .trim()
                        .parse::<i64>()
                        .or_else(|err| {
                            if let IntErrorKind::Empty = err.kind() {
                                Ok(db.root_group_id())
                            } else {
                                Err(err)
                            }
                        })
                        .unwrap();

                    print!("- group name: ");
                    stdout.flush().ok();
                    stdin.read_line(&mut group_name).unwrap();
                    let group_name = group_name.trim();
                    // validate group name
                    if !group_name
                        .chars()
                        .all(|ch| ch == ' ' || ch.is_ascii_graphic())
                    {
                        todo!("handle invalid input \"{}\"", group_name);
                    }

                    let group_data = EntryGroupData::new(group_name.into());
                    db.insert_entry_group(parent_group_id, &group_data).unwrap();

                    return Ok(());
                }
            }
        }

        Commands::Gen { target } => match target {
            GenTarget::Zsh { cmd_name } => {
                clap_complete::generate(
                    Shell::Zsh,
                    &mut CliArgs::command(),
                    cmd_name.to_str().unwrap(),
                    &mut io::stdout(),
                );

                return Ok(());
            }
        },
    }
}

fn read_password(show_password: bool) -> String {
    if show_password {
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        return password.lines().next().unwrap().into();
    } else {
        return rpassword::read_password().unwrap();
    }
}
