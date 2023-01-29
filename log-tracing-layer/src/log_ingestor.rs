use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};

pub type Log = Map<String, Value>;

#[derive(Debug)]
pub struct LogEvent {
    pub log: Log,
    pub received_at: DateTime<Utc>,
}

#[async_trait]
pub trait LogIngestor: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self);
    async fn ingest(&mut self, log: Log);
    async fn flush(&mut self);
}
