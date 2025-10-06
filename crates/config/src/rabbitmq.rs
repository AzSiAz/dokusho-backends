use serde::Deserialize;

/// RabbitMQ connection configuration loaded from the environment.
#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMqConfig {
    /// AMQP URI, e.g. `amqp://guest:guest@127.0.0.1:5672`.
    pub uri: String,
    /// Optional prefetch (QoS) hint applied by workers.
    pub prefetch: Option<u16>,
}

impl RabbitMqConfig {
    /// Returns the AMQP URI as a string slice.
    pub fn uri(&self) -> &str {
        &self.uri
    }
}
