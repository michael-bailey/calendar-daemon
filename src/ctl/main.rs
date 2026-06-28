pub mod cli;
mod validation;

use std::{
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};
use bcrypt::DEFAULT_COST;
use clap::{Parser, Subcommand};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, Ia5String, IsCa, KeyIdMethod, KeyPair, KeyUsagePurpose, SanType};
use time::{Duration, OffsetDateTime};
use crate::cli::{CaCommand, CertCommand, Cli, Command};

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Passwd {
            password
        } => {
            password_encode(password);
            Ok(())
        },
        Command::Ca { command } => match command {
            CaCommand::Init {
                out,
                name,
                days,
                force,
            } => init_ca(&out, &name, days, force),
        },
        Command::Cert { command } => match command {
            CertCommand::Issue {
                ca,
                ca_key,
                out,
                name,
                subject_alt_names,
                days,
                force,
            } => issue_cert(&ca, &ca_key, &out, &name, &subject_alt_names, days, force),
        },
    }
}

fn password_encode(password: String) {
    let password_hash = bcrypt::hash(password, DEFAULT_COST).expect("Failed to hash password");
    println!("password hash: {}", password_hash);
}

fn init_ca(out: &Path, name: &str, days: i64, force: bool) -> anyhow::Result<()> {
    validate_days(days)?;
    fs::create_dir_all(out)?;

    let cert_path = out.join("ca.pem");
    let key_path = out.join("ca-key.pem");
    ensure_writable(&cert_path, force)?;
    ensure_writable(&key_path, force)?;

    // ...
    let mut params = CertificateParams::default();
    params.distinguished_name = distinguished_name(name);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    params.key_identifier_method = KeyIdMethod::Sha256;  // ← adds SKI
    params.not_before = OffsetDateTime::now_utc();
    params.not_after = params.not_before + Duration::days(days);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    fs::write(&cert_path, cert.pem())?;
    fs::write(&key_path, key_pair.serialize_pem())?;

    println!("wrote {}", cert_path.display());
    println!("wrote {}", key_path.display());
    println!(
        "trust {} in your OS trust store, then use it to verify issued certs",
        cert_path.display()
    );
    Ok(())
}

use std::process::Command as ProcessCommand;

fn issue_cert(
    ca_path: &Path,
    ca_key_path: &Path,
    out: &Path,
    name: &str,
    subject_alt_names: &[String],
    days: i64,
    force: bool,
) -> anyhow::Result<()> {
    validate_days(days)?;
    fs::create_dir_all(out)?;

    let cert_path = out.join(format!("{name}.pem"));
    let key_path = out.join(format!("{name}-key.pem"));
    ensure_writable(&cert_path, force)?;
    ensure_writable(&key_path, force)?;

    let san = subject_alt_names
        .iter()
        .map(|s| {
            if s.parse::<IpAddr>().is_ok() {
                format!("IP:{s}")
            } else {
                format!("DNS:{s}")
            }
        })
        .collect::<Vec<_>>()
        .join(",");

    let csr_path = out.join(format!("{name}.csr"));
    let ext_path = out.join(format!("{name}-ext.cnf"));

    fs::write(&ext_path, format!(
        "[ext]\n\
         subjectAltName = {san}\n\
         keyUsage = critical,digitalSignature,keyEncipherment\n\
         extendedKeyUsage = serverAuth\n\
         subjectKeyIdentifier = hash\n\
         authorityKeyIdentifier = keyid,issuer\n\
         basicConstraints = critical,CA:FALSE\n"
    ))?;

    // Generate key
    run("openssl", &[
        "ecparam", "-name", "prime256v1", "-genkey", "-noout",
        "-out", key_path.to_str().unwrap(),
    ])?;

    // Generate CSR
    run("openssl", &[
        "req", "-new",
        "-key", key_path.to_str().unwrap(),
        "-subj", &format!("/CN={name}"),
        "-out", csr_path.to_str().unwrap(),
    ])?;

    // Sign with CA
    run("openssl", &[
        "x509", "-req",
        "-days", &days.to_string(),
        "-in", csr_path.to_str().unwrap(),
        "-CA", ca_path.to_str().unwrap(),
        "-CAkey", ca_key_path.to_str().unwrap(),
        "-CAcreateserial",
        "-extfile", ext_path.to_str().unwrap(),
        "-extensions", "ext",
        "-out", cert_path.to_str().unwrap(),
    ])?;

    // Clean up temp files
    let _ = fs::remove_file(&csr_path);
    let _ = fs::remove_file(&ext_path);

    println!("wrote {}", cert_path.display());
    println!("wrote {}", key_path.display());
    Ok(())
}

fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = ProcessCommand::new(program).args(args).status()?;
    if !status.success() {
        anyhow::bail!("{program} exited with status {status}");
    }
    Ok(())
}

fn distinguished_name(common_name: &str) -> DistinguishedName {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, common_name);
    distinguished_name
}

fn parse_subject_alt_names(names: &[String]) -> anyhow::Result<Vec<SanType>> {
    names
        .iter()
        .map(|name| {
            if let Ok(ip) = name.parse::<IpAddr>() {
                Ok(SanType::IpAddress(ip))
            } else {
                Ok(SanType::DnsName(name.clone().try_into()?))
            }
        })
        .collect()
}

fn ensure_writable(path: &Path, force: bool) -> anyhow::Result<()> {
    if path.exists() && !force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite",
            path.display()
        );
    }
    Ok(())
}

fn validate_days(days: i64) -> anyhow::Result<()> {
    if days <= 0 {
        anyhow::bail!("--days must be greater than zero");
    }
    Ok(())
}
