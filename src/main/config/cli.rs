use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "caldav-server")]
#[command(about = "A single-user CalDAV server targeting macOS Calendar")]
pub struct Cli {

	#[arg(long)]
	pub host: Option<String>,

	#[arg(long)]
	pub port: Option<u16>,

	#[arg(long)]
	pub enable_tls: Option<bool>,

	#[arg(long)]
	pub database_url: Option<String>,

	#[arg(long, short = 'u')]
	pub username: Option<String>,

	#[arg(long, short = 'p')]
	pub password_hash: Option<String>,

	#[arg(long)]
	pub display_name: Option<String>,
}