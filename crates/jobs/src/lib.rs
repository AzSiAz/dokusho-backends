//! Type-safe RabbitMQ job utilities built on [`lapin`].
//!
//! Implement the [`Job`] trait for each unit of work. The same implementation
//! can be used by the API to enqueue messages via [`JobPublisher`], and by the
//! background worker through [`Worker::run`].
//!
//! ```no_run
//! use std::error::Error;
//!
//! use async_trait::async_trait;
//! use dokusho_config::AppConfig;
//! use dokusho_jobs::{Job, JobAction, JobContext, JobError, Message, QueueSettings, Worker};
//! use serde::{Deserialize, Serialize};
//! use uuid::Uuid;
//!
//! #[derive(Clone)]
//! struct RefreshSeriesJob {
//!     mq: dokusho_config::RabbitMqConfig,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct RefreshSeriesPayload {
//!     source_id: Uuid,
//! }
//!
//! #[async_trait]
//! impl Job for RefreshSeriesJob {
//!     type Payload = RefreshSeriesPayload;
//!
//!     fn name(&self) -> &'static str {
//!         "refresh_series"
//!     }
//!
//!     fn settings(&self) -> QueueSettings {
//!         QueueSettings::from_config(&self.mq, "refresh_series")
//!     }
//!
//!     async fn handle(
//!         &self,
//!         message: &Message<Self::Payload>,
//!         _ctx: &JobContext<Self>,
//!     ) -> Result<JobAction, JobError> {
//!         println!("Refreshing series {}", message.payload().source_id);
//!         Ok(JobAction::Ack)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let config = AppConfig::from_env()?;
//!     let job = RefreshSeriesJob {
//!         mq: config.rabbitmq.clone(),
//!     };
//!     Worker::run(job).await?;
//!     Ok(())
//! }
//! ```

use std::{fmt, sync::Arc, time::Duration};

use anyhow::{Error as AnyhowError, Result};
use async_trait::async_trait;

use chrono::Utc;
use cron::Schedule;
use dokusho_config::RabbitMqConfig;
use futures_util::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer, ExchangeKind,
    message::Delivery,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        BasicQosOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    publisher_confirm::Confirmation,
    types::FieldTable,
};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::time::sleep;
use tokio_executor_trait::Tokio as TokioExecutor;
#[cfg(unix)]
use tokio_reactor_trait::Tokio as TokioReactor;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, error, info, warn};
use uuid::Uuid;

/// Configuration describing how a job interacts with RabbitMQ.
#[derive(Debug, Clone)]
pub struct QueueSettings {
    /// AMQP URI, e.g. `amqp://guest:guest@127.0.0.1:5672/%2f`.
    pub uri: String,
    /// Name of the queue to consume from / publish to.
    pub queue_name: String,
    /// Routing key used when publishing messages. Defaults to the queue name.
    pub routing_key: String,
    /// Optional exchange configuration.
    pub exchange: Option<ExchangeConfig>,
    /// Queue declaration options.
    pub declare_options: QueueDeclareOptions,
    /// Additional queue declaration arguments.
    pub declare_arguments: FieldTable,
    /// Queue binding options.
    pub bind_options: QueueBindOptions,
    /// Additional queue binding arguments.
    pub bind_arguments: FieldTable,
    /// Consumer options.
    pub consume_options: BasicConsumeOptions,
    /// Additional arguments passed when creating the consumer.
    pub consume_arguments: FieldTable,
    /// Publishing options used when sending messages.
    pub publish_options: BasicPublishOptions,
    /// Message properties applied to each published message.
    pub publish_properties: BasicProperties,
    /// Optional prefetch count (QoS). `None` leaves the broker defaults.
    pub prefetch_count: Option<u16>,
}

impl QueueSettings {
    /// Builds default queue settings for the provided URI/queue.
    pub fn new(uri: impl Into<String>, queue: impl Into<String>) -> Self {
        let uri = uri.into();
        let queue_name = queue.into();
        let consume_options = BasicConsumeOptions {
            no_ack: false,
            ..Default::default()
        };

        Self {
            routing_key: queue_name.clone(),
            uri,
            queue_name,
            exchange: None,
            declare_options: QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            declare_arguments: FieldTable::default(),
            bind_options: QueueBindOptions::default(),
            bind_arguments: FieldTable::default(),
            consume_options,
            consume_arguments: FieldTable::default(),
            publish_options: BasicPublishOptions::default(),
            publish_properties: BasicProperties::default(),
            prefetch_count: Some(32),
        }
    }

    /// Constructs settings using a shared [`RabbitMqConfig`].
    pub fn from_config(config: &RabbitMqConfig, queue: impl Into<String>) -> Self {
        let mut settings = Self::new(config.uri.clone(), queue);
        settings.prefetch_count = config.prefetch;
        settings
    }

    /// Overrides the routing key used when publishing.
    pub fn with_routing_key(mut self, routing_key: impl Into<String>) -> Self {
        self.routing_key = routing_key.into();
        self
    }

    /// Attaches an exchange configuration.
    pub fn with_exchange(mut self, exchange: ExchangeConfig) -> Self {
        self.exchange = Some(exchange);
        self
    }

    /// Overrides the queue declaration arguments.
    pub fn with_declare_arguments(mut self, arguments: FieldTable) -> Self {
        self.declare_arguments = arguments;
        self
    }

    /// Overrides the queue binding arguments.
    pub fn with_bind_arguments(mut self, arguments: FieldTable) -> Self {
        self.bind_arguments = arguments;
        self
    }

    /// Sets the prefetch (QoS) count.
    pub fn with_prefetch(mut self, prefetch: Option<u16>) -> Self {
        self.prefetch_count = prefetch;
        self
    }

    /// Returns the exchange name, or the default exchange when not set.
    pub fn exchange_name(&self) -> &str {
        self.exchange
            .as_ref()
            .map(|exchange| exchange.name.as_str())
            .unwrap_or("")
    }

    /// Returns the routing key used for publishing.
    pub fn routing_key(&self) -> &str {
        &self.routing_key
    }
}

/// Exchange declaration configuration.
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    /// Name of the exchange.
    pub name: String,
    /// Exchange type (direct, fanout, topic, ...).
    pub kind: ExchangeKind,
    /// Exchange declaration options.
    pub declare_options: ExchangeDeclareOptions,
    /// Additional exchange arguments.
    pub declare_arguments: FieldTable,
}

impl ExchangeConfig {
    /// Builds a default durable exchange configuration.
    pub fn new(name: impl Into<String>, kind: ExchangeKind) -> Self {
        Self {
            name: name.into(),
            kind,
            declare_options: ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            declare_arguments: FieldTable::default(),
        }
    }
}

/// Decisions a job handler can return after processing a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    /// Message processed successfully; acknowledge it.
    Ack,
    /// Reject the message and dead-letter it.
    Reject,
    /// Reject the message but ask the broker to requeue it.
    Requeue,
    /// Retry the message with exponential backoff.
    Retry,
}

/// Strategy to apply when the handler returns an error or payload deserialisation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStrategy {
    /// Acknowledge the message even though an error occurred.
    Ack,
    /// Reject and dead-letter the message.
    Reject,
    /// Reject and requeue the message.
    Requeue,
    /// Retry the message with exponential backoff.
    Retry,
}

impl FailureStrategy {
    fn into_action(self) -> JobAction {
        match self {
            FailureStrategy::Ack => JobAction::Ack,
            FailureStrategy::Reject => JobAction::Reject,
            FailureStrategy::Requeue => JobAction::Requeue,
            FailureStrategy::Retry => JobAction::Retry,
        }
    }
}

/// Metadata extracted from a RabbitMQ delivery.
#[derive(Debug, Clone)]
pub struct DeliveryMetadata {
    /// Delivery tag, unique per channel.
    pub delivery_tag: u64,
    /// Whether the broker flagged this message as redelivered.
    pub redelivered: bool,
    /// Exchange the message originated from.
    pub exchange: String,
    /// Routing key associated with the delivery.
    pub routing_key: String,
    /// Optional message identifier.
    pub message_id: Option<String>,
    /// Optional correlation identifier.
    pub correlation_id: Option<String>,
    /// Arbitrary headers.
    pub headers: Option<FieldTable>,
    /// Number of retry attempts (extracted from x-retry-count header).
    pub retry_count: u32,
}

