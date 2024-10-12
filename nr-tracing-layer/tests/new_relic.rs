#[cfg(test)]
mod tests {

    use nr_tracing_layer::{NewRelicOptions, Region};
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
        std::env::var("NR_API_KEY").unwrap_or("invented_api_key".to_string())
    }

    fn setup(server: &httpmock::MockServer) -> (httpmock::Mock, nr_tracing_layer::LogLayer) {
        dotenvy::from_filename(".env").ok();
        let api_key = get_api_key();

        let url = server.base_url().clone();
        let mock = server.mock(|when, then| {
            when.any_request().header_exists("Api-Key");
            then.status(202).json_body(serde_json::json!([]));
        });

        let options = NewRelicOptions::new("nr-tracing-layer", api_key)
            .with_url(url)
            .with_tags("env:dev");

        let dd = nr_tracing_layer::create(options);

        (mock, dd)
    }

    #[test]
    fn new_relic_works() {
        // server where logs will be sent
        let server = httpmock::MockServer::start();
        let (mock_server, nr) = setup(&server);

        // create the subscriber
        let subscriber = tracing_subscriber::registry().with(nr);
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
        let (_, nr) = setup(&server);

        // create the subscriber
        let subscriber = tracing_subscriber::registry()
            // this shows the traces to the terminal...
            .with(tracing_subscriber::fmt::Layer::new().json())
            .with(nr);
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

    #[test]
    #[ignore]
    fn nr_manual_tests_without_mock_server() {
        // set up
        dotenvy::from_filename(".env").ok();
        let api_key = get_api_key();

        let options = NewRelicOptions::new("nr-tracing-layer", api_key)
            .with_region(Region::US)
            .with_tags("env:dev");

        let nr = nr_tracing_layer::create(options);

        // create the subscriber
        let subscriber = tracing_subscriber::registry()
            // this shows the traces to the terminal...
            .with(tracing_subscriber::fmt::Layer::new().json())
            .with(nr);
        let _s = subscriber::set_default(subscriber);

        // test the logs and check the terminal
        // log("mensaje de prueba");

        // // proof that logs are not blocking
        // std::thread::sleep(std::time::Duration::from_secs(2));
        // // plain log
        tracing::info!(
            ip = "127.0.0.1",
            person = r#"{ "name": "rob", "age": 15 }"#,
            message = "Testing Json1"
        );
        tracing::info!(
            ip = "127.0.0.1",
            person = r#"{ "name": "rob", "age": 15 }"#,
            message = "Testing Json2"
        );
        tracing::info!(
            ip = "127.0.0.1",
            person = r#"{ "name": "rob", "age": 15 }"#,
            message = "Testing Json3"
        );
        // // yet another log
        // log("3a");
    }
}
