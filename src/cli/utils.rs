use chrono::Utc;
use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Result, VibeTicketError};

/// Gets the project root directory
///
/// This function searches for a .vibe-ticket directory in the current directory
/// and its parents, similar to how Git finds the repository root.
pub fn find_project_root(start_dir: Option<&str>) -> Result<PathBuf> {
    let start = if let Some(dir) = start_dir {
        PathBuf::from(dir)
    } else {
        env::current_dir().map_err(VibeTicketError::Io)?
    };

    let mut current = start.as_path();

    loop {
        let vibe_ticket_dir = current.join(".vibe-ticket");
        if vibe_ticket_dir.exists() && vibe_ticket_dir.is_dir() {
            return Ok(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    Err(VibeTicketError::ProjectNotInitialized)
}

/// Gets the .vibe-ticket directory path
#[must_use]
pub fn get_vibe_ticket_dir(project_root: &Path) -> PathBuf {
    project_root.join(".vibe-ticket")
}

/// Generate a slug from a title
///
/// Creates a URL-safe slug from the given title string
pub fn generate_slug(title: &str) -> String {
    // Get current date for prefix
    let date_prefix = Utc::now().format("%Y%m%d%H%M").to_string();

    // Clean and convert title to slug
    let title_slug = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Truncate if too long
    let max_title_len = 50;
    let truncated = if title_slug.len() > max_title_len {
        &title_slug[..max_title_len]
    } else {
        &title_slug
    };

    format!("{}-{}", date_prefix, truncated)
}

/// Validates a ticket slug
///
/// Slugs must be lowercase alphanumeric with hyphens
pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        return Err(VibeTicketError::InvalidSlug {
            slug: slug.to_string(),
        });
    }

    let valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !valid || slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err(VibeTicketError::InvalidSlug {
            slug: slug.to_string(),
        });
    }

    Ok(())
}

/// Generates a slug from a title
#[must_use]
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Gets the default editor from environment variables
#[must_use]
pub fn get_editor() -> String {
    env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        })
}

/// Parses comma-separated tags
pub fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string)
        .collect()
}

/// Formats duration in a human-readable way
#[must_use]
pub fn format_duration(duration: chrono::Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

/// Opens a URL in the default browser
pub fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(VibeTicketError::Io)?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(VibeTicketError::Io)?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .map_err(VibeTicketError::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("fix-login-bug").is_ok());
        assert!(validate_slug("feature-123").is_ok());
        assert!(validate_slug("test").is_ok());

        assert!(validate_slug("").is_err());
        assert!(validate_slug("Fix-Login").is_err()); // uppercase
        assert!(validate_slug("-start").is_err()); // starts with hyphen
        assert!(validate_slug("end-").is_err()); // ends with hyphen
        assert!(validate_slug("double--hyphen").is_err()); // double hyphen
        assert!(validate_slug("special@char").is_err()); // special char
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Fix Login Bug"), "fix-login-bug");
        assert_eq!(slugify("Feature #123"), "feature-123");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special@#$Characters"), "special-characters");
        assert_eq!(slugify("underscore_test"), "underscore-test");
    }

    #[test]
    fn test_parse_tags() {
        assert_eq!(
            parse_tags("bug, frontend, urgent"),
            vec!["bug", "frontend", "urgent"]
        );
        assert_eq!(parse_tags("single"), vec!["single"]);
        assert_eq!(parse_tags(""), Vec::<String>::new());
        assert_eq!(parse_tags("  tag1  ,  tag2  "), vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_format_duration() {
        use chrono::Duration;

        assert_eq!(format_duration(Duration::minutes(45)), "45m");
        assert_eq!(
            format_duration(Duration::hours(2) + Duration::minutes(30)),
            "2h 30m"
        );
        assert_eq!(
            format_duration(Duration::days(3) + Duration::hours(5)),
            "3d 5h"
        );
    }
}
