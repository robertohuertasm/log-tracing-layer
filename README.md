# log-tracing-subscriber

This is a **monorepo exposing several crates**.

[![Crates.io](https://img.shields.io/crates/v/log-tracing-layer?label=log-tracing-layer&style=flat-square)](https://crates.io/crates/log-tracing-layer)

It's a base library to easily build [tracing layers](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html).

[![Crates.io](https://img.shields.io/crates/v/dd-tracing-layer?label=dd-tracing-layer&style=flat-square)](https://crates.io/crates/dd-tracing-layer)

Tracing layer that will send logs to the [Datadog Log API](https://docs.datadoghq.com/api/latest/logs/?code-lang=typescript#send-logs).

[![Crates.io](https://img.shields.io/crates/v/nr-tracing-layer?label=nr-tracing-layer&style=flat-square)](https://crates.io/crates/nr-tracing-layer)

Tracing layer that will send logs to the [New Relic Log API](https://docs.newrelic.com/docs/logs/get-started/get-started-log-management/).
