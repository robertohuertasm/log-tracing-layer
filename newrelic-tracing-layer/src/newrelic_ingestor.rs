use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use log_tracing_layer::{Log, LogEvent, LogIngestor};
use serde_json::json;
use std::{collections::VecDeque, error::Error, io::Write, sync::Arc};
use tokio::sync::RwLock;

const DD_SOURCE: &str = "newrelic-tracing-layer";
const MAX_BATCH_SIZE: usize = 1000;
const MAX_BATCH_DURATION_SECS: i64 = 5;
const MAX_RETRIES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    US,
    EU,
    FedRAMP,
}

#[derive(Debug, Default)]
pub struct NewRelicOptions {
    pub api_key: String,
    pub service_name: String,
    pub region: Option<Region>,
    pub url: Option<String>,
    pub common: Option<Vec<String>>,
}

impl NewRelicOptions {
    pub fn new(service_name: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub const fn with_region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    #[must_use]
    pub fn with_commons(mut self, common: Vec<String>) -> Self {
        self.common = Some(common.into());
        self
    }

    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

#[derive(Debug, Default)]
pub struct NewRelicLogIngestor {
    url: String,
    service_name: String,
    api_key: String,
    // common: Option<Vec<String>>,
    client: reqwest::Client,
    queue: Arc<RwLock<VecDeque<LogEvent>>>,
}

impl NewRelicLogIngestor {
    pub fn new(options: NewRelicOptions) -> Self {
        // https://docs.newrelic.com/docs/logs/log-api/introduction-log-api/
        let url = options.url.unwrap_or_else(|| {
            match options.region {
                Some(Region::US) | None => "https://log-api.newrelic.com/log/v1",
                Some(Region::EU) => "https://log-api.eu.newrelic.com/log/v1",
                Some(Region::FedRAMP) => "https://gov-log-api.newrelic.com/log/v1",
            }
            .to_string()
        });

        let source_tags = &format!("source-version:{}", env!("CARGO_PKG_VERSION"));

        // let common = options
        //     .common
        //     .map_or_else(|| source_tags.into(), |t| format!("{t}, {source_tags}"));

        Self {
            url,
            service_name: options.service_name,
            api_key: options.api_key,
            // tags,
            client: reqwest::Client::new(),
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    fn compress(&self, logs: &[Log]) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes = serde_json::to_vec(&logs)?;
        let mut encoder = libflate::gzip::Encoder::new(Vec::new())?;
        encoder.write_all(&bytes)?;
        let result = encoder.finish().into_result()?;
        Ok(result)
    }

    #[async_recursion]
    async fn send_logs(&self, logs: &[Log], retries: u8) {
        if retries > MAX_RETRIES {
            eprintln!("Failed to send logs after {} retries", retries);
        }

        let retry = || self.send_logs(logs, retries + 1);

        // compress the logs
        let compressed_logs = match self.compress(logs) {
            Ok(logs) => logs,
            Err(e) => {
                eprintln!("Failed to compress logs: {:?}", e);
                return;
            }
        };

        match self
            .client
            .post(&self.url)
            .header("User-Agent", "newrelic-tracing-subscriber/0.1.0")
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Content-Encoding", "gzip")
            .body(compressed_logs)
            .send()
            .await
        {
            Ok(res) => match res.status().as_u16() {
                202 => {
                    //println!("Accepted: the request has been accepted for processing");
                }
                400 => {
                    eprintln!("Bad request (likely an issue in the payload formatting)");
                }
                401 => {
                    eprintln!("Unauthorized (likely a missing API Key)");
                }
                403 => {
                    eprintln!("Permission issue (likely using an invalid API Key)");
                }
                408 => {
                    eprintln!("Request Timeout, request should be retried after some time");
                    retry().await;
                }
                413 => {
                    eprintln!("Payload too large (batch is above 1MB uncompressed)");
                    // split batch
                    let logs_len = logs.len();
                    let half = logs_len / 2;
                    let (left, right) = logs.split_at(half);
                    self.send_logs(left, retries + 1).await;
                    self.send_logs(right, retries + 1).await;
                }
                429 => {
                    eprintln!("Too Many Requests, request should be retried after some time");
                    retry().await;
                }
                500 => {
                    eprintln!("Internal Server Error, the server encountered an unexpected condition that prevented it from fulfilling the request, request should be retried after some time");
                    retry().await;
                }
                503 => {
                    eprintln!("Service Unavailable, the server is not ready to handle the request probably because it is overloaded, request should be retried after some time");
                    retry().await;
                }
                _ => {
                    eprintln!("Unknown error, try again later");
                    retry().await;
                }
            },
            Err(e) => {
                eprintln!("Failed to send logs to New Relic: {:?}", e);
            }
        }
    }

    #[async_recursion]
    async fn try_send(&mut self, is_flush: bool) {
        {
            // send current logs to new relic if there are any
            let queue = self.queue.read().await;
            if queue.is_empty() {
                return;
            }
            if !is_flush {
                // send the logs only if the last one is more than 5 seconds old
                // or if the queue has more than MAX_BATCH_SIZE logs
                let last_log = queue.back().unwrap();
                let now = Utc::now();
                let diff = now - last_log.received_at;
                if diff < Duration::seconds(MAX_BATCH_DURATION_SECS) && queue.len() < MAX_BATCH_SIZE
                {
                    return;
                }
            }
        }

        // get the logs to send
        let logs = {
            let mut queue = self.queue.write().await;
            // max amount of logs to send at once is 1000
            let tail = usize::min(queue.len(), MAX_BATCH_SIZE);
            queue.drain(..tail).map(|e| e.log).collect::<Vec<_>>()
        };

        // send them (retries if it fails)
        self.send_logs(&logs, 0).await;

        // check if the queue is empty and flush again if it's not
        let is_queue_empty = { self.queue.read().await.is_empty() };
        if !is_queue_empty {
            self.try_send(is_flush).await;
        }
    }
}

impl Clone for NewRelicLogIngestor {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            service_name: self.service_name.clone(),
            api_key: self.api_key.clone(),
            // tags: self.tags.clone(),
            client: self.client.clone(),
            queue: self.queue.clone(),
        }
    }
}

#[async_trait]
impl LogIngestor for NewRelicLogIngestor {
    fn name(&self) -> &'static str {
        "new_relic"
    }

    fn start(&self) {
        // start a timer that will flush the queue every n seconds
        let mut this = self.clone();
        tokio::spawn(async move {
            let period = std::time::Duration::from_secs(MAX_BATCH_DURATION_SECS as u64);
            let mut interval = tokio::time::interval(period);
            loop {
                interval.tick().await;
                this.try_send(false).await;
            }
        });
    }

    async fn ingest(&mut self, mut log: Log) {
        // add new relic specific fields
        log.insert("source".to_string(), json!(DD_SOURCE));
        log.insert("service".to_string(), json!(self.service_name));
        // log.insert("ddtags".to_string(), json!(self.tags));
        // TODO: (ROB) arrays are not supported so we must index the spans property

        let log_event = LogEvent {
            log,
            received_at: Utc::now(),
        };
        self.queue.write().await.push_back(log_event);
    }

    async fn flush(&mut self) {
        self.try_send(true).await;
    }
}
