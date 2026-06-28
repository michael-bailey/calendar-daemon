use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "caldav-server")]
#[command(about = "A single-user CalDAV server targeting macOS Calendar")]
pub struct Cli {
    #[command(subcommand)]
		pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Hash a password for CALDAV_PASSWORD_HASH.
    Passwd {
        /// Plaintext password to hash.
        password: String,
    },
}