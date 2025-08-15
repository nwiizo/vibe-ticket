use crate::core::{Priority, Status, Ticket};
use crate::error::{Result, VibeTicketError};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};

/// Common date filtering utilities
pub struct DateFilter;

impl DateFilter {
    /// Parse a date filter string
    pub fn parse(filter: &str) -> Result<DateRange> {
        let filter = filter.trim();
        
        // Handle relative dates
        if filter == "today" {
            let today = Local::now().date_naive();
            return Ok(DateRange::Day(today));
        }
        
        if filter == "yesterday" {
            let yesterday = Local::now().date_naive() - Duration::days(1);
            return Ok(DateRange::Day(yesterday));
        }
        
        if filter == "week" || filter == "this-week" {
            let now = Local::now();
            let start_of_week = now.date_naive() - Duration::days(now.weekday().num_days_from_monday() as i64);
            let end_of_week = start_of_week + Duration::days(6);
            return Ok(DateRange::Range(start_of_week, end_of_week));
        }
        
        if filter == "month" || filter == "this-month" {
            let now = Local::now().naive_local();
            let start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .ok_or_else(|| VibeTicketError::InvalidInput("Invalid date".to_string()))?;
            let end = if now.month() == 12 {
                NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
            }
            .ok_or_else(|| VibeTicketError::InvalidInput("Invalid date".to_string()))?
            - Duration::days(1);
            return Ok(DateRange::Range(start, end));
        }
        
        // Handle "last N days" format
        if let Some(days_str) = filter.strip_prefix("last-") {
            if let Ok(days) = days_str.parse::<i64>() {
                let end = Local::now().date_naive();
                let start = end - Duration::days(days - 1);
                return Ok(DateRange::Range(start, end));
            }
        }
        
        // Try to parse as specific date
        if let Ok(date) = NaiveDate::parse_from_str(filter, "%Y-%m-%d") {
            return Ok(DateRange::Day(date));
        }
        
        // Try to parse as date range
        if let Some((start_str, end_str)) = filter.split_once("..") {
            let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
                .map_err(|_| VibeTicketError::InvalidInput(format!("Invalid start date: {}", start_str)))?;
            let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
                .map_err(|_| VibeTicketError::InvalidInput(format!("Invalid end date: {}", end_str)))?;
            return Ok(DateRange::Range(start, end));
        }
        
        Err(VibeTicketError::InvalidInput(format!(
            "Invalid date filter: '{}'. Use formats like 'today', 'yesterday', 'week', 'month', 'last-7', '2024-01-15', or '2024-01-01..2024-01-31'",
            filter
        )))
    }
}

/// Date range for filtering
#[derive(Debug, Clone)]
pub enum DateRange {
    Day(NaiveDate),
    Range(NaiveDate, NaiveDate),
}

impl DateRange {
    /// Check if a datetime falls within this range
    pub fn contains(&self, datetime: &DateTime<Utc>) -> bool {
        let date = datetime.naive_local().date();
        match self {
            DateRange::Day(day) => date == *day,
            DateRange::Range(start, end) => date >= *start && date <= *end,
        }
    }
}

/// Common ticket filtering logic
pub struct TicketFilter {
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub assignee: Option<String>,
    pub tags: Vec<String>,
    pub open_only: bool,
    pub closed_only: bool,
    pub has_tasks: Option<bool>,
    pub created_after: Option<DateRange>,
    pub updated_after: Option<DateRange>,
    pub closed_after: Option<DateRange>,
    pub sort_by: SortBy,
    pub reverse: bool,
    pub limit: Option<usize>,
}

/// Sort options for tickets
#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    Created,
    Updated,
    Priority,
    Status,
    Title,
}

impl Default for TicketFilter {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            assignee: None,
            tags: Vec::new(),
            open_only: false,
            closed_only: false,
            has_tasks: None,
            created_after: None,
            updated_after: None,
            closed_after: None,
            sort_by: SortBy::Created,
            reverse: false,
            limit: None,
        }
    }
}

impl TicketFilter {
    /// Apply all filters to a list of tickets
    pub fn apply(self, mut tickets: Vec<Ticket>) -> Vec<Ticket> {
        // Filter tickets
        let filtered: Vec<Ticket> = tickets
            .into_iter()
            .filter(|ticket| self.matches(ticket))
            .collect();
        
        // Sort tickets
        let mut sorted = self.sort(filtered);
        
        // Apply limit if specified
        if let Some(limit) = self.limit {
            sorted.truncate(limit);
        }
        
        sorted
    }
    
    /// Check if a ticket matches all filter criteria
    fn matches(&self, ticket: &Ticket) -> bool {
        // Status filter
        if let Some(ref status) = self.status {
            if ticket.status != *status {
                return false;
            }
        }
        
        // Priority filter
        if let Some(ref priority) = self.priority {
            if ticket.priority != *priority {
                return false;
            }
        }
        
        // Assignee filter
        if let Some(ref assignee) = self.assignee {
            if ticket.assignee.as_ref() != Some(assignee) {
                return false;
            }
        }
        
        // Tags filter
        if !self.tags.is_empty() {
            if !self.tags.iter().all(|tag| ticket.tags.contains(tag)) {
                return false;
            }
        }
        
        // Open/closed filters
        if self.open_only && ticket.status == Status::Done {
            return false;
        }
        if self.closed_only && ticket.status != Status::Done {
            return false;
        }
        
        // Task filter
        if let Some(has_tasks) = self.has_tasks {
            if has_tasks && ticket.tasks.is_empty() {
                return false;
            }
            if !has_tasks && !ticket.tasks.is_empty() {
                return false;
            }
        }
        
        // Date filters
        if let Some(ref range) = self.created_after {
            if !range.contains(&ticket.created_at) {
                return false;
            }
        }
        
        if let Some(ref range) = self.updated_after {
            if !range.contains(&ticket.updated_at) {
                return false;
            }
        }
        
        if let Some(ref range) = self.closed_after {
            if let Some(closed_at) = ticket.closed_at {
                if !range.contains(&closed_at) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    /// Sort tickets according to sort criteria
    fn sort(&self, mut tickets: Vec<Ticket>) -> Vec<Ticket> {
        tickets.sort_by(|a, b| {
            let ordering = match self.sort_by {
                SortBy::Created => a.created_at.cmp(&b.created_at),
                SortBy::Updated => a.updated_at.cmp(&b.updated_at),
                SortBy::Priority => b.priority.cmp(&a.priority), // Higher priority first
                SortBy::Status => a.status.cmp(&b.status),
                SortBy::Title => a.title.cmp(&b.title),
            };
            
            if self.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
        
        tickets
    }
}