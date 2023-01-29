# log-tracing-layer

[![license](https://img.shields.io/crates/l/log-tracing-layer?style=for-the-badge)](https://github.com/robertohuertasm/log-tracing-layer/blob/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/log-tracing-layer?style=for-the-badge)](https://crates.io/crates/log-tracing-layer)
[![docs.rs](https://img.shields.io/docsrs/log-tracing-layer?style=for-the-badge)](https://docs.rs/log-tracing-layer)

A tracing layer for logging events.

This is just a base library that can be used to create a custom tracing layer.

## Libraries using this crate

- [Datadog: dd-tracing-layer](https://docs.rs/dd-tracing-layer)

## How to use it

Feel free to look at the [dd-tracing-layer](https://docs.rs/dd-tracing-layer) crate to see how to use this crate, but basically, you need to provide a [`LogIngestor`] implementation.
