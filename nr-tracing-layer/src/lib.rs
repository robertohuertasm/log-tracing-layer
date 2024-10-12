//! A tracing layer that sends logs to [New Relic](https://docs.newrelic.com/docs/logs/get-started/get-started-log-management/).
//!
//! ## Example
//!
//! ```rust
//! use nr_tracing_layer::NewRelicOptions;
//! use tracing_subscriber::prelude::*;
//! use tracing::{instrument, subscriber};
//!
//! #[instrument]
//! fn log(msg: &'static str) {
//!   tracing::info!("your message: {}", msg);
//! }
//!
//! fn main() {
//!     let options = NewRelicOptions::new("my-service", "my-new-relic-api-key")
//!         .with_tags("env:dev");
//!     let dd = nr_tracing_layer::create(options);
//!     let subscriber = tracing_subscriber::registry()
//!         .with(tracing_subscriber::fmt::Layer::new().json())
//!         .with(dd);
//!     let _s = subscriber::set_default(subscriber);
//!     log("hello world!");
//! }
//!```
mod new_relic_ingestor;

pub use log_tracing_layer::LogLayer;
pub use new_relic_ingestor::{NewRelicOptions, Region};

/// Creates a log layer that will send logs to New Relic.
#[must_use]
pub fn create(options: NewRelicOptions) -> LogLayer {
    let ingestor = new_relic_ingestor::NewRelicLogIngestor::new(options);
    LogLayer::new(ingestor)
}
