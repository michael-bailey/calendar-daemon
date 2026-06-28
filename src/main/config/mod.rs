use clap::Parser;
use crate::config::cli::Cli;
use crate::config::environment::Environment;

pub mod cli;
pub mod environment;

/// Server configuration loaded from environment variables.
///
/// In production set these via a Kubernetes `Secret` or `.env` file.
/// In tests, use `Config::for_testing()`.

#[derive(Debug, Clone)]
pub struct Config {
	/// e.g. "https://cal.example.com" — no trailing slash
	pub host: String,
	/// the port to listen on
	pub port: u16,
	/// SQLite connection string, e.g. "sqlite:./data/calendar.db?mode=rwc"
	pub database_url: String,
	/// Single-user credentials
	pub username: String,
	/// bcrypt hash of the password
	pub password_hash: String,
	/// Display name shown in CalDAV clients
	pub display_name: String,
	pub tls_cert_path: String,
	pub tls_key_path: String,
}

impl Config {
	pub fn from_env_cli() -> anyhow::Result<Config> {
		let env = Environment::new();
		let cli = Cli::parse();

		Ok(
			Self {
				host: cli.host.or(env.host).unwrap_or_else(|| "localhost".to_owned()),
				port: cli.port.or(env.port).unwrap_or(443),
				database_url: cli.database_url.or(env.database_url).unwrap_or_else(|| "sqlite::memory:".to_owned()),
				username: cli.username.or(env.username).unwrap_or_else(|| "username".to_owned()),
				password_hash: cli.password_hash.or(env.password_hash).unwrap_or_else(|| DEFAULT_PASSWORD_HASH.to_owned()),
				display_name: cli.display_name.or(env.display_name).unwrap_or_else(|| "Calendar Daemon".to_owned()),
				tls_cert_path: "./certs/server.pem".to_string(),
				tls_key_path: "./certs/server-key.pem".to_string(),
			}
		)
	}

	pub fn get_bind_address(&self) -> String {
		format!("0.0.0.0:{}", self.port)
	}

	pub fn get_base_url(&self) -> String {
		if self.port != 443 { format!("https://{}:{}", self.host, self.port) }
		else { format!("https://{}", self.host) }
	}

	#[cfg(test)]
	pub fn for_testing() -> Self {
		Self {
			host: "localhost".to_string(),
			port: 80,
			database_url: "sqlite::memory:".to_string(),
			username: "Alice".to_string(),
			// bcrypt hash of "password" — never use in production
			password_hash: "$2b$04$KYOzNqb1dsiAu8B6RcFLYuAAbNlypvqIGWLU4QhsSXZCykocyy46K"
					.to_string(),
			display_name: "Alice".to_string(),
			tls_cert_path: "./certs/server.pem".to_string(),
			tls_key_path: "./certs/server-key.pem".to_string(),
		}
	}
}

const DEFAULT_PASSWORD_HASH: &'static str = "$2b$12$y7CMALk1O/sUPuU15pp/0eNxkzikxgsvRfx/7xfNmzN3Egp5JTN3m";