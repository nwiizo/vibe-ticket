use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the priority level of a ticket
///
/// Priority helps in task management and scheduling decisions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Low priority - can be deferred
    Low,

    /// Medium priority - normal workflow
    Medium,

    /// High priority - should be addressed soon
    High,

    /// Critical priority - requires immediate attention
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

impl Priority {
    /// Returns all possible priority values
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High, Self::Critical]
    }

    /// Returns the numeric value for sorting (higher = more urgent)
    pub fn value(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Returns the emoji representation of the priority
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Low => "🟢",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::Critical => "🔴",
        }
    }

    /// Returns the color code for terminal output
    pub fn color(&self) -> &'static str {
        match self {
            Self::Low => "green",
            Self::Medium => "yellow",
            Self::High => "magenta",
            Self::Critical => "red",
        }
    }

    /// Returns whether this priority requires immediate attention
    pub fn is_urgent(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

impl TryFrom<&str> for Priority {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "low" | "l" => Ok(Self::Low),
            "medium" | "med" | "m" | "normal" => Ok(Self::Medium),
            "high" | "h" => Ok(Self::High),
            "critical" | "crit" | "c" | "urgent" => Ok(Self::Critical),
            _ => Err(format!("Invalid priority: {}", value)),
        }
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Low,
            2 => Self::Medium,
            3 => Self::High,
            4.. => Self::Critical,
            _ => Self::Medium,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_priority() {
        assert_eq!(Priority::default(), Priority::Medium);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_priority_value() {
        assert_eq!(Priority::Low.value(), 1);
        assert_eq!(Priority::Medium.value(), 2);
        assert_eq!(Priority::High.value(), 3);
        assert_eq!(Priority::Critical.value(), 4);
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!(Priority::try_from("low").unwrap(), Priority::Low);
        assert_eq!(Priority::try_from("L").unwrap(), Priority::Low);
        assert_eq!(Priority::try_from("medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from("normal").unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from("urgent").unwrap(), Priority::Critical);
        assert!(Priority::try_from("invalid").is_err());
    }

    #[test]
    fn test_priority_urgency() {
        assert!(!Priority::Low.is_urgent());
        assert!(!Priority::Medium.is_urgent());
        assert!(Priority::High.is_urgent());
        assert!(Priority::Critical.is_urgent());
    }
}
