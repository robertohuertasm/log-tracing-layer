use crate::log_ingestor::Log;
use crate::log_ingestor::LogIngestor;
use crate::visitor::JsonVisitor;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use tokio::sync::mpsc::unbounded_channel;
use tracing::span;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct LogLayer {
    tx: Option<tokio::sync::mpsc::UnboundedSender<Log>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl LogLayer {
    /// Creates a new `LogLayer` with the provided log ingestor.
    ///
    /// This method spawns a dedicated thread running a Tokio runtime to handle log ingestion asynchronously.
    /// The ingestor will receive logs sent through an unbounded channel, process them, and flush on shutdown.
    ///
    /// # Parameters
    ///
    /// - `ingestor`: An instance implementing the [`LogIngestor`] trait, responsible for handling log events.
    ///
    /// # Returns
    ///
    /// A new `LogLayer` instance with the ingestion thread running.
    ///
    /// # Panics
    ///
    /// Panics if the ingestion thread cannot be spawned.
    pub fn new<I>(mut ingestor: I) -> Self
    where
        I: LogIngestor + 'static,
    {
        let (tx, mut rx) = unbounded_channel::<Log>();
        // create a separate thread to manage log ingestion
        let handle = std::thread::Builder::new()
            .name(ingestor.name().into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Err(e) => {
                        eprintln!("Runtime creation failure: {e:?}");
                        return;
                    }
                    Ok(r) => r,
                };

                rt.block_on(async move {
                    ingestor.start();
                    while let Some(log) = rx.recv().await {
                        ingestor.ingest(log).await;
                    }
                    ingestor.flush().await;
                });
                drop(rt);
            })
            .expect("Something went wrong spawning the thread");

        Self {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    fn create_log<S: Subscriber + for<'a> LookupSpan<'a>>(
        event: &tracing::Event<'_>,
        ctx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> Map<String, Value> {
        let mut log: Map<String, Value> = Map::new();
        let mut spans: Vec<Map<String, Value>> = vec![];

        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let mut new_span: Map<String, Value> = Map::new();
                new_span.insert("name".to_string(), json!(span.name()));

                if let Some(fields) = span.extensions().get::<Map<String, Value>>() {
                    fields.iter().for_each(|(k, v)| {
                        new_span.insert(k.clone(), v.clone());
                    });
                }

                spans.push(new_span);
            }
        }

        // if no last span, it means there are no spans at all
        if let Some(last) = spans.last() {
            log.insert("span".to_string(), json!(last));
            log.insert("spans".to_string(), json!(spans));
        }

        log.insert(
            "level".to_string(),
            json!(event.metadata().level().as_str()),
        );
        log.insert("target".to_string(), json!(event.metadata().target()));

        if let Some(file) = event.metadata().file() {
            log.insert("file".to_string(), json!(file));
        }
        if let Some(line) = event.metadata().line() {
            log.insert("line".to_string(), json!(line));
        }

        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);

        visitor.fields.iter().for_each(|(k, v)| {
            log.insert(k.clone(), v.clone());
        });

        log.insert(
            "timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        log
    }
}

impl Drop for LogLayer {
    fn drop(&mut self) {
        // closing the channel
        if let Some(tx) = self.tx.take() {
            drop(tx);
        }
        // waiting for the thread to finish
        if let Some(handle) = self.handle.take() {
            let _result = handle.join();
        }
    }
}

impl<S> Layer<S> for LogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        // visit values and insert them into extensions as serde_json::Map<String, serde_json::Value>
        // this way, we will be able to access them later
        let mut visitor = JsonVisitor::default();
        attrs.record(&mut visitor);
        extensions.insert(visitor.fields);
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        // send to the channel
        if let Some(tx) = &self.tx {
            let log = Self::create_log(event, &ctx);
            if let Err(e) = tx.send(log) {
                eprintln!("LAYER: Error sending log to ingestor: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::RwLock;
    use tracing::info;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    pub struct TestLogIngestor {
        pub vec: Arc<RwLock<Vec<Value>>>,
    }

    impl TestLogIngestor {
        pub fn new(vec: Arc<RwLock<Vec<Value>>>) -> Self {
            Self { vec }
        }
    }

    #[async_trait]
    impl LogIngestor for TestLogIngestor {
        fn name(&self) -> &'static str {
            "TestLogIngestor"
        }
        fn start(&self) {}
        async fn ingest(&mut self, log: Log) {
            self.vec
                .write()
                .unwrap()
                .push(log.get("span").unwrap().clone());
        }
        async fn flush(&mut self) {}
    }

    // Credits to https://github.com/DirkRusche
    // See https://github.com/robertohuertasm/log-tracing-layer/issues/7#issuecomment-3161295281
    #[test]
    fn should_have_span_in_second_log() {
        // given
        let vec = Arc::new(RwLock::new(vec![]));
        let test_log_ingestor = TestLogIngestor::new(vec.clone());
        let log = LogLayer::new(test_log_ingestor);

        let _subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::Layer::new())
            .with(log)
            .init();

        // when
        let span = span!(tracing::Level::INFO, "test", "foo" = "bar");
        span.in_scope(|| {
            info!("log1");
            info!("log2");
        });

        // to make sure the ingestion is finished
        std::thread::sleep(std::time::Duration::from_millis(100));

        // then
        let read_lock = vec.read().unwrap();

        println!("READ: {:?}", read_lock);

        assert_eq!(read_lock.len(), 2);

        assert_eq!(read_lock[0]["name"].as_str().unwrap(), "test");
        assert_eq!(read_lock[0]["foo"].as_str().unwrap(), "bar");

        assert_eq!(read_lock[1]["name"].as_str().unwrap(), "test");
        assert_eq!(read_lock[1]["foo"].as_str().unwrap(), "bar");
    }
}
