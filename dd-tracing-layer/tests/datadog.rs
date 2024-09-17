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

    fn get_api_key() -> String {
        std::env::var("DD_API_KEY").expect("DD_API_KEY must be set as an environment variable")
    }

    fn setup(server: &httpmock::MockServer) -> (httpmock::Mock, dd_tracing_layer::LogLayer) {
        dotenvy::from_filename(".env").ok();
        let api_key = get_api_key();

        let url = server.base_url().clone();
        let mock = server.mock(|when, then| {
            when.any_request().header_exists("DD-API-KEY");
            then.status(202).json_body(serde_json::json!([]));
        });

        let options = DatadogOptions::new("dd-tracing-layer", api_key)
            .with_url(url)
            .with_tags("env:dev");

        let dd = dd_tracing_layer::create(options);

        (mock, dd)
    }

    #[test]
    fn datadog_works() {
        // server where logs will be sent
        let server = httpmock::MockServer::start();
        let (mock_server, dd) = setup(&server);

        // create the subscriber
        let subscriber = tracing_subscriber::registry().with(dd);
        let _s = subscriber::set_default(subscriber);

        // test the logs
        log("this is a test message");
        // assert the server was hit, and we wait for the logs to be sent
        std::thread::sleep(std::time::Duration::from_secs(8));
        assert_eq!(mock_server.hits(), 1);
    }

    /// This test is just to test manually test the feature
    /// Comment the ignore attribute to run the test and alter the code
    /// as you see fit.
    #[test]
    #[ignore]
    fn manual_tests() {
        // server where logs will be sent
        let server = httpmock::MockServer::start();
        let (_, dd) = setup(&server);

        // create the subscriber
        let subscriber = tracing_subscriber::registry()
            // this shows the traces to the terminal...
            .with(tracing_subscriber::fmt::Layer::new().json())
            .with(dd);
        let _s = subscriber::set_default(subscriber);

        // test the logs and check the terminal
        log("a");
        // proof that logs are not blocking
        std::thread::sleep(std::time::Duration::from_secs(2));
        // plain log
        tracing::info!(
            ip = "127.0.0.1",
            person = r#"{ "name": "rob", "age": 15 }"#,
            message = "Testing Json"
        );
        // yet another log
        log("3a");
    }
}
