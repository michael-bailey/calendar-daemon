use time::OffsetDateTime;

/// A calendar collection — maps to a WebDAV collection resource.
#[derive(Debug, Clone)]
pub struct Calendar {
    /// Stable unique identifier (used in URLs)
    pub id: String,
    /// Display name shown in calendar clients
    pub display_name: String,
    /// Optional colour hint (e.g. "#3a87ad")
    pub color: Option<String>,
    /// Description shown in calendar clients
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Calendar {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: id.into(),
            display_name: display_name.into(),
            color: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}
