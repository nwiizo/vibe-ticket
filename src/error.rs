use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for vibe-ticket
///
/// This enum represents all possible errors that can occur in the application.
/// Using thiserror for automatic Error trait implementation.
#[derive(Error, Debug)]
pub enum VibeTicketError {
    /// I/O related errors
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// YAML serialization/deserialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Git operation errors
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Ticket not found
    #[error("Ticket not found: {id}")]
    TicketNotFound { id: String },

    /// Task not found
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },

    /// Invalid ticket status
    #[error("Invalid ticket status: {status}")]
    InvalidStatus { status: String },

    /// Invalid priority
    #[error("Invalid priority: {priority}")]
    InvalidPriority { priority: String },

    /// Project not initialized
    #[error("Project not initialized. Run 'vibe-ticket init' first")]
    ProjectNotInitialized,

    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Missing required field in template
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    /// No tickets found
    #[error("No tickets found")]
    NoTicketsFound,

    /// Project already initialized
    #[error("Project already initialized at {}", path.display())]
    ProjectAlreadyInitialized { path: PathBuf },

    /// No active ticket
    #[error("No active ticket. Use 'vibe-ticket start <id>' to start working on a ticket")]
    NoActiveTicket,

    /// Multiple active tickets
    #[error("Multiple active tickets found. This should not happen")]
    MultipleActiveTickets,

    /// Invalid slug format
    #[error("Invalid slug format: {slug}. Slugs must be lowercase alphanumeric with hyphens")]
    InvalidSlug { slug: String },

    /// Duplicate ticket
    #[error("Ticket with slug '{slug}' already exists")]
    DuplicateTicket { slug: String },

    /// File operation error
    #[error("File operation failed for {}: {message}", path.display())]
    FileOperation { path: PathBuf, message: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Template error
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    /// Dialoguer error (for interactive mode)
    #[error("Interactive input error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    /// UUID parsing error
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    /// Specification not found
    #[error("Specification not found: {id}")]
    SpecNotFound { id: String },

    /// No active specification
    #[error("No active specification. Use 'vibe-ticket spec activate <id>' to set active spec")]
    NoActiveSpec,

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
    /// Parse error for data formats
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialization error for data formats
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type alias for vibe-ticket operations
pub type Result<T> = std::result::Result<T, VibeTicketError>;

impl VibeTicketError {
    /// Creates a custom error with the given message
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    /// Returns true if this error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::TicketNotFound { .. }
                | Self::TaskNotFound { .. }
                | Self::NoActiveTicket
                | Self::InvalidSlug { .. }
        )
    }

    /// Returns true if this error is a configuration issue
    #[must_use]
    pub const fn is_config_error(&self) -> bool {
        matches!(
            self,
            Self::Config(_) | Self::ProjectNotInitialized | Self::ProjectAlreadyInitialized { .. }
        )
    }

    /// Returns a user-friendly error message
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Io(e) if e.kind() == io::ErrorKind::NotFound => {
                "File or directory not found".to_string()
            },
            Self::Io(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                "Permission denied. Check file permissions".to_string()
            },
            Self::Git(e) => format!("Git operation failed: {}", e.message()),
            _ => self.to_string(),
        }
    }

    /// Creates a serialization error with consistent formatting
    pub fn serialization_error(format: &str, error: impl std::fmt::Display) -> Self {
        Self::custom(format!("Failed to serialize to {format}: {error}"))
    }

    /// Creates a deserialization error with consistent formatting
    pub fn deserialization_error(format: &str, error: impl std::fmt::Display) -> Self {
        Self::custom(format!("Failed to deserialize from {format}: {error}"))
    }

    /// Creates an I/O error with consistent formatting
    pub fn io_error(
        operation: &str,
        path: &std::path::Path,
        error: impl std::fmt::Display,
    ) -> Self {
        Self::custom(format!(
            "Failed to {} {}: {}",
            operation,
            path.display(),
            error
        ))
    }

    /// Creates a parsing error with consistent formatting
    pub fn parse_error(type_name: &str, value: &str, error: impl std::fmt::Display) -> Self {
        Self::custom(format!("Failed to parse '{value}' as {type_name}: {error}"))
    }

    /// Returns suggested actions for the error
    #[must_use]
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::ProjectNotInitialized => vec![
                "Run 'vibe-ticket init' to initialize the project".to_string(),
                "Make sure you're in the correct directory".to_string(),
            ],
            Self::NoActiveTicket => vec![
                "Run 'vibe-ticket list' to see available tickets".to_string(),
                "Run 'vibe-ticket start <id>' to start working on a ticket".to_string(),
            ],
            Self::InvalidSlug { .. } => vec![
                "Use lowercase letters, numbers, and hyphens only".to_string(),
                "Example: 'fix-login-bug' or 'feature-123'".to_string(),
            ],
            Self::DuplicateTicket { slug } => vec![
                format!("Use a different slug or check existing ticket '{}'", slug),
                "Run 'vibe-ticket list' to see all tickets".to_string(),
            ],
            Self::NoActiveSpec => vec![
                "Run 'vibe-ticket spec list' to see available specifications".to_string(),
                "Run 'vibe-ticket spec activate <id>' to set an active specification".to_string(),
            ],
            Self::SpecNotFound { id } => vec![
                format!("Check if specification '{}' exists", id),
                "Run 'vibe-ticket spec list' to see all specifications".to_string(),
            ],
            _ => vec![],
        }
    }
}

/// Error context extension trait
pub trait ErrorContext<T> {
    /// Adds context to the error
    fn context(self, msg: &str) -> Result<T>;

    /// Adds context with a lazy message
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<VibeTicketError>,
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            VibeTicketError::Custom(format!("{msg}: {base_error}"))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            VibeTicketError::Custom(format!("{}: {}", f(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VibeTicketError::TicketNotFound {
            id: "123".to_string(),
        };
        assert_eq!(err.to_string(), "Ticket not found: 123");
    }

    #[test]
    fn test_is_recoverable() {
        assert!(VibeTicketError::NoActiveTicket.is_recoverable());
        assert!(!VibeTicketError::ProjectNotInitialized.is_recoverable());
    }

    #[test]
    fn test_suggestions() {
        let err = VibeTicketError::ProjectNotInitialized;
        let suggestions = err.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("vibe-ticket init"));
    }
}
