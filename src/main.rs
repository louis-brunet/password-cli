use std::{env, io::Write, path::PathBuf};

use clap::{Parser, Subcommand};
use password_cli::{model::{database::Database, EntryData}, encryption::Cipher, Credentials};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,

    /// credentials file containing the username and password
    #[arg(short, long)]
    credentials_file: Option<PathBuf>,
    // user: String,
    //
    // #[arg(short, long)]
    // key_file: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// does testing things
    Test {
        /// echo the given string
        #[arg(short, long)]
        echo: Option<String>,
    },

    /// retrieve entry information
    Get {
        // #[arg(short = 'g', long)]
        // parent_group: Option<String>,
        //
        // #[arg(short = 't', long = "type", name = "TYPE", value_enum, default_value_t = GetTarget::Entry)]
        // target_type: GetTarget,
        //
        /// if given, filters the results to those matching the given name
        #[arg(short, long)]
        name: Option<String>,

        /// get the username from each entry
        #[arg(short, long)]
        username: bool,

        /// get the password from each entry
        #[arg(short, long)]
        password: bool,
    },

    /// add entries
    Add {
        // #[arg(short, long)]
        // name: Option<String>,
        //
        // #[arg(short, long)]
        // username: Option<String>,

        // TODO prompt for password with rpassword crate
        // #[arg(short, long)]
        // password: String,
    },

    /// open the password manager in interactive mode
    Open,
}
//
// #[derive(ValueEnum, Clone, Debug)]
// enum GetTarget {
//     Entry,
//     Group,
// }

fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();
    log::trace!("initialized logger");

    let cli = CliArgs::parse();
    let username: Box<str>;
    let password: Box<str>;

    if let Some(path) = cli.credentials_file {
        let file_content = std::fs::read_to_string(path).expect("could not read credentials file");
        let mut file_lines = file_content.lines();
        username = file_lines.next().expect("missing username").into();
        password = file_lines.next().expect("missing password").into();
    } else {
        print!("Username: ");
        std::io::stdout().flush().unwrap();

        match std::io::stdin().lines().next() {
            Some(line) => username = line.expect("TODO: why can this error ?").into(),
            None => return, // EOF
        }

        password = rpassword::prompt_password("Password: ").unwrap().into();
    }

    let credentials = Credentials::new(username, password);
    let key_input: [&[u8]; 2] = (&credentials).into();
    let cipher = Cipher::new(&key_input);

    let db_dir = env::var("DB_DIR").expect("missing env var DB_DIR");
    let db_suffix = env::var("DB_SUFFIX").expect("missing env var DB_SUFFIX");
    let db_path = format!("{}{}{}", db_dir, credentials.user(), db_suffix);

    let db = Database::open(db_path, &cipher).expect("could not open database connection");
    log::trace!("initialized db");
    // db.init_test_tables();

    match cli.command {
        Commands::Test { echo } => {
            println!("{}", echo.unwrap_or("".to_string()));
            println!();


            let root_group_id = db.root_group_id();
            let entry = EntryData::new(
                "inserted entry".into(),
                "username".into(),
                "password".into(),
            );
            let entry_id = db.insert_entry(root_group_id, &entry).unwrap();
            let inserted_entry = db.entry(entry_id);
            log::trace!(
                "inserted entry: {:?}",
                inserted_entry
            );

            println!("\nAll entries:\n\t{:?}", db.entries(None));
            println!("\nFiltered entries (by name={:?}):\n\t{:?}", "ted", db.entries(Some("ted")));

            // encryption::test();
        }

        Commands::Get {
            name,
            username,
            password,
        } => {
            let matched = db.entries(name.as_deref()).unwrap();

            // let manager =
            //     PasswordManager::open(credentials).expect("could not open with given credentials");
            // let matched = manager.entries(name.as_deref());

            for entry in matched {
                let entry_data = entry.data();
                print!("{}", entry_data.name());
                if username {
                    print!("\t{}", entry_data.username());
                }
                if password {
                    print!("\t{}", entry_data.password());
                }
                println!();
            }
        }

        Commands::Add {
        } => todo!(),

        // Commands::List { target: ListTarget::Groups } => todo!("list groups"),
        // Commands::List { target: ListTarget::Entries } => todo!("list entries"),
        Commands::Open => {
            todo!()
            // interactive(credentials);
        }
    }
}

// fn interactive(credentials: Credentials) {
//     println!("Opening password manager");
//
//     let manager = PasswordManager::open(credentials).expect("could not open password manager");
//
//     let stdin = std::io::stdin();
//     let mut stdout = std::io::stdout();
//     let mut input_buf = String::new();
//     let mut quit = false;
//     let mut working_group = manager.root();
//
//     while !quit {
//         println!();
//         println!("===========================");
//         print_entries(working_group);
//         print_groups(working_group);
//         println!("===========================");
//         println!();
//         println!("What to do ?");
//         println!(" [1] see entry contents");
//         println!(" [2] cd");
//         println!(" [q] quit");
//
//         print!("\n[1/2/q] ");
//         let _ = stdout.flush();
//         input_buf.clear();
//         let _ = stdin.read_line(&mut input_buf);
//
//         match input_buf.chars().next() {
//             Some('1') => {
//                 let entries = working_group.entries();
//                 let entry_count = entries.len();
//                 if entry_count > 0 {
//                     let mut entry_index = None;
//
//                     while entry_index.is_none() && !quit {
//                         print_entries(working_group);
//                         print!("which entry ? [{}-{}] ", 1, entry_count);
//                         let _ = stdout.flush();
//                         input_buf.clear();
//                         let _ = stdin.read_line(&mut input_buf);
//
//                         match input_buf.trim().parse::<usize>() {
//                             // input should be valid 1-based index in entries array
//                             Ok(num) if num == 0 || num > entries.len() => {
//                                 println!("{} is out of bounds", num);
//                             }
//                             Ok(entry_num) => {
//                                 entry_index = Some(entry_num - 1);
//                             }
//                             Err(err) => {
//                                 if let IntErrorKind::Empty = err.kind() {
//                                     quit = true;
//                                 } else {
//                                     println!("Could not parse entry number: {}", err);
//                                 }
//                             }
//                         }
//                     }
//                     if let Some(idx) = entry_index {
//                         let entry = &entries[idx];
//                         println!("{:?}", entry);
//                     }
//                 } else {
//                     println!("No entries in working group!");
//                 }
//             }
//             Some('2') => {
//                 todo!()
//             }
//             Some('q') => quit = true,
//             Some('\r' | '\n') => (),
//             Some(ch) => println!("unrecognized option: '{}'", ch),
//             None => quit = true,
//         }
//     }
// }

// fn print_groups(group: &Group) {
//     let groups = group.groups();
//
//     // if groups.is_empty() {
//     //     println!("Group '{}' has no subgroups", group.name());
//     // } else {
//     println!("Subgroups in group '{}':", group.name());
//     for (idx, group) in groups.iter().enumerate() {
//         println!(" {:2} - {}", idx + 1, group.name());
//     }
//     // }
// }
//
// fn print_entries(group: &Group) {
//     let entries = group.entries();
//
//     // if entries.is_empty() {
//     //     println!("Group '{}' has no entries", group.name());
//     // } else {
//     println!("Entries in group '{}':", group.name());
//
//     for (idx, entry) in entries.iter().enumerate() {
//         println!(" {:2} - {}", idx + 1, entry.name());
//     }
//     // }
// }
