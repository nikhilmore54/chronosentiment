use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateReference {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub plugin: String,
    pub metadata: HashMap<String, String>,
}
