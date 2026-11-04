use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const DEVICE_COOKIE_NAME: &str = "ultracrew_device_token";
pub const DEVICE_HEADER_NAME: &str = "x-device-token";
pub const SESSIONS_DIR: &str = "data/device_sessions";
pub const COOKIE_MAX_AGE_SECS: i64 = 30 * 86400; // 30 days

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeviceSessionRecord {
    pub device_token: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub revision: u64,
    pub updated_at: String, // ISO 8601
    pub state: Value,
}

#[derive(Debug, Deserialize)]
pub struct SaveDeviceSessionInput {
    pub device_token: Option<String>,
    pub schema_version: Option<u32>,
    pub revision: Option<u64>,
    pub state: Value,
}

/// Token validation to prevent path traversal or invalid characters.
pub fn is_valid_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 128
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Detects whether the incoming request arrived over HTTPS (directly or via reverse proxy).
pub fn is_secure_request(headers: &HeaderMap) -> bool {
    if let Some(proto) = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) {
        proto.eq_ignore_ascii_case("https")
    } else {
        false
    }
}

/// Extracts device token from Cookie header, X-Device-Token header, or returns None.
pub fn extract_device_token(headers: &HeaderMap) -> Option<String> {
    // 1. Check Cookie header (preferred browser mechanism)
    if let Some(cookie_str) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for cookie in cookie_str.split(';') {
            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == DEVICE_COOKIE_NAME {
                let val = parts[1].trim();
                if is_valid_token(val) {
                    return Some(val.to_string());
                }
            }
        }
    }
    // 2. Check X-Device-Token header fallback (explicit programmatic / CLI mechanism)
    if let Some(token_header) = headers.get(DEVICE_HEADER_NAME).and_then(|v| v.to_str().ok()) {
        let val = token_header.trim();
        if is_valid_token(val) {
            return Some(val.to_string());
        }
    }
    None
}

/// Formats a Set-Cookie header string with browser security flags (HttpOnly, SameSite=Lax, and Secure if HTTPS).
pub fn make_cookie_header(token: &str, max_age: i64, is_secure: bool) -> String {
    let secure_flag = if is_secure { "; Secure" } else { "" };
    format!(
        "{}={}; Path=/; Max-Age={}; SameSite=Lax; HttpOnly{}",
        DEVICE_COOKIE_NAME, token, max_age, secure_flag
    )
}

pub fn get_session_file_path(base_dir: &Path, token: &str) -> PathBuf {
    base_dir.join(format!("{}.json", token))
}

/// Reads session record from disk, enforcing expiration (30 days) and valid schema version.
pub fn read_session_from_disk(base_dir: &Path, token: &str) -> Option<DeviceSessionRecord> {
    let file_path = get_session_file_path(base_dir, token);
    if file_path.exists() {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(record) = serde_json::from_str::<DeviceSessionRecord>(&content) {
                // Check schema version support
                if record.schema_version != 1 {
                    let _ = fs::remove_file(&file_path);
                    return None;
                }

                // Check server-side expiration (COOKIE_MAX_AGE_SECS)
                if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(&record.updated_at) {
                    let age = chrono::Utc::now().signed_duration_since(parsed_time);
                    if age.num_seconds() > COOKIE_MAX_AGE_SECS {
                        // Expired session file -> safely delete and ignore
                        let _ = fs::remove_file(&file_path);
                        return None;
                    }
                }

                return Some(record);
            }
        }
    }
    None
}

pub fn write_session_to_disk(
    base_dir: &Path,
    record: &DeviceSessionRecord,
) -> Result<(), std::io::Error> {
    fs::create_dir_all(base_dir)?;
    let file_path = get_session_file_path(base_dir, &record.device_token);
    let json = serde_json::to_string_pretty(record)?;
    fs::write(file_path, json)
}

pub fn delete_session_from_disk(base_dir: &Path, token: &str) -> Result<(), std::io::Error> {
    let file_path = get_session_file_path(base_dir, token);
    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

// ── HTTP Handlers ────────────────────────────────────────────────────────────

pub async fn get_device_session_handler(headers: HeaderMap) -> impl IntoResponse {
    let base_dir = Path::new(SESSIONS_DIR);
    let maybe_token = extract_device_token(&headers);
    let secure = is_secure_request(&headers);

    if let Some(token) = maybe_token {
        if let Some(record) = read_session_from_disk(base_dir, &token) {
            return (
                StatusCode::OK,
                [(
                    header::SET_COOKIE,
                    make_cookie_header(&token, COOKIE_MAX_AGE_SECS, secure),
                )],
                Json(json!({
                    "device_token": record.device_token,
                    "schema_version": record.schema_version,
                    "revision": record.revision,
                    "updated_at": record.updated_at,
                    "state": record.state,
                })),
            );
        }
    }

    // No existing session or session file not found -> issue fresh device token
    let new_token = Uuid::new_v4().to_string();
    (
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            make_cookie_header(&new_token, COOKIE_MAX_AGE_SECS, secure),
        )],
        Json(json!({
            "device_token": new_token,
            "schema_version": 1,
            "revision": 0,
            "updated_at": null,
            "state": null,
        })),
    )
}

