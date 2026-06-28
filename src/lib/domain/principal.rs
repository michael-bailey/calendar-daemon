/// The authenticated user / principal.
///
/// For a single-user server this is effectively a constant, but modelling
/// it as a type keeps the door open for multi-user later.
#[derive(Debug, Clone)]
pub struct Principal {
    /// URL path segment, e.g. "alice"
    pub username: String,
    /// Display name returned in PROPFIND responses
    pub display_name: String,
    /// bcrypt hash of the password
    pub password_hash: String,
}

impl Principal {
    pub fn new(
        username: impl Into<String>,
        display_name: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            display_name: display_name.into(),
            password_hash: password_hash.into(),
        }
    }
}
