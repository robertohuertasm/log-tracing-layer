//! A tracing layer that sends logs to [New Relic](https://docs.newrelic.com/docs/logs/log-api/introduction-log-api/).
//!
//! It's mainly useful when you don't have access to your infrastructure and you cannot use the [Datadog Agent](https://docs.datadoghq.com/agent/) or any [other mean](https://docs.datadoghq.com/logs/log_collection/?tab=host#setup).
//!
//! ## Example
//!
//! ```rust
//! use dd_tracing_layer::DatadogOptions;
//! use tracing_subscriber::prelude::*;
//! use tracing::{instrument, subscriber};
//!
//! #[instrument]
//! fn log(msg: &'static str) {
//!   tracing::info!("your message: {}", msg);
//! }
//!
//! fn main() {
//!     let options = DatadogOptions::new("my-service", "my-datadog-api-key")
//!         .with_tags("env:dev");
//!     let dd = dd_tracing_layer::create(options);
//!     let subscriber = tracing_subscriber::registry()
//!         .with(tracing_subscriber::fmt::Layer::new().json())
//!         .with(dd);
//!     let _s = subscriber::set_default(subscriber);
//!     log("hello world!");
//! }
//!```
mod newrelic_ingestor;

pub use log_tracing_layer::LogLayer;
pub use newrelic_ingestor::{NewRelicOptions, Region};

/// Creates a log layer that will send logs to New Relic
#[must_use]
pub fn create(options: NewRelicOptions) -> LogLayer {
    let ingestor = newrelic_ingestor::NewRelicLogIngestor::new(options);
    LogLayer::new(ingestor)
}