pub async fn save_device_session_handler(
    headers: HeaderMap,
    Json(input): Json<SaveDeviceSessionInput>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let base_dir = Path::new(SESSIONS_DIR);
    let secure = is_secure_request(&headers);

    // Priority: input.device_token (if valid) -> header/cookie token -> new UUID
    let token = if let Some(ref t) = input.device_token {
        if is_valid_token(t) {
            t.clone()
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid device_token format"})),
            ));
        }
    } else if let Some(t) = extract_device_token(&headers) {
        t
    } else {
        Uuid::new_v4().to_string()
    };

    // Concurrency / revision protection: check for stale sequence numbers
    let existing_record = read_session_from_disk(base_dir, &token);
    let new_revision = if let Some(ref existing) = existing_record {
        if let Some(incoming_rev) = input.revision {
            if incoming_rev < existing.revision {
                return Err((
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Stale revision: incoming revision is older than stored revision",
                        "stored_revision": existing.revision,
                        "incoming_revision": incoming_rev,
                    })),
                ));
            }
            incoming_rev
        } else {
            existing.revision + 1
        }
    } else {
        input.revision.unwrap_or(1)
    };

    let updated_at = chrono::Utc::now().to_rfc3339();
    let record = DeviceSessionRecord {
        device_token: token.clone(),
        schema_version: input.schema_version.unwrap_or(1),
        revision: new_revision,
        updated_at: updated_at.clone(),
        state: input.state,
    };

    if let Err(e) = write_session_to_disk(base_dir, &record) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to persist device session: {}", e)
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        [(
            header::SET_COOKIE,
            make_cookie_header(&token, COOKIE_MAX_AGE_SECS, secure),
        )],
        Json(json!({
            "status": "saved",
            "device_token": token,
            "schema_version": record.schema_version,
            "revision": record.revision,
            "updated_at": updated_at,
        })),
    ))
}

pub async fn clear_device_session_handler(headers: HeaderMap) -> impl IntoResponse {
    let base_dir = Path::new(SESSIONS_DIR);
    let secure = is_secure_request(&headers);
    if let Some(token) = extract_device_token(&headers) {
        let _ = delete_session_from_disk(base_dir, &token);
    }

    (
        StatusCode::OK,
        [(header::SET_COOKIE, make_cookie_header("", 0, secure))],
        Json(json!({"status": "cleared"})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_token_validation() {
        assert!(is_valid_token("valid-token-123"));
        assert!(is_valid_token("device_token_abc"));
        assert!(is_valid_token("a1b2c3d4-e5f6-7890-abcd-ef1234567890"));
        assert!(!is_valid_token(""));
        assert!(!is_valid_token("../../../etc/passwd"));
        assert!(!is_valid_token("token with spaces"));
        assert!(!is_valid_token("token;semi=colon"));
    }

    #[test]
    fn test_extract_device_token_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("other=123; ultracrew_device_token=my-uuid-token; foo=bar"),
        );
        assert_eq!(
            extract_device_token(&headers),
            Some("my-uuid-token".to_string())
        );
    }

    #[test]
    fn test_extract_device_token_header_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert(
            DEVICE_HEADER_NAME,
            HeaderValue::from_static("header-token-456"),
        );
        assert_eq!(
            extract_device_token(&headers),
            Some("header-token-456".to_string())
        );
    }

    #[test]
    fn test_cookie_header_generation() {
        let cookie = make_cookie_header("test-token", 86400, false);
        assert!(cookie.contains("ultracrew_device_token=test-token"));
        assert!(cookie.contains("Max-Age=86400"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("HttpOnly"));
        assert!(!cookie.contains("Secure"));

        let secure_cookie = make_cookie_header("test-token", 86400, true);
        assert!(secure_cookie.contains("; Secure"));

        let clear_cookie = make_cookie_header("", 0, false);
        assert!(clear_cookie.contains("Max-Age=0"));
    }

    #[test]
    fn test_disk_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("test_session_{}", Uuid::new_v4()));
        let record = DeviceSessionRecord {
            device_token: "test-device-uuid".to_string(),
            schema_version: 1,
            revision: 42,
            updated_at: chrono::Utc::now().to_rfc3339(),
            state: json!({
                "step": 5,
                "staff_count": 20,
                "workflow_mode": "sample"
            }),
        };

        write_session_to_disk(&temp_dir, &record).expect("write failed");
        let loaded = read_session_from_disk(&temp_dir, "test-device-uuid").expect("read failed");
        assert_eq!(loaded, record);
        assert_eq!(loaded.revision, 42);
        assert_eq!(loaded.schema_version, 1);

        delete_session_from_disk(&temp_dir, "test-device-uuid").expect("delete failed");
        assert!(read_session_from_disk(&temp_dir, "test-device-uuid").is_none());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_session_expiry() {
        let temp_dir = std::env::temp_dir().join(format!("test_session_{}", Uuid::new_v4()));
        // Record timestamped 40 days ago (> 30-day cap)
        let old_time = (chrono::Utc::now() - chrono::Duration::days(40)).to_rfc3339();
        let record = DeviceSessionRecord {
            device_token: "expired-device".to_string(),
            schema_version: 1,
            revision: 1,
            updated_at: old_time,
            state: json!({"step": 1}),
        };

        write_session_to_disk(&temp_dir, &record).expect("write failed");
        assert!(read_session_from_disk(&temp_dir, "expired-device").is_none());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
