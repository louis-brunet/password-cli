use clap::Parser;
use password_cli::cli::{self, CliArgs};

fn main() {
    if dotenv::dotenv().is_err() {
        // could not load env vars, i.e. no logging
        // TODO: use clap 'env' feature to validate environmemnt variables later

        // log::error!("error: could not load environment variables");
        // eprintln!("error: could not load environment variables");
        // return;
    }
    env_logger::init();
    log::trace!("initialized environment variables and logger");

    let cli_args = CliArgs::parse();
    log::trace!("finished parsing CLI arguments");

    cli::run(cli_args).expect("password-cli error");
}
