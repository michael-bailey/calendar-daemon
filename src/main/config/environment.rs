/// Server configuration loaded from environment variables.
///
/// In production set these via a Kubernetes `Secret` or `.env` file.
/// In tests, use `Config::for_testing()`.

#[derive(Debug, Clone)]
pub struct Environment {
	pub host: Option<String>,
	pub port: Option<u16>,
	/// SQLite connection string, e.g. "sqlite:./data/calendar.db?mode=rwc"
	pub database_url: Option<String>,

	/// Single-user credentials
	pub username: Option<String>,
	/// bcrypt hash of the password
	pub password_hash: Option<String>,
	/// Display name shown in CalDAV clients
	pub display_name: Option<String>,
}

impl Environment {
	pub fn new() -> Self {
		Self {
			host: get_env("HOST"),
			port: get_env("PORT").map(|p| p.parse().unwrap()),
			database_url: get_env("DATABASE_URL"),
			username: get_env("CALDAV_USERNAME"),
			password_hash: get_env("CALDAV_PASSWORD_HASH"),
			display_name: get_env("CALDAV_DISPLAY_NAME"),
		}
	}
}

fn get_env(key: &str) -> Option<String> {
	std::env::var(key).ok()
}
