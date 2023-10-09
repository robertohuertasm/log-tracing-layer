#[cfg(test)]
mod tests {

    use newrelic_tracing_layer::NewRelicOptions;
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
    fn newrelic_works() {
        let server = httpmock::MockServer::start();
        let _mock = server.mock(|when, then| {
            when.any_request();
            then.status(202).json_body(serde_json::json!([]));
        });
        let options = NewRelicOptions::new(
            "newrelic-tracing-layer",
            "6fbeedb52a0ca813c0f39254ebc90458FFFFNRAL",
        );
        // .with_url(server.base_url())
        //.with_tags("env:dev");
        let dd = newrelic_tracing_layer::create(options);
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::Layer::new().json())
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
