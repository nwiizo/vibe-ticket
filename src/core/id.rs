use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Macro to generate ID types backed by UUID
macro_rules! define_id_type {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new random ID
            #[must_use] pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Creates an ID from a UUID
            #[must_use] pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the inner UUID
            #[must_use] pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Parses an ID from a string
            pub fn parse_str(input: &str) -> Result<Self, uuid::Error> {
                Ok(Self(Uuid::parse_str(input)?))
            }

            /// Returns a shortened version of the ID for display
            #[must_use] pub fn short(&self) -> String {
                self.0.to_string()[..8].to_string()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }
    };
}

// Define the ID types
define_id_type! {
    /// Unique identifier for a ticket
    ///
    /// Uses UUID v4 internally for globally unique identification
    TicketId
}

define_id_type! {
    /// Unique identifier for a task
    ///
    /// Uses UUID v4 internally for globally unique identification
    TaskId
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ticket_id_tests {
        use super::*;

        #[test]
        fn test_creation() {
            let id1 = TicketId::new();
            let id2 = TicketId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_parsing() {
            let id = TicketId::new();
            let id_str = id.to_string();
            let parsed = TicketId::parse_str(&id_str).unwrap();
            assert_eq!(id, parsed);
        }

        #[test]
        fn test_short() {
            let id = TicketId::new();
            let short = id.short();
            assert_eq!(short.len(), 8);
            assert!(id.to_string().starts_with(&short));
        }

        #[test]
        fn test_from_uuid() {
            let uuid = Uuid::new_v4();
            let id = TicketId::from_uuid(uuid);
            assert_eq!(id.as_uuid(), &uuid);
        }

        #[test]
        fn test_default() {
            let id1 = TicketId::default();
            let id2 = TicketId::default();
            assert_ne!(id1, id2);
        }
    }

    mod task_id_tests {
        use super::*;

        #[test]
        fn test_creation() {
            let id1 = TaskId::new();
            let id2 = TaskId::new();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_parsing() {
            let id = TaskId::new();
            let id_str = id.to_string();
            let parsed = TaskId::parse_str(&id_str).unwrap();
            assert_eq!(id, parsed);
        }

        #[test]
        fn test_short() {
            let id = TaskId::new();
            let short = id.short();
            assert_eq!(short.len(), 8);
            assert!(id.to_string().starts_with(&short));
        }

        #[test]
        fn test_from_uuid() {
            let uuid = Uuid::new_v4();
            let id = TaskId::from_uuid(uuid);
            assert_eq!(id.as_uuid(), &uuid);
        }

        #[test]
        fn test_default() {
            let id1 = TaskId::default();
            let id2 = TaskId::default();
            assert_ne!(id1, id2);
        }
    }
}
