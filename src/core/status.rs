use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the current status of a ticket
///
/// The status follows a typical workflow progression from
/// Todo → Doing → Done, with additional states for special cases.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Ticket is created but work hasn't started
    Todo,

    /// Work is actively being done on the ticket
    Doing,

    /// Work on the ticket is completed
    Done,

    /// Ticket is blocked by external dependencies
    Blocked,

    /// Ticket is in review/QA phase
    Review,
}

/// Visual properties for a status
struct StatusVisual {
    display: &'static str,
    emoji: &'static str,
    color: &'static str,
}

impl Status {
    /// Returns the visual properties for this status
    const fn visual(&self) -> StatusVisual {
        match self {
            Self::Todo => StatusVisual {
                display: "Todo",
                emoji: "📋",
                color: "blue",
            },
            Self::Doing => StatusVisual {
                display: "Doing",
                emoji: "🔧",
                color: "yellow",
            },
            Self::Done => StatusVisual {
                display: "Done",
                emoji: "✅",
                color: "green",
            },
            Self::Blocked => StatusVisual {
                display: "Blocked",
                emoji: "🚫",
                color: "red",
            },
            Self::Review => StatusVisual {
                display: "Review",
                emoji: "👀",
                color: "cyan",
            },
        }
    }

    /// Returns all possible status values
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Todo,
            Self::Doing,
            Self::Done,
            Self::Blocked,
            Self::Review,
        ]
    }

    /// Returns whether the status represents active work
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Doing | Self::Review)
    }

    /// Returns whether the status represents completed work
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        matches!(self, Self::Done)
    }

    /// Returns whether the status allows starting work
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(self, Self::Todo | Self::Blocked)
    }

    /// Returns the emoji representation of the status
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        self.visual().emoji
    }

    /// Returns the color code for terminal output
    #[must_use]
    pub const fn color(&self) -> &'static str {
        self.visual().color
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::Todo
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.visual().display)
    }
}

impl TryFrom<&str> for Status {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "todo" => Ok(Self::Todo),
            "doing" | "in-progress" | "wip" => Ok(Self::Doing),
            "done" | "completed" | "closed" => Ok(Self::Done),
            "blocked" => Ok(Self::Blocked),
            "review" | "reviewing" => Ok(Self::Review),
            _ => Err(format!("Invalid status: {value}")),
        }
    }
}
impl std::str::FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s).map_err(|_| format!("Invalid status: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_status() {
        assert_eq!(Status::default(), Status::Todo);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(Status::Todo.to_string(), "Todo");
        assert_eq!(Status::Doing.to_string(), "Doing");
        assert_eq!(Status::Done.to_string(), "Done");
        assert_eq!(Status::Blocked.to_string(), "Blocked");
        assert_eq!(Status::Review.to_string(), "Review");
    }

    #[test]
    fn test_status_emoji() {
        assert_eq!(Status::Todo.emoji(), "📋");
        assert_eq!(Status::Doing.emoji(), "🔧");
        assert_eq!(Status::Done.emoji(), "✅");
        assert_eq!(Status::Blocked.emoji(), "🚫");
        assert_eq!(Status::Review.emoji(), "👀");
    }

    #[test]
    fn test_status_color() {
        assert_eq!(Status::Todo.color(), "blue");
        assert_eq!(Status::Doing.color(), "yellow");
        assert_eq!(Status::Done.color(), "green");
        assert_eq!(Status::Blocked.color(), "red");
        assert_eq!(Status::Review.color(), "cyan");
    }

    #[test]
    fn test_status_properties() {
        assert!(Status::Doing.is_active());
        assert!(Status::Review.is_active());
        assert!(!Status::Todo.is_active());

        assert!(Status::Done.is_completed());
        assert!(!Status::Doing.is_completed());

        assert!(Status::Todo.can_start());
        assert!(Status::Blocked.can_start());
        assert!(!Status::Doing.can_start());
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(Status::try_from("todo").unwrap(), Status::Todo);
        assert_eq!(Status::try_from("DOING").unwrap(), Status::Doing);
        assert_eq!(Status::try_from("in-progress").unwrap(), Status::Doing);
        assert_eq!(Status::try_from("completed").unwrap(), Status::Done);
        assert!(Status::try_from("invalid").is_err());
    }

    #[test]
    fn test_all_statuses() {
        let all = Status::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&Status::Todo));
        assert!(all.contains(&Status::Doing));
        assert!(all.contains(&Status::Done));
        assert!(all.contains(&Status::Blocked));
        assert!(all.contains(&Status::Review));
    }
}
