//! Queue Module
//!
//! RabbitMQ connection and message handling.

pub mod consumer;
pub mod publisher;

pub use consumer::*;
pub use publisher::*;

use anyhow::Result;
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties};

/// Create a new RabbitMQ channel
pub async fn create_channel(rabbitmq_url: &str) -> Result<Channel> {
    let conn = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Declare the scrape jobs queue
    channel
        .queue_declare(
            "scrape_jobs",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    Ok(channel)
}
