#[cfg(test)]
mod tests {

    use dd_tracing_layer::DatadogOptions;
    use tracing::{instrument, subscriber};
    use tracing_subscriber::prelude::*;

    #[instrument]
    fn log(msg: &'static str) {
        // tracing::info!(ip = "127.0.0.1", "Hello, world!");
        log2(msg);
    }

    #[instrument]
    fn log2(msg2: &'static str) {
        tracing::info!(ip = "127.0.0.1", message = msg2);
    }

    #[test]
    fn datadog_works() {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .with_module_level("log_tracing_layer::layer", log::LevelFilter::Debug)
            .with_module_level(
                "dd_tracing_layer::datadog_ingestor",
                log::LevelFilter::Debug,
            )
            .without_timestamps()
            .init()
            .unwrap();
        let server = httpmock::MockServer::start();
        let _mock = server.mock(|when, then| {
            when.any_request();
            then.status(202).json_body(serde_json::json!([]));
        });
        let options = DatadogOptions::new("dd-tracing-layer", "21695c1b35156511441c0d3ace5943f4")
            .with_url(server.base_url())
            .with_tags("env:dev");
        let dd = dd_tracing_layer::create(options);
        let subscriber = tracing_subscriber::registry()
            // .with(tracing_subscriber::fmt::Layer::new().json())
            .with(dd);
        let _s = subscriber::set_default(subscriber);
        log("a");
        std::thread::sleep(std::time::Duration::from_secs(2));
        log("2a");
        std::thread::sleep(std::time::Duration::from_secs(2));
        log("3a");
        std::thread::sleep(std::time::Duration::from_secs(6));
        log("4a");
    }
}