impl DeliveryMetadata {
    fn from_delivery(delivery: &Delivery) -> Self {
        let retry_count = delivery
            .properties
            .headers()
            .as_ref()
            .and_then(|headers| headers.inner().get("x-retry-count"))
            .and_then(|value| {
                if let lapin::types::AMQPValue::LongUInt(count) = value {
                    Some(*count)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        Self {
            delivery_tag: delivery.delivery_tag,
            redelivered: delivery.redelivered,
            exchange: delivery.exchange.to_string(),
            routing_key: delivery.routing_key.to_string(),
            message_id: delivery
                .properties
                .message_id()
                .as_ref()
                .map(|value| value.to_string()),
            correlation_id: delivery
                .properties
                .correlation_id()
                .as_ref()
                .map(|value| value.to_string()),
            headers: delivery.properties.headers().clone(),
            retry_count,
        }
    }
}

/// Message delivered to the worker.
#[derive(Debug)]
pub struct Message<P> {
    payload: P,
    metadata: DeliveryMetadata,
}

impl<P> Message<P> {
    fn new(payload: P, metadata: DeliveryMetadata) -> Self {
        Self { payload, metadata }
    }

    /// Returns the deserialised payload.
    pub fn payload(&self) -> &P {
        &self.payload
    }

    /// Returns metadata associated with the delivery.
    pub fn metadata(&self) -> &DeliveryMetadata {
        &self.metadata
    }
}

/// Context passed to job handlers.
#[derive(Clone)]
pub struct JobContext<J: Job> {
    publisher: JobPublisher<J>,
    cancellation_token: CancellationToken,
}

impl<J: Job> JobContext<J> {
    fn new(publisher: JobPublisher<J>, cancellation_token: CancellationToken) -> Self {
        Self {
            publisher,
            cancellation_token,
        }
    }

    /// Returns a publisher tied to the same queue and exchange.
    pub fn publisher(&self) -> &JobPublisher<J> {
        &self.publisher
    }

    /// Returns the cancellation token for graceful shutdown.
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

/// Error wrapper used inside job handlers.
#[derive(Debug)]
pub struct JobError {
    inner: AnyhowError,
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for JobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<AnyhowError> for JobError {
    fn from(err: AnyhowError) -> Self {
        Self { inner: err }
    }
}

impl From<PublishError> for JobError {
    fn from(err: PublishError) -> Self {
        Self { inner: err.into() }
    }
}

/// Errors emitted while publishing messages.
#[derive(Debug, Error)]
pub enum PublishError {
    /// Unable to serialise the payload to JSON.
    #[error("failed to serialise job payload: {0}")]
    Serialize(#[from] serde_json::Error),
    /// AMQP-level error while publishing or waiting for confirms.
    #[error("amqp error: {0}")]
    Amqp(#[from] lapin::Error),
}

/// Errors that abort worker start or processing.
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Connection to RabbitMQ failed.
    #[error("failed to connect to RabbitMQ: {0}")]
    Connection(#[source] lapin::Error),
    /// Channel creation failed.
    #[error("failed to create channel: {0}")]
    Channel(#[source] lapin::Error),
    /// Queue / exchange declaration failed.
    #[error("failed to declare topology: {0}")]
    Topology(#[source] lapin::Error),
    /// QoS (prefetch) configuration failed.
    #[error("failed to configure qos: {0}")]
    Qos(#[source] lapin::Error),
    /// Failed to create the consumer.
    #[error("failed to start consumer: {0}")]
    Consumer(#[source] lapin::Error),
    /// The consumer was cancelled by the broker.
    #[error("consumer cancelled by broker")]
    ConsumerCancelled,
    /// Error while consuming deliveries.
    #[error("failed to receive a message: {0}")]
    Consume(#[source] lapin::Error),
    /// Failed to acknowledge a message.
    #[error("failed to acknowledge message: {0}")]
    Ack(#[source] lapin::Error),
    /// Failed to negatively acknowledge a message.
    #[error("failed to negatively acknowledge message: {0}")]
    Nack(#[source] lapin::Error),
    /// Error initialising a publisher for the worker.
    #[error("failed to initialise publisher: {0}")]
    PublisherSetup(#[source] PublishError),
    /// Cron schedule could not be parsed.
    #[error("failed to parse cron pattern `{pattern}`: {source}")]
    ScheduleParse {
        /// Provided cron pattern.
        pattern: String,
        /// Parsing error.
        #[source]
        source: cron::error::Error,
    },
    /// Converting cron instants to durations failed.
    #[error("failed to compute delay for cron schedule")]
    ScheduleTime,
    /// Worker shutdown was requested.
    #[error("worker shutdown requested")]
    Shutdown,
}

/// Configuration for retry with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts before giving up.
    pub max_retries: u32,
    /// Initial delay before the first retry.
    pub initial_interval: Duration,
    /// Maximum delay between retries.
    pub max_interval: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    fn should_retry(&self, retry_count: u32) -> bool {
        retry_count < self.max_retries
    }
}

/// Asynchronous job implementation used by the worker and publisher.
#[async_trait]
pub trait Job: Clone + Send + Sync + 'static {
    /// Payload type transported through RabbitMQ.
    type Payload: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Human readable name primarily for logs.
    fn name(&self) -> &'static str;

    /// Queue & exchange settings.
    fn settings(&self) -> QueueSettings;

    /// Optional consumer tag; defaults to a random UUID-based tag.
    fn consumer_tag(&self) -> Option<String> {
        None
    }

    /// Optional cron expression (in standard five-field format).
    fn cron_schedule(&self) -> Option<&'static str> {
        None
    }

    /// Override to provide a payload when running on a schedule.
    async fn scheduled_payload(&self) -> Result<Option<Self::Payload>, JobError> {
        Ok(None)
    }

    /// Override to customise how scheduled ticks enqueue work.
    async fn on_schedule(&self, publisher: &JobPublisher<Self>) -> Result<(), JobError> {
        if let Some(payload) = self.scheduled_payload().await? {
            publisher.publish(&payload).await?;
        }
        Ok(())
    }

    /// Handle a delivery fetched from the queue.
    async fn handle(
        &self,
        message: &Message<Self::Payload>,
        context: &JobContext<Self>,
    ) -> Result<JobAction, JobError>;

    /// Optional retry configuration for exponential backoff.
    fn retry_config(&self) -> Option<RetryConfig> {
        None
    }

    /// Called before handling a message; useful for tracing/metrics.
    async fn before_handle(&self, _message: &Message<Self::Payload>) {}

    /// Called after handling a message; useful for tracing/metrics.
    async fn after_handle(
        &self,
        _message: &Message<Self::Payload>,
        _result: &Result<JobAction, JobError>,
    ) {
    }

    /// Decide how to react when `handle` returns an error.
    fn on_error(&self, message: &Message<Self::Payload>, error: &JobError) -> FailureStrategy {
        warn!(
            job = self.name(),
            meta = ?message.metadata(),
            error = %error,
            "job handler returned an error"
        );
        if self.retry_config().is_some() {
            FailureStrategy::Retry
        } else {
            FailureStrategy::Requeue
        }
    }

    /// Decide how to react when payload deserialisation fails.
    fn on_deserialization_error(
        &self,
        raw_payload: &[u8],
        error: &serde_json::Error,
    ) -> FailureStrategy {
        warn!(
            job = self.name(),
            error = %error,
            payload_preview = %String::from_utf8_lossy(&raw_payload[..raw_payload.len().min(256)]),
            "failed to deserialize job payload"
        );
        FailureStrategy::Reject
    }
}

/// Typed publisher for a job.
#[derive(Clone)]
pub struct JobPublisher<J: Job> {
    job: Arc<J>,
    /// Connection must be kept alive; dropping it closes all channels.
    _connection: Arc<Connection>,
    channel: Channel,
    settings: QueueSettings,
}

impl<J: Job> JobPublisher<J> {
    /// Connects to RabbitMQ and prepares a publisher for the given job.
    pub async fn connect(job: J) -> Result<Self, PublishError> {
        let job = Arc::new(job);
        let settings = job.settings();
        let connection = Arc::new(
            Connection::connect(&settings.uri, connection_properties())
                .await
                .map_err(PublishError::from)?,
        );
        Self::with_connection(job, connection, settings).await
    }

    async fn with_connection(
        job: Arc<J>,
        connection: Arc<Connection>,
        settings: QueueSettings,
    ) -> Result<Self, PublishError> {
        let channel = connection
            .create_channel()
            .await
            .map_err(PublishError::from)?;
        declare_topology(&channel, &settings)
            .await
            .map_err(PublishError::from)?;
        Ok(Self {
            job,
            _connection: connection,
            channel,
            settings,
        })
    }

    /// Returns the queue name this publisher targets.
    pub fn queue(&self) -> &str {
        &self.settings.queue_name
    }

    /// Publish a payload to the queue/exchange configured by the job.
    pub async fn publish(&self, payload: &J::Payload) -> Result<Confirmation> {
        let body = serde_json::to_vec(payload)?;
        let confirm = self
            .channel
            .basic_publish(
                self.settings.exchange_name(),
                self.settings.routing_key(),
                self.settings.publish_options,
                &body,
                self.settings.publish_properties.clone(),
            )
            .await?
            .await?;
        Ok(confirm)
    }

    /// Access the underlying job definition.
    pub fn job(&self) -> &J {
        &self.job
    }
}

/// Convenience helper: connect, publish once, then drop the connection.
pub async fn publish_once<J: Job>(job: J, payload: &J::Payload) -> Result<Confirmation> {
    let publisher = JobPublisher::connect(job).await?;
    publisher.publish(payload).await
}

/// Background worker runner.
pub struct Worker;

impl Worker {
    /// Runs the worker until the consumer is cancelled or an unrecoverable error occurs.
    pub async fn run<J>(job: J) -> Result<(), WorkerError>
    where
        J: Job,
    {
        Self::run_with_cancellation(job, CancellationToken::new()).await
    }

    /// Runs the worker with a cancellation token for graceful shutdown.
    pub async fn run_with_cancellation<J>(
        job: J,
        cancellation_token: CancellationToken,
    ) -> Result<(), WorkerError>
    where
        J: Job,
    {
        let job = Arc::new(job);
        let settings = job.settings();

        info!(job = job.name(), queue = %settings.queue_name, "starting worker");

        let connection = Arc::new(
            Connection::connect(&settings.uri, connection_properties())
                .await
                .map_err(WorkerError::Connection)?,
        );

        let channel = connection
            .create_channel()
            .await
            .map_err(WorkerError::Channel)?;

        declare_topology(&channel, &settings)
            .await
            .map_err(WorkerError::Topology)?;

        if let Some(prefetch) = settings.prefetch_count {
            channel
                .basic_qos(prefetch, BasicQosOptions::default())
                .await
                .map_err(WorkerError::Qos)?;
        }

        let consumer_tag = job
            .consumer_tag()
            .unwrap_or_else(|| format!("{}-{}", job.name(), Uuid::new_v4()))
            .to_string();

        let consumer = channel
            .basic_consume(
                &settings.queue_name,
                &consumer_tag,
                settings.consume_options,
                settings.consume_arguments.clone(),
            )
            .await
            .map_err(WorkerError::Consumer)?;

        let publisher =
            JobPublisher::with_connection(job.clone(), connection.clone(), settings.clone())
                .await
                .map_err(WorkerError::PublisherSetup)?;

        spawn_scheduler(
            job.clone(),
            publisher.clone(),
            job.cron_schedule(),
            cancellation_token.clone(),
        )?;

        consume_loop(job, consumer, publisher, cancellation_token).await
    }
}

fn spawn_scheduler<J: Job>(
    job: Arc<J>,
    publisher: JobPublisher<J>,
    pattern: Option<&'static str>,
    cancellation_token: CancellationToken,
) -> Result<(), WorkerError> {
    if let Some(pattern) = pattern {
        let schedule =
            pattern
                .parse::<Schedule>()
                .map_err(|source| WorkerError::ScheduleParse {
                    pattern: pattern.to_string(),
                    source,
                })?;
        tokio::spawn(async move {
            let upcoming = schedule.upcoming(Utc);
            for next in upcoming {
                let now = Utc::now();
                let delay = match (next - now).to_std() {
                    Ok(duration) => duration,
                    Err(_) => {
                        warn!(job = job.name(), "skipping past-due cron occurrence");
                        continue;
                    }
                };

                tokio::select! {
                    _ = sleep(delay) => {
                        if let Err(error) = job.on_schedule(&publisher).await {
                            error!(job = job.name(), error = %error, "scheduled run failed");
                        }
                    }
                    _ = cancellation_token.cancelled() => {
                        info!(job = job.name(), "scheduler shutdown requested");
                        break;
                    }
                }

                if cancellation_token.is_cancelled() {
                    break;
                }
            }
        });
    }

    Ok(())
}

async fn consume_loop<J: Job>(
    job: Arc<J>,
    mut consumer: Consumer,
    publisher: JobPublisher<J>,
    cancellation_token: CancellationToken,
) -> Result<(), WorkerError> {
    loop {
        tokio::select! {
            delivery_result = consumer.next() => {
                match delivery_result {
                    Some(Ok(delivery)) => {
                        process_delivery(&job, &delivery, &publisher, &cancellation_token).await?
                    }
                    Some(Err(error)) => return Err(WorkerError::Consume(error)),
                    None => {
                        info!(job = job.name(), "consumer stream ended");
                        return Err(WorkerError::ConsumerCancelled);
                    }
                }
            }
            _ = cancellation_token.cancelled() => {
                info!(job = job.name(), "worker shutdown requested");
                return Err(WorkerError::Shutdown);
            }
        }
    }
}

async fn process_delivery<J: Job>(
    job: &Arc<J>,
    delivery: &Delivery,
    publisher: &JobPublisher<J>,
    cancellation_token: &CancellationToken,
) -> Result<(), WorkerError> {
    let metadata = DeliveryMetadata::from_delivery(delivery);

    match serde_json::from_slice::<J::Payload>(&delivery.data) {
        Ok(payload) => {
            let message = Message::new(payload, metadata);
            let context = JobContext::new(publisher.clone(), cancellation_token.clone());

            job.before_handle(&message).await;

            let span = tracing::info_span!(
                "job_handle",
                job = job.name(),
                message_id = ?message.metadata().message_id,
                retry_count = message.metadata().retry_count,
            );

            let result = job.handle(&message, &context).instrument(span).await;

            job.after_handle(&message, &result).await;

            match result {
                Ok(action) => {
                    handle_action(job, delivery, &message, action, publisher).await?;
                }
                Err(error) => {
                    let strategy = job.on_error(&message, &error);
                    let action = strategy.into_action();
                    error!(
                        job = job.name(),
                        error = %error,
                        action = ?action,
                        retry_count = message.metadata().retry_count,
                        "job handler failed"
                    );
                    handle_action(job, delivery, &message, action, publisher).await?;
                }
            }
        }
        Err(error) => {
            let strategy = job.on_deserialization_error(&delivery.data, &error);
            let action = strategy.into_action();
            error!(
                job = job.name(),
                error = %error,
                action = ?action,
                "failed to deserialize message"
            );
            apply_action(delivery, action).await?;
        }
    }

    Ok(())
}

async fn handle_action<J: Job>(
    job: &Arc<J>,
    delivery: &Delivery,
    message: &Message<J::Payload>,
    action: JobAction,
    publisher: &JobPublisher<J>,
) -> Result<(), WorkerError> {
    match action {
        JobAction::Retry => {
            if let Some(retry_config) = job.retry_config() {
                if retry_config.should_retry(message.metadata().retry_count) {
                    let next_attempt = message.metadata().retry_count + 1;
                    let delay = retry_config
                        .initial_interval
                        .mul_f64(
                            retry_config
                                .multiplier
                                .powi(message.metadata().retry_count as i32),
                        )
                        .min(retry_config.max_interval);

                    info!(
                        job = job.name(),
                        retry_count = next_attempt,
                        delay_ms = delay.as_millis(),
                        "scheduling message retry"
                    );

                    let mut headers = delivery.properties.headers().clone().unwrap_or_default();
                    headers.insert(
                        "x-retry-count".into(),
                        lapin::types::AMQPValue::LongUInt(next_attempt),
                    );

                    let mut properties = delivery.properties.clone();
                    properties = properties.with_headers(headers);

                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .map_err(WorkerError::Ack)?;

                    let publisher = publisher.clone();
                    let job_name = job.name();
                    let data = delivery.data.clone();
                    let exchange_name = publisher.settings.exchange_name().to_string();
                    let routing_key = publisher.settings.routing_key().to_string();
                    let publish_options = publisher.settings.publish_options;

                    tokio::spawn(async move {
                        sleep(delay).await;

                        match publisher
                            .channel
                            .basic_publish(
                                &exchange_name,
                                &routing_key,
                                publish_options,
                                &data,
                                properties,
                            )
                            .await
                        {
                            Ok(confirm) => match confirm.await {
                                Ok(Confirmation::Ack(_)) => {
                                    info!(
                                        job = job_name,
                                        retry_count = next_attempt,
                                        "retry message published successfully"
                                    );
                                }
                                Ok(Confirmation::Nack(_)) | Ok(Confirmation::NotRequested) => {
                                    error!(
                                        job = job_name,
                                        retry_count = next_attempt,
                                        "retry message rejected by broker"
                                    );
                                }
                                Err(error) => {
                                    error!(
                                        job = job_name,
                                        retry_count = next_attempt,
                                        error = %error,
                                        "failed to confirm retry message publish"
                                    );
                                }
                            },
                            Err(error) => {
                                error!(
                                    job = job_name,
                                    retry_count = next_attempt,
                                    error = %error,
                                    "failed to republish message for retry"
                                );
                            }
                        }
                    });
                } else {
                    warn!(
                        job = job.name(),
                        retry_count = message.metadata().retry_count,
                        max_retries = retry_config.max_retries,
                        "max retries exceeded, rejecting message"
                    );
                    apply_action(delivery, JobAction::Reject).await?;
                }
            } else {
                apply_action(delivery, JobAction::Requeue).await?;
            }
        }
        _ => apply_action(delivery, action).await?,
    }

    Ok(())
}

async fn apply_action(delivery: &Delivery, action: JobAction) -> Result<(), WorkerError> {
    match action {
        JobAction::Ack => {
            delivery
                .ack(BasicAckOptions::default())
                .await
                .map_err(WorkerError::Ack)?;
        }
        JobAction::Reject => {
            delivery
                .nack(BasicNackOptions {
                    multiple: false,
                    requeue: false,
                })
                .await
                .map_err(WorkerError::Nack)?;
        }
        JobAction::Requeue => {
            delivery
                .nack(BasicNackOptions {
                    multiple: false,
                    requeue: true,
                })
                .await
                .map_err(WorkerError::Nack)?;
        }
        JobAction::Retry => {
            delivery
                .nack(BasicNackOptions {
                    multiple: false,
                    requeue: true,
                })
                .await
                .map_err(WorkerError::Nack)?;
        }
    }
    Ok(())
}

fn connection_properties() -> ConnectionProperties {
    let mut props = ConnectionProperties::default().with_executor(TokioExecutor::current());
    #[cfg(unix)]
    {
        props = props.with_reactor(TokioReactor::current());
    }
    props
}

async fn declare_topology(channel: &Channel, settings: &QueueSettings) -> Result<(), lapin::Error> {
    if let Some(exchange) = &settings.exchange {
        channel
            .exchange_declare(
                &exchange.name,
                exchange.kind.clone(),
                exchange.declare_options,
                exchange.declare_arguments.clone(),
            )
            .await?;
    }

    channel
        .queue_declare(
            &settings.queue_name,
            settings.declare_options,
            settings.declare_arguments.clone(),
        )
        .await?;

    if let Some(exchange) = &settings.exchange {
        channel
            .queue_bind(
                &settings.queue_name,
                &exchange.name,
                settings.routing_key(),
                settings.bind_options,
                settings.bind_arguments.clone(),
            )
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_settings_defaults() {
        let settings = QueueSettings::new("amqp://localhost", "queue");
        assert_eq!(settings.queue_name, "queue");
        assert_eq!(settings.routing_key(), "queue");
        assert!(settings.prefetch_count.is_some());
        assert!(settings.exchange.is_none());
    }
}
