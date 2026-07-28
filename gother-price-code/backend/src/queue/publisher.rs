//! Queue Publisher
//!
//! Publishes messages to RabbitMQ queues.

use anyhow::Result;
use lapin::{options::*, BasicProperties, Channel};
use serde::Serialize;

use crate::models::ScrapeJobMessage;

/// Publish a scrape job to the queue
pub async fn publish_job(
    channel: &Channel,
    queue_name: &str,
    message: &ScrapeJobMessage,
) -> Result<()> {
    let payload = serde_json::to_vec(message)?;

    channel
        .basic_publish(
            "",
            queue_name,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default()
                .with_delivery_mode(2) // Persistent
                .with_content_type("application/json".into()),
        )
        .await?
        .await?;

    tracing::info!("Published job {} to queue", message.job_id);
    Ok(())
}

/// Generic publish function for any serializable message
pub async fn publish<T: Serialize>(
    channel: &Channel,
    queue_name: &str,
    message: &T,
) -> Result<()> {
    let payload = serde_json::to_vec(message)?;

    channel
        .basic_publish(
            "",
            queue_name,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into()),
        )
        .await?
        .await?;

    Ok(())
}
