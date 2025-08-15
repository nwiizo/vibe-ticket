use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::core::{Ticket, TicketId};

/// A cached ticket entry with timestamp
#[derive(Clone)]
struct CachedEntry<T> {
    data: T,
    timestamp: Instant,
}

/// Thread-safe in-memory cache for tickets
pub struct TicketCache {
    /// Cache storage
    cache: Arc<RwLock<HashMap<CacheKey, CachedEntry<CacheValue>>>>,
    /// Time-to-live for cached entries
    ttl: Duration,
}

/// Keys used in the cache
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum CacheKey {
    /// Single ticket by ID
    Ticket(TicketId),
    /// All tickets
    AllTickets,
}

/// Values stored in the cache
#[derive(Clone)]
enum CacheValue {
    /// Single ticket
    Ticket(Box<Ticket>),
    /// Multiple tickets
    Tickets(Vec<Ticket>),
}

impl TicketCache {
    /// Creates a new cache with the specified TTL
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Creates a new cache with a default TTL of 5 minutes
    #[must_use]
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(300))
    }

    /// Gets a single ticket from cache
    #[must_use]
    pub fn get_ticket(&self, id: &TicketId) -> Option<Ticket> {
        let entry = self.cache.read().ok()?.get(&CacheKey::Ticket(id.clone()))?.clone();

        if self.is_expired(&entry.timestamp) {
            return None;
        }

        match &entry.data {
            CacheValue::Ticket(ticket) => Some((**ticket).clone()),
            CacheValue::Tickets(_) => None,
        }
    }

    /// Caches a single ticket
    pub fn cache_ticket(&self, ticket: &Ticket) {
        if let Ok(mut cache) = self.cache.write() {
            let entry = CachedEntry {
                data: CacheValue::Ticket(Box::new(ticket.clone())),
                timestamp: Instant::now(),
            };
            cache.insert(CacheKey::Ticket(ticket.id.clone()), entry);
        }
    }

    /// Gets all tickets from cache
    #[must_use]
    pub fn get_all_tickets(&self) -> Option<Vec<Ticket>> {
        let entry = self.cache.read().ok()?.get(&CacheKey::AllTickets)?.clone();

        if self.is_expired(&entry.timestamp) {
            return None;
        }

        match &entry.data {
            CacheValue::Tickets(tickets) => Some(tickets.clone()),
            CacheValue::Ticket(_) => None,
        }
    }

    /// Caches all tickets
    pub fn cache_all_tickets(&self, tickets: &[Ticket]) {
        if let Ok(mut cache) = self.cache.write() {
            let entry = CachedEntry {
                data: CacheValue::Tickets(tickets.to_vec()),
                timestamp: Instant::now(),
            };
            cache.insert(CacheKey::AllTickets, entry);

            // Also cache individual tickets
            for ticket in tickets {
                let ticket_entry = CachedEntry {
                    data: CacheValue::Ticket(Box::new(ticket.clone())),
                    timestamp: Instant::now(),
                };
                cache.insert(CacheKey::Ticket(ticket.id.clone()), ticket_entry);
            }
        }
    }

    /// Invalidates a specific ticket in the cache
    pub fn invalidate_ticket(&self, id: &TicketId) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&CacheKey::Ticket(id.clone()));
            // Also invalidate all tickets cache since it might contain outdated data
            cache.remove(&CacheKey::AllTickets);
        }
    }

    /// Invalidates all cached data
    pub fn invalidate_all(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Checks if a timestamp has expired
    fn is_expired(&self, timestamp: &Instant) -> bool {
        timestamp.elapsed() > self.ttl
    }

    /// Removes expired entries from the cache
    pub fn cleanup_expired(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| !self.is_expired(&entry.timestamp));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Priority, Status};
    use std::thread;

    fn create_test_ticket(suffix: &str) -> Ticket {
        let id = format!("12345678-1234-1234-1234-{suffix:0>12}");
        Ticket {
            id: TicketId::parse_str(&id).unwrap(),
            slug: format!("test-{suffix}"),
            title: format!("Test Ticket {suffix}"),
            description: String::new(),
            priority: Priority::Medium,
            status: Status::Todo,
            tags: vec![],
            created_at: chrono::Utc::now(),
            started_at: None,
            closed_at: None,
            assignee: None,
            tasks: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_cache_and_retrieve_ticket() {
        let cache = TicketCache::new(Duration::from_secs(60));
        let ticket = create_test_ticket("1");

        // Cache the ticket
        cache.cache_ticket(&ticket);

        // Retrieve it
        let cached = cache.get_ticket(&ticket.id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, ticket.id);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = TicketCache::new(Duration::from_millis(100));
        let ticket = create_test_ticket("2");

        cache.cache_ticket(&ticket);

        // Should exist immediately
        assert!(cache.get_ticket(&ticket.id).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert!(cache.get_ticket(&ticket.id).is_none());
    }

    #[test]
    fn test_cache_all_tickets() {
        let cache = TicketCache::new(Duration::from_secs(60));
        let tickets = vec![
            create_test_ticket("3"),
            create_test_ticket("4"),
            create_test_ticket("5"),
        ];

        cache.cache_all_tickets(&tickets);

        // Check all tickets cache
        let cached_all = cache.get_all_tickets();
        assert!(cached_all.is_some());
        assert_eq!(cached_all.unwrap().len(), 3);

        // Check individual tickets are also cached
        assert!(cache.get_ticket(&tickets[0].id).is_some());
        assert!(cache.get_ticket(&tickets[1].id).is_some());
        assert!(cache.get_ticket(&tickets[2].id).is_some());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = TicketCache::new(Duration::from_secs(60));
        let ticket = create_test_ticket("6");

        cache.cache_ticket(&ticket);
        assert!(cache.get_ticket(&ticket.id).is_some());

        cache.invalidate_ticket(&ticket.id);
        assert!(cache.get_ticket(&ticket.id).is_none());
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = TicketCache::new(Duration::from_millis(100));
        let ticket1 = create_test_ticket("7");
        let ticket2 = create_test_ticket("8");

        cache.cache_ticket(&ticket1);
        thread::sleep(Duration::from_millis(150));
        cache.cache_ticket(&ticket2);

        // Before cleanup, expired ticket might still be in cache
        cache.cleanup_expired();

        // After cleanup, only non-expired ticket should remain
        assert!(cache.get_ticket(&ticket1.id).is_none());
        assert!(cache.get_ticket(&ticket2.id).is_some());
    }
}
