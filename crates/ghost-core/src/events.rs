//! Event system for Ghost API
//!
//! This module defines events emitted by the Ghost engine
//! for monitoring and observability.

pub use ghost_schema::GhostEvent;

/// Event handler trait for processing events
///
/// Implement this trait to receive and process Ghost events.
pub trait EventHandler: Send + Sync {
    /// Handles an event
    ///
    /// This method is called for each event emitted by the Ghost engine.
    /// Implementations should be fast and non-blocking.
    fn handle(&self, event: &GhostEvent);
}

/// Default event logger
///
/// Logs all events at INFO level using the tracing crate.
pub struct EventLogger;

impl EventHandler for EventLogger {
    fn handle(&self, event: &GhostEvent) {
        tracing::info!(
            event_type = %event.event_type(),
            "Ghost event"
        );
    }
}

/// Event metrics collector
///
/// Collects counts of events by type for monitoring.
pub struct EventMetrics {
    /// Total events processed
    pub total_events: std::sync::atomic::AtomicU64,
    /// Events by type
    pub by_type: std::sync::RwLock<std::collections::HashMap<String, u64>>,
}

impl EventMetrics {
    /// Creates new event metrics
    pub fn new() -> Self {
        Self {
            total_events: std::sync::atomic::AtomicU64::new(0),
            by_type: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Returns the total event count
    pub fn total(&self) -> u64 {
        self.total_events.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns count for a specific event type
    pub fn count_for(&self, event_type: &str) -> u64 {
        self.by_type.read()
            .map(|m| m.get(event_type).copied().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Records an event
    fn record(&self, event_type: &str) {
        self.total_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut by_type) = self.by_type.write() {
            *by_type.entry(event_type.to_string()).or_insert(0) += 1;
        }
    }
}

impl EventHandler for EventMetrics {
    fn handle(&self, event: &GhostEvent) {
        self.record(event.event_type());
    }
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus for distributing events
///
/// Manages multiple event handlers and distributes events to all of them.
pub struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
    metrics: EventMetrics,
}

impl EventBus {
    /// Creates a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            metrics: EventMetrics::new(),
        }
    }

    /// Creates a new event bus with default logger
    pub fn with_logger() -> Self {
        let mut bus = Self::new();
        bus.add_handler(Box::new(EventLogger));
        bus
    }

    /// Adds a handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Adds a handler and returns self for chaining
    pub fn with_handler(mut self, handler: Box<dyn EventHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Publishes an event to all handlers
    ///
    /// Events are delivered synchronously to all handlers in order.
    /// Handlers should be fast to avoid blocking event distribution.
    pub fn publish(&self, event: &GhostEvent) {
        // Record metrics
        self.metrics.record(event.event_type());

        // Deliver to all handlers
        for handler in &self.handlers {
            handler.handle(event);
        }
    }

    /// Returns the number of handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Returns the metrics collector
    pub fn metrics(&self) -> &EventMetrics {
        &self.metrics
    }

    /// Clears all handlers
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event filter for selective handling
pub struct EventFilter {
    /// Event types to handle (empty = all)
    allowed_types: Vec<String>,
    /// Event types to exclude
    excluded_types: Vec<String>,
}

impl EventFilter {
    /// Creates a new filter that allows all events
    pub fn new() -> Self {
        Self {
            allowed_types: Vec::new(),
            excluded_types: Vec::new(),
        }
    }

    /// Creates a filter that only allows specific event types
    pub fn allow_only(types: Vec<&str>) -> Self {
        Self {
            allowed_types: types.iter().map(|s| s.to_string()).collect(),
            excluded_types: Vec::new(),
        }
    }

    /// Creates a filter that excludes specific event types
    pub fn exclude(types: Vec<&str>) -> Self {
        Self {
            allowed_types: Vec::new(),
            excluded_types: types.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Checks if an event should be handled
    pub fn should_handle(&self, event: &GhostEvent) -> bool {
        let event_type = event.event_type().to_string();

        // Check exclusion first
        if self.excluded_types.contains(&event_type) {
            return false;
        }

        // If allow list is empty, allow all non-excluded
        if self.allowed_types.is_empty() {
            return true;
        }

        self.allowed_types.contains(&event_type)
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Filtered event handler that only processes certain events
pub struct FilteredHandler {
    filter: EventFilter,
    handler: Box<dyn EventHandler>,
}

impl FilteredHandler {
    /// Creates a new filtered handler
    pub fn new(filter: EventFilter, handler: Box<dyn EventHandler>) -> Self {
        Self { filter, handler }
    }
}

impl EventHandler for FilteredHandler {
    fn handle(&self, event: &GhostEvent) {
        if self.filter.should_handle(event) {
            self.handler.handle(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let bus = EventBus::new();
        assert_eq!(bus.handler_count(), 0);
    }

    #[test]
    fn test_event_bus_add_handler() {
        let mut bus = EventBus::new();
        bus.add_handler(Box::new(EventLogger));
        assert_eq!(bus.handler_count(), 1);
    }

    #[test]
    fn test_event_bus_publish() {
        let mut bus = EventBus::new();
        bus.add_handler(Box::new(EventLogger));

        // Should not panic - use WorkerOffline event
        bus.publish(&GhostEvent::WorkerOffline {
            worker_id: "test".to_string(),
            reason: "test".to_string(),
        });
    }

    #[test]
    fn test_event_metrics() {
        let metrics = EventMetrics::new();
        assert_eq!(metrics.total(), 0);

        metrics.record("test");
        assert_eq!(metrics.total(), 1);
        assert_eq!(metrics.count_for("test"), 1);
    }

    #[test]
    fn test_event_filter_allow() {
        let filter = EventFilter::allow_only(vec!["worker_registered", "worker_offline"]);

        assert!(filter.should_handle(&GhostEvent::WorkerOffline {
            worker_id: "test".to_string(),
            reason: "test".to_string(),
        }));
        assert!(!filter.should_handle(&GhostEvent::HealthCheckCompleted {
            worker_id: "test".to_string(),
            passed: true,
            latency_ms: 100,
        }));
    }

    #[test]
    fn test_event_filter_exclude() {
        let filter = EventFilter::exclude(vec!["health_check_completed"]);

        assert!(filter.should_handle(&GhostEvent::WorkerOffline {
            worker_id: "test".to_string(),
            reason: "test".to_string(),
        }));
        assert!(!filter.should_handle(&GhostEvent::HealthCheckCompleted {
            worker_id: "test".to_string(),
            passed: true,
            latency_ms: 100,
        }));
    }
}
