use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Serialize, Deserialize)]
pub struct MetricEvent {
    pub event_id: String,
    pub value: Option<f64>,
    pub tags: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

pub struct MetricStore {
    metrics: HashMap<String, VecDeque<MetricEvent>>,
    retention: Duration,
    data_dir: PathBuf,
}

impl MetricStore {
    pub fn new(retention: Duration, data_dir: PathBuf) -> Self {
        let mut metrics: HashMap<String, VecDeque<MetricEvent>> = HashMap::new();

        if let Ok(bots) = storage::discover_bots(&data_dir) {
            for bot_name in bots {
                match storage::load_lines::<MetricEvent>(&data_dir, &bot_name) {
                    Ok(events) => {
                        if !events.is_empty() {
                            metrics.insert(bot_name, VecDeque::from(events));
                        }
                    }
                    Err(e) => eprintln!("warning: failed to load metrics for {bot_name}: {e}"),
                }
            }
        }

        MetricStore {
            metrics,
            retention,
            data_dir,
        }
    }

    pub fn record(
        &mut self,
        bot_name: &str,
        event_id: String,
        value: Option<f64>,
        tags: HashMap<String, String>,
    ) -> DateTime<Utc> {
        let timestamp = Utc::now();
        let event = MetricEvent {
            event_id,
            value,
            tags,
            timestamp,
        };

        if let Err(e) = storage::append_line(&self.data_dir, bot_name, &event) {
            eprintln!("warning: failed to persist metric for {bot_name}: {e}");
        }

        let events = self.metrics.entry(bot_name.to_owned()).or_default();
        events.push_back(event);
        timestamp
    }

    pub fn query<'a>(
        &'a self,
        bot_name: &str,
        event_id_filter: Option<&str>,
    ) -> Vec<&'a MetricEvent> {
        let Some(events) = self.metrics.get(bot_name) else {
            return Vec::new();
        };
        match event_id_filter {
            Some(filter) => events.iter().filter(|e| e.event_id == filter).collect(),
            None => events.iter().collect(),
        }
    }

    pub fn event_ids(&self, bot_name: &str) -> Vec<String> {
        let Some(events) = self.metrics.get(bot_name) else {
            return Vec::new();
        };
        let mut ids: Vec<String> = events
            .iter()
            .map(|e| e.event_id.clone())
            .collect::<HashSet<_>>() // de-dupe
            .into_iter()
            .collect();
        ids.sort();
        ids
    }

    pub fn prune(&mut self) {
        let cutoff = Utc::now() - self.retention;
        for events in self.metrics.values_mut() {
            while events.front().is_some_and(|e| e.timestamp < cutoff) {
                events.pop_front();
            }
        }

        let empty_bots: Vec<String> = self
            .metrics
            .iter()
            .filter(|(_, events)| events.is_empty())
            .map(|(name, _)| name.clone())
            .collect();
        for name in &empty_bots {
            self.metrics.remove(name);
            if let Err(e) = storage::remove_bot_file(&self.data_dir, name) {
                eprintln!("warning: failed to remove metric file for {name}: {e}");
            }
        }

        for (name, events) in &self.metrics {
            if let Err(e) = storage::rewrite_lines(&self.data_dir, name, events.iter()) {
                eprintln!("warning: failed to rewrite metrics for {name}: {e}");
            }
        }
    }

    pub fn remove_bot(&mut self, name: &str) {
        self.metrics.remove(name);
        if let Err(e) = storage::remove_bot_file(&self.data_dir, name) {
            eprintln!("warning: failed to remove metric file for {name}: {e}");
        }
    }
}
