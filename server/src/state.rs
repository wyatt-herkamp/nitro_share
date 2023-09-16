use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use digestible::Digestible;
use helper_macros::Rules;
use serde::Serializer;
use typeshare::typeshare;

#[derive(Debug, Rules, Digestible)]
#[typeshare]
pub struct State {
    #[rule]
    #[digestible(digest_with = atomics::digest_relaxed)]
    #[typeshare(typescript(type = "boolean"))]
    pub first_user: AtomicBool,
    #[rule(serialize_with = serialize_date_time)]
    #[digestible(digest_with = digest_with_hash)]
    #[typeshare(typescript(type = "Date"))]
    pub started_at: DateTime<Utc>,
}

impl State {
    pub fn new(is_first_user: bool) -> Self {
        Self {
            first_user: AtomicBool::new(is_first_user),
            started_at: Utc::now(),
        }
    }
    pub fn is_first_user(&self) -> bool {
        self.first_user.load(Ordering::Relaxed)
    }
    pub fn created_first_user(&self) {
        self.first_user.store(false, Ordering::Relaxed);
    }
}

pub fn serialize_date_time<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.to_rfc2822())
}
