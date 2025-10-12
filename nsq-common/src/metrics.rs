//! Metrics collection and reporting

use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;
use crate::config::BaseConfig;
use crate::errors::{NsqError, Result};

/// Metrics collector
pub struct Metrics {
    counters: Arc<DashMap<String, u64>>,
    gauges: Arc<DashMap<String, f64>>,
    histograms: Arc<DashMap<String, Vec<f64>>>,
    statsd_client: Option<statsd::Client>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new(config: &BaseConfig) -> Result<Self> {
        let statsd_client = if let Some(addr) = &config.statsd_address {
            Some(statsd::Client::new(addr, &config.statsd_prefix)
                .map_err(|e| NsqError::Metrics(e.to_string()))?)
        } else {
            None
        };
        
        Ok(Self {
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
            statsd_client,
        })
    }
    
    /// Increment a counter
    pub fn incr(&self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
        
        if let Some(ref client) = self.statsd_client {
            let _ = client.count(name, value as f64);
        }
    }
    
    /// Set a gauge value
    pub fn gauge(&self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
        
        if let Some(ref client) = self.statsd_client {
            let _ = client.gauge(name, value);
        }
    }
    
    /// Record a histogram value
    pub fn histogram(&self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        
        if let Some(ref client) = self.statsd_client {
            let _ = client.time(name, || value as f64);
        }
    }
    
    /// Time a function execution
    pub fn time<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.histogram(name, duration.as_millis() as f64);
        result
    }
    
    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).map(|v| *v).unwrap_or(0)
    }
    
    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).map(|v| *v)
    }
    
    /// Get histogram statistics
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        self.histograms.get(name).map(|values| {
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let count = sorted_values.len();
            if count == 0 {
                return HistogramStats::default();
            }
            
            let sum: f64 = sorted_values.iter().sum();
            let mean = sum / count as f64;
            
            let median = if count % 2 == 0 {
                (sorted_values[count / 2 - 1] + sorted_values[count / 2]) / 2.0
            } else {
                sorted_values[count / 2]
            };
            
            let p95_idx = (count as f64 * 0.95) as usize;
            let p99_idx = (count as f64 * 0.99) as usize;
            
            HistogramStats {
                count,
                sum,
                mean,
                median,
                p95: sorted_values[p95_idx.min(count - 1)],
                p99: sorted_values[p99_idx.min(count - 1)],
                min: sorted_values[0],
                max: sorted_values[count - 1],
            }
        })
    }
    
    /// Get all metrics as a snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters: std::collections::HashMap<String, u64> = self.counters
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        
        let gauges: std::collections::HashMap<String, f64> = self.gauges
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        
        let histograms: std::collections::HashMap<String, HistogramStats> = self.histograms
            .iter()
            .map(|entry| (entry.key().clone(), self.get_histogram_stats(entry.key()).unwrap_or_default()))
            .collect();
        
        MetricsSnapshot {
            counters,
            gauges,
            histograms,
        }
    }
}

/// Histogram statistics
#[derive(Debug, Clone, Default)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
}

/// Metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub counters: std::collections::HashMap<String, u64>,
    pub gauges: std::collections::HashMap<String, f64>,
    pub histograms: std::collections::HashMap<String, HistogramStats>,
}

/// Timer for measuring durations
pub struct Timer {
    name: String,
    metrics: Metrics,
    start: Instant,
}

impl Timer {
    /// Create a new timer
    pub fn new(name: String, metrics: Metrics) -> Self {
        Self {
            name,
            metrics,
            start: Instant::now(),
        }
    }
    
    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        self.metrics.histogram(&self.name, duration.as_millis() as f64);
    }
}

impl Clone for Metrics {
    fn clone(&self) -> Self {
        Self {
            counters: self.counters.clone(),
            gauges: self.gauges.clone(),
            histograms: self.histograms.clone(),
            statsd_client: None, // statsd client cannot be cloned
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics.histogram(&self.name, duration.as_millis() as f64);
    }
}
