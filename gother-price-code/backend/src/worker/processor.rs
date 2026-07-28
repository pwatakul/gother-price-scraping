//! Worker Processor
//!
//! Main worker loop that processes jobs from the queue.

use anyhow::Result;
use futures::StreamExt;
use std::sync::Arc;

use crate::api::AppState;
use crate::queue::{ack, create_consumer, parse_job_message, reject};
use crate::worker::jobs::process_scrape_job;

/// Run the worker loop
pub async fn run(state: AppState) -> Result<()> {
    let state = Arc::new(state);
    let consumer = create_consumer(&state.rabbitmq, &state.config.rabbitmq_queue_name).await?;

    tracing::info!("Worker started, waiting for jobs...");

    let mut consumer_stream = consumer;

    while let Some(delivery) = consumer_stream.next().await {
        match delivery {
            Ok(delivery) => {
                let state = Arc::clone(&state);
                let channel = state.rabbitmq.clone();
                let delivery_tag = delivery.delivery_tag;

                // Parse the message
                match parse_job_message(&delivery.data) {
                    Ok(message) => {
                        tracing::info!("Processing job: {}", message.job_id);

                        // Process the job
                        match process_scrape_job(&state, message).await {
                            Ok(_) => {
                                tracing::info!("Job completed successfully");
                                if let Err(e) = ack(&channel, delivery_tag).await {
                                    tracing::error!("Failed to ack message: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Job failed: {}", e);
                                if let Err(e) = reject(&channel, delivery_tag).await {
                                    tracing::error!("Failed to reject message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse message: {}", e);
                        if let Err(e) = reject(&channel, delivery_tag).await {
                            tracing::error!("Failed to reject message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Consumer error: {}", e);
            }
        }
    }

    Ok(())
}
