//! Queue Consumer
//!
//! Consumes messages from RabbitMQ queues.

use anyhow::Result;
use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel, Consumer};

use crate::models::ScrapeJobMessage;

/// Create a consumer for the scrape jobs queue
pub async fn create_consumer(channel: &Channel, queue_name: &str) -> Result<Consumer> {
    let consumer = channel
        .basic_consume(
            queue_name,
            "scrape_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(consumer)
}

/// Parse a delivery into a ScrapeJobMessage
pub fn parse_job_message(data: &[u8]) -> Result<ScrapeJobMessage> {
    let message: ScrapeJobMessage = serde_json::from_slice(data)?;
    Ok(message)
}

/// Acknowledge a message
pub async fn ack(channel: &Channel, delivery_tag: u64) -> Result<()> {
    channel
        .basic_ack(delivery_tag, BasicAckOptions::default())
        .await?;
    Ok(())
}

/// Reject a message (won't be requeued)
pub async fn reject(channel: &Channel, delivery_tag: u64) -> Result<()> {
    channel
        .basic_reject(delivery_tag, BasicRejectOptions { requeue: false })
        .await?;
    Ok(())
}

/// Reject a message with requeue
pub async fn requeue(channel: &Channel, delivery_tag: u64) -> Result<()> {
    channel
        .basic_reject(delivery_tag, BasicRejectOptions { requeue: true })
        .await?;
    Ok(())
}
