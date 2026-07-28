//! Hotel Group Model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Hotel Group - a collection of hotels for batch processing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HotelGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Hotel Group with hotel count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotelGroupWithCount {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub hotel_count: i64,
    pub last_scraped_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new hotel group
#[derive(Debug, Deserialize)]
pub struct CreateHotelGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a hotel group
#[derive(Debug, Deserialize)]
pub struct UpdateHotelGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Hotel Group Member - junction table entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HotelGroupMember {
    pub id: Uuid,
    pub hotel_group_id: Uuid,
    pub hotel_id: Uuid,
}
