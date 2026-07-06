use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateReference {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub plugin: String,
    pub metadata: HashMap<String, String>,
}
