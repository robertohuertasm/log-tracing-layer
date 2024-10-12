# nr-tracing-layer

[![license](https://img.shields.io/crates/l/nr-tracing-layer?style=for-the-badge)](https://github.com/robertohuertasm/log-tracing-layer/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/nr-tracing-layer?style=for-the-badge)](https://crates.io/crates/nr-tracing-layer)
[![docs.rs](https://img.shields.io/docsrs/nr-tracing-layer?style=for-the-badge)](https://docs.rs/nr-tracing-layer)

A [tracing layer](https://tokio.rs/tokio/topics/tracing) that sends logs to the [New Relic Log API](https://docs.newrelic.com/docs/logs/get-started/get-started-log-management/).


## Requirements

You'll need a [New Relic API Key](https://docs.newrelic.com/docs/apis/intro-apis/new-relic-api-keys/) for everything to work.

## Endpoint

By default, will try to send the logs to the `US1` region.

You can easily change the region or provide a custom URL if needed.

## Example

Here's a simple example of how to set it up and use it:

```rust
use nr_tracing_layer::NewRelicOptions;
use tracing_subscriber::prelude::*;
use tracing::{instrument, subscriber}

#[instrument]
fn log(msg: &'static str) {
    tracing::info!("your message: {}", msg);
}

fn main() {
    let options = NewRelicOptions::new("my-service", "my-new-relic-api-key")
        .with_tags("env:dev");
    let dd = nr_tracing_layer::create(options);
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::Layer::new().json())
        .with(dd);
    let _s = subscriber::set_default(subscriber);
    log("hello world!");
}
```

## Caveats

The layer will send the logs either 5 seconds after the last log is received or when the buffer arrives to 1000 logs. This is basically due to a limitation in the Datadog API.
