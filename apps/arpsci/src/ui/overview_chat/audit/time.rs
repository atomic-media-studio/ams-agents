use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn resolve_from_optional_payload(raw: Option<&str>) -> String {
    if let Some(s) = raw {
        let t = s.trim();
        if !t.is_empty() && OffsetDateTime::parse(t, &Rfc3339).is_ok() {
            return t.to_string();
        }
    }
    now_rfc3339()
}
