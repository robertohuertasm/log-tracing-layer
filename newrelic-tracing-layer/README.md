# newrelic-tracing-layer

[![license](https://img.shields.io/crates/l/dd-tracing-layer?style=for-the-badge)](https://github.com/robertohuertasm/log-tracing-layer/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/dd-tracing-layer?style=for-the-badge)](https://crates.io/crates/dd-tracing-layer)
[![docs.rs](https://img.shields.io/docsrs/dd-tracing-layer?style=for-the-badge)](https://docs.rs/dd-tracing-layer)

A [tracing layer](https://tokio.rs/tokio/topics/tracing) that sends logs to the [Datadog Log API](https://docs.datadoghq.com/api/latest/logs/?code-lang=typescript#send-logs).

It's mainly useful when you don't have access to your infrastructure and you cannot use the [Datadog Agent](https://docs.datadoghq.com/agent/) or any [other mean](https://docs.datadoghq.com/logs/log_collection/?tab=host#setup).

## Requirements

You'll need a [Datadog API Key](https://docs.datadoghq.com/account_management/api-app-keys/#api-keys) for everything to work.

## Endpoint

This crate uses the `v2` logs endpoints and, by default, will try to send the logs to the `US1` region.

You can easily change the region or provide a custom URL if needed.

## Example

Here's a simple example of how to set it up and use it:

```rust
use dd_tracing_layer::DatadogOptions;
use tracing_subscriber::prelude::*;
use tracing::{instrument, subscriber};

#[instrument]
fn log(msg: &'static str) {
    tracing::info!("your message: {}", msg);
}

fn main() {
    let options = DatadogOptions::new("my-service", "my-datadog-api-key")
        .with_tags("env:dev");
    let dd = dd_tracing_layer::create(options);
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::Layer::new().json())
        .with(dd);
    let _s = subscriber::set_default(subscriber);
    log("hello world!");
}
```

## Caveats

The layer will send the logs either 5 seconds after the last log is received or when the buffer arrives to 1000 logs. This is basically due to a limitation in the Datadog API.
