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

/// Visual and semantic properties for a priority
struct PriorityProperties {
    display: &'static str,
    value: u8,
    emoji: &'static str,
    color: &'static str,
}

impl Priority {
    /// Returns all properties for this priority
    const fn properties(&self) -> PriorityProperties {
        match self {
            Self::Low => PriorityProperties {
                display: "Low",
                value: 1,
                emoji: "🟢",
                color: "green",
            },
            Self::Medium => PriorityProperties {
                display: "Medium",
                value: 2,
                emoji: "🟡",
                color: "yellow",
            },
            Self::High => PriorityProperties {
                display: "High",
                value: 3,
                emoji: "🟠",
                color: "magenta",
            },
            Self::Critical => PriorityProperties {
                display: "Critical",
                value: 4,
                emoji: "🔴",
                color: "red",
            },
        }
    }

    /// Returns all possible priority values
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High, Self::Critical]
    }

    /// Returns the numeric value for sorting (higher = more urgent)
    #[must_use]
    pub const fn value(&self) -> u8 {
        self.properties().value
    }

    /// Returns the emoji representation of the priority
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        self.properties().emoji
    }

    /// Returns the color code for terminal output
    #[must_use]
    pub const fn color(&self) -> &'static str {
        self.properties().color
    }

    /// Returns whether this priority requires immediate attention
    #[must_use]
    pub const fn is_urgent(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.properties().display)
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
            _ => Err(format!("Invalid priority: {value}")),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s).map_err(|_| format!("Invalid priority: {s}"))
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
    fn test_priority_display() {
        assert_eq!(Priority::Low.to_string(), "Low");
        assert_eq!(Priority::Medium.to_string(), "Medium");
        assert_eq!(Priority::High.to_string(), "High");
        assert_eq!(Priority::Critical.to_string(), "Critical");
    }

    #[test]
    fn test_priority_emoji() {
        assert_eq!(Priority::Low.emoji(), "🟢");
        assert_eq!(Priority::Medium.emoji(), "🟡");
        assert_eq!(Priority::High.emoji(), "🟠");
        assert_eq!(Priority::Critical.emoji(), "🔴");
    }

    #[test]
    fn test_priority_color() {
        assert_eq!(Priority::Low.color(), "green");
        assert_eq!(Priority::Medium.color(), "yellow");
        assert_eq!(Priority::High.color(), "magenta");
        assert_eq!(Priority::Critical.color(), "red");
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
    fn test_priority_from_u8() {
        assert_eq!(Priority::from(1), Priority::Low);
        assert_eq!(Priority::from(2), Priority::Medium);
        assert_eq!(Priority::from(3), Priority::High);
        assert_eq!(Priority::from(4), Priority::Critical);
        assert_eq!(Priority::from(5), Priority::Critical);
        assert_eq!(Priority::from(0), Priority::Medium);
    }

    #[test]
    fn test_priority_urgency() {
        assert!(!Priority::Low.is_urgent());
        assert!(!Priority::Medium.is_urgent());
        assert!(Priority::High.is_urgent());
        assert!(Priority::Critical.is_urgent());
    }

    #[test]
    fn test_all_priorities() {
        let all = Priority::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], Priority::Low);
        assert_eq!(all[1], Priority::Medium);
        assert_eq!(all[2], Priority::High);
        assert_eq!(all[3], Priority::Critical);
    }
}
