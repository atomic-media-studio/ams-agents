use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
pub struct AuditRecord {
    pub schema_version: u32,
    pub kind: &'static str,
    pub ts: String,
    pub conversation_id: String,
    pub request_id: String,
    pub event_id: String,
    pub details: serde_json::Value,
}
