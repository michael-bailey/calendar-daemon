use anyhow::anyhow;

pub fn validate_san(san: &str) -> anyhow::Result<(), String> {
	if san.is_empty() { return Err("San is invalid".to_string()); }
	Ok(())
}