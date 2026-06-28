use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "calenderctl")]
#[command(about = "Administrative tools to control calender service")]
pub struct Cli {
	#[command(subcommand)]
	pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Manage certificate authorities.
	Ca {
		#[command(subcommand)]
		command: CaCommand,
	},
	/// Manage leaf certificates signed by a CA.
	Cert {
		#[command(subcommand)]
		command: CertCommand,
	},
	Passwd {
		/// Password to be encoded
		#[arg(default_value = "password")]
		password: String
	}
}



#[derive(Debug, Subcommand)]
pub enum CaCommand {
	/// Create a local CA certificate and private key.
	Init {
		/// Directory to write CA files into.
		#[arg(long, default_value = "certs")]
		out: PathBuf,
		/// Common name for the CA certificate.
		#[arg(long, default_value = "CalendarKit Local CA")]
		name: String,
		/// Validity period in days.
		#[arg(long, default_value_t = 3650)]
		days: i64,
		/// Overwrite existing output files.
		#[arg(long)]
		force: bool,
	},
}

#[derive(Debug, Subcommand)]
pub enum CertCommand {
	/// Create a server certificate signed by a CA.
	Issue {
		/// CA certificate PEM path.
		#[arg(long, default_value = "certs/ca.pem")]
		ca: PathBuf,
		/// CA private key PEM path.
		#[arg(long, default_value = "certs/ca-key.pem")]
		ca_key: PathBuf,
		/// Directory to write certificate files into.
		#[arg(long, default_value = "certs")]
		out: PathBuf,
		/// Certificate basename.
		#[arg(long, default_value = "server")]
		name: String,
		/// DNS name or IP address for the certificate. Repeatable.
		#[arg(long = "san", required = true)]
		subject_alt_names: Vec<String>,
		/// Validity period in days.
		#[arg(long, default_value_t = 825)]
		days: i64,
		/// Overwrite existing output files.
		#[arg(long)]
		force: bool,
	},
}