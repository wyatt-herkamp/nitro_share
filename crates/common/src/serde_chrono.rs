use chrono::{DateTime, FixedOffset};
use serde::Serializer;

pub fn serialize_date_time<S>(
    date: &DateTime<FixedOffset>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.to_rfc2822())
}
pub fn serialize_date_time_optional<S>(
    date: &Option<DateTime<FixedOffset>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => serializer.serialize_str(&date.to_rfc2822()),
        None => serializer.serialize_none(),
    }
}
