use std::str::FromStr;

use serde_json::json;
use tracing::field::Visit;

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct JsonVisitor {
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl JsonVisitor {
    fn filter_insert(&mut self, field: &tracing::field::Field, value: serde_json::Value) {
        self.fields.insert(field.name().to_string(), value);
    }
}
impl Visit for JsonVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        // try to parse the string in case it's already a json value
        let json_value = serde_json::Value::from_str(value).unwrap_or_else(|_| json!(value));
        self.filter_insert(field, json_value);
    }
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.filter_insert(field, json!(value));
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.filter_insert(field, json!(value));
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.filter_insert(field, json!(value));
    }
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.filter_insert(field, json!(value));
    }
    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.filter_insert(field, json!(value.to_string()));
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.filter_insert(field, json!(format!("{value:?}")));
    }
}
