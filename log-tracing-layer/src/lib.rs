//! A tracing layer for logging events.
//!
//! This is a base library that can be used to create a custom tracing layer.
//!
//! ## Libraries using this crate
//!
//! - [Datadog: dd-tracing-layer](https://docs.rs/dd-tracing-layer)
//!
//! ## How to use it
//!
//! Feel free to look at the [dd-tracing-layer](https://docs.rs/dd-tracing-layer) crate to see how to use this crate, but basically, you need to provide a [`LogIngestor`] implementation.
mod layer;
mod log_ingestor;
mod visitor;

pub use layer::LogLayer;
pub use log_ingestor::{Log, LogEvent, LogIngestor};
