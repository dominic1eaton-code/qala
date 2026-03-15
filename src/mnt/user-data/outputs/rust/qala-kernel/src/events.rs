//! Event Aggregator — collects, counts, and stores recent platform events.
//!
//! The kernel acts as the central event telemetry sink. Every service that
//! processes an event posts a copy to `POST /v1/kernel/events`. The aggregator:
//!
//! 1. Increments a counter for every `(topic, source_service)` pair seen.
//! 2. Pushes the envelope into a fixed-capacity ring buffer (oldest evicted first).
//! 3. Exposes query capabilities for the `/v1/kernel/events` GET endpoint.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use qala_shared::events::{EventEnvelope, EventTopic};

// ── Topic counter ─────────────────────────────────────────────────────────────

/// Aggregated counts for a single (topic, source_service) pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCounter {
    pub topic:          String,
    pub source_service: String,
    pub count:          u64,
    pub first_seen:     DateTime<Utc>,
    pub last_seen:      DateTime<Utc>,
}

// ── Query parameters ──────────────────────────────────────────────────────────

/// Query parameters for `GET /v1/kernel/events`.
#[derive(Debug, Default, Deserialize)]
pub struct EventQuery {
    /// Filter by topic name (e.g. `"SDE_EVENTS"`).
    pub topic: Option<String>,
    /// Filter by source service name.
    pub source_service: Option<String>,
    /// Maximum number of events to return (default 50, max 500).
    pub limit: Option<usize>,
}

// ── Aggregator ────────────────────────────────────────────────────────────────

/// In-memory event telemetry store.
///
/// Stores a bounded ring buffer of recent event envelopes and a counter map
/// keyed by `"TOPIC::source_service"`.
#[derive(Debug)]
pub struct EventAggregator {
    /// Ring buffer of recent events — oldest are dropped when capacity is reached.
    buffer: VecDeque<EventEnvelope>,
    /// Maximum number of events to retain in the ring buffer.
    capacity: usize,
    /// Counter map: key = `"TOPIC::source_service"` → count record.
    counters: HashMap<String, TopicCounter>,
    /// Total number of events ever ingested (monotonically increasing).
    total_ingested: u64,
}

impl EventAggregator {
    /// Create a new aggregator with the given ring buffer capacity.
    ///
    /// # Panics
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "EventAggregator capacity must be > 0");
        Self {
            buffer:         VecDeque::with_capacity(capacity),
            capacity,
            counters:       HashMap::new(),
            total_ingested: 0,
        }
    }

    /// Ingest a new event envelope.
    ///
    /// Updates the counter for `(topic, source_service)` and appends the
    /// envelope to the ring buffer. If the buffer is full, the oldest event
    /// is silently evicted.
    pub fn ingest(&mut self, envelope: EventEnvelope) {
        // Update or create counter
        let key = format!("{}::{}", envelope.topic, envelope.source_service);
        let now = Utc::now();
        let counter = self.counters.entry(key).or_insert_with(|| TopicCounter {
            topic:          envelope.topic.to_string(),
            source_service: envelope.source_service.clone(),
            count:          0,
            first_seen:     now,
            last_seen:      now,
        });
        counter.count += 1;
        counter.last_seen = now;

        // Append to ring buffer, evicting oldest if at capacity
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(envelope);

        self.total_ingested += 1;
    }

    /// Query recent events with optional filters and a result limit.
    pub fn query(&self, q: &EventQuery) -> Vec<&EventEnvelope> {
        let limit = q.limit.unwrap_or(50).min(500);

        self.buffer
            .iter()
            .rev() // most recent first
            .filter(|e| {
                if let Some(ref topic) = q.topic {
                    if e.topic.to_string() != *topic {
                        return false;
                    }
                }
                if let Some(ref svc) = q.source_service {
                    if e.source_service != *svc {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .collect()
    }

    /// Return all current topic counters, ordered by count descending.
    pub fn counters_by_count(&self) -> Vec<&TopicCounter> {
        let mut v: Vec<&TopicCounter> = self.counters.values().collect();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        v
    }

    /// Return the counter for a specific topic, summed across all source services.
    pub fn topic_total(&self, topic: &EventTopic) -> u64 {
        let topic_str = topic.to_string();
        self.counters
            .values()
            .filter(|c| c.topic == topic_str)
            .map(|c| c.count)
            .sum()
    }

    /// Total events ever ingested.
    pub fn total_ingested(&self) -> u64 {
        self.total_ingested
    }

    /// Number of events currently in the ring buffer.
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Ring buffer capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Snapshot summary suitable for the kernel status response.
    pub fn summary(&self) -> EventAggregatorSummary {
        EventAggregatorSummary {
            total_ingested:   self.total_ingested,
            buffer_size:      self.buffer_len(),
            buffer_capacity:  self.capacity,
            topic_counters:   self.counters_by_count().into_iter().cloned().collect(),
        }
    }
}

/// Summary of the event aggregator state, embedded in the kernel status response.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventAggregatorSummary {
    pub total_ingested:  u64,
    pub buffer_size:     usize,
    pub buffer_capacity: usize,
    pub topic_counters:  Vec<TopicCounter>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use qala_shared::events::{EventEnvelope, EventTopic};
    use serde_json::json;

    fn make_envelope(topic: EventTopic, source: &str, event_type: &str) -> EventEnvelope {
        EventEnvelope::new(topic, event_type, source, None, json!({}))
    }

    #[test]
    fn ingest_increments_counter() {
        let mut agg = EventAggregator::new(10);
        agg.ingest(make_envelope(EventTopic::SdeEvents, "sde-svc", "sde.created"));
        agg.ingest(make_envelope(EventTopic::SdeEvents, "sde-svc", "sde.created"));
        assert_eq!(agg.total_ingested(), 2);
        assert_eq!(agg.topic_total(&EventTopic::SdeEvents), 2);
    }

    #[test]
    fn ring_buffer_evicts_oldest() {
        let mut agg = EventAggregator::new(3);
        for i in 0..5u32 {
            let env = EventEnvelope::new(
                EventTopic::BuildEvents,
                format!("build.step.{i}"),
                "ci-svc",
                None,
                json!({ "step": i }),
            );
            agg.ingest(env);
        }
        assert_eq!(agg.buffer_len(), 3);
        assert_eq!(agg.total_ingested(), 5);
    }

    #[test]
    fn query_filters_by_topic() {
        let mut agg = EventAggregator::new(20);
        agg.ingest(make_envelope(EventTopic::SdeEvents,   "sde-svc",  "sde.created"));
        agg.ingest(make_envelope(EventTopic::BuildEvents, "ci-svc",   "build.started"));
        agg.ingest(make_envelope(EventTopic::BuildEvents, "ci-svc",   "build.completed"));

        let q = EventQuery { topic: Some("BUILD_EVENTS".into()), ..Default::default() };
        assert_eq!(agg.query(&q).len(), 2);
    }

    #[test]
    fn query_limit_respected() {
        let mut agg = EventAggregator::new(100);
        for _ in 0..20 {
            agg.ingest(make_envelope(EventTopic::SdeEvents, "sde-svc", "sde.created"));
        }
        let q = EventQuery { limit: Some(5), ..Default::default() };
        assert_eq!(agg.query(&q).len(), 5);
    }
}
