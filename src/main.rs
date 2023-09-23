use clap::Parser;
use password_cli::cli::{self, CliArgs};

fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();
    log::trace!("initialized environment variables and logger");

    let cli_args = CliArgs::parse();
    log::trace!("finished parsing CLI arguments");

    cli::run(cli_args).expect("password-cli error");
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
