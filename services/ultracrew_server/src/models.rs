use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecisionCase {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub schedule: Option<std::collections::HashMap<String, Vec<String>>>,
    pub metadata: Option<serde_json::Value>,
}

impl DecisionCase {
    pub fn new(
        title: String,
        description: String,
        schedule: Option<std::collections::HashMap<String, Vec<String>>>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        DecisionCase {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            created_at: now,
            updated_at: now,
            schedule,
            metadata,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Recommendation {
    pub id: String,
    pub decision_case_id: String,
    pub action: String,
    pub explanation: String,
    pub created_at: u64,
}

impl Recommendation {
    pub fn new(decision_case_id: String, action: String, explanation: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Recommendation {
            id: Uuid::new_v4().to_string(),
            decision_case_id,
            action,
            explanation,
            created_at: now,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleVersion {
    pub version_id: String,
    pub decision_case_id: String,
    pub schedule: std::collections::HashMap<String, Vec<String>>, // nurse_id -> daily shifts
    pub author: String,
    pub timestamp: u64,
    pub description: Option<String>,
    pub status: String, // Draft | Committed
}

impl ScheduleVersion {
    pub fn new(
        decision_case_id: String,
        schedule: std::collections::HashMap<String, Vec<String>>,
        author: String,
        description: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        ScheduleVersion {
            version_id: Uuid::new_v4().to_string(),
            decision_case_id,
            schedule,
            author,
            timestamp: now,
            description,
            status: "Draft".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecisionLog {
    pub log_id: String,
    pub decision_case_id: String,
    pub action: String,
    pub timestamp: u64,
    pub details: Option<serde_json::Value>,
}

impl DecisionLog {
    pub fn new(
        decision_case_id: String,
        action: String,
        details: Option<serde_json::Value>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        DecisionLog {
            log_id: Uuid::new_v4().to_string(),
            decision_case_id,
            action,
            timestamp: now,
            details,
        }
    }
}
