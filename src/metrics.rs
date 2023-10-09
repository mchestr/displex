use prometheus_client::{
    encoding::{
        EncodeLabelSet,
        EncodeLabelValue,
    },
    metrics::{
        counter::Counter,
        family::Family,
    },
    registry::Registry,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum Action {
    NewSubscriber,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CounterLabels {
    pub action: Action,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    counters: Family<CounterLabels, Counter>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            counters: Family::default(),
        }
    }

    pub fn inc_subscriber(&self) {
        self.counters
            .get_or_create(&CounterLabels {
                action: Action::NewSubscriber,
            })
            .inc();
    }
}

pub fn new_registry(metrics: &Metrics) -> Registry {
    let mut registry = Registry::default();
    registry.register(
        "displex_counters",
        "counter metrics",
        metrics.counters.clone(),
    );
    registry
}
