use std::fmt::{Debug, Display};

use chrono::{DateTime, Duration, FixedOffset};
use digestible::Digestible;
use http::HeaderValue;

#[derive(Debug)]
pub enum CacheControlParams {
    MaxAge(Duration),
    Public,
    Private,
}
impl Display for CacheControlParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheControlParams::MaxAge(duration) => write!(f, "max-age={}", duration.num_seconds()),
            CacheControlParams::Public => write!(f, "public"),
            CacheControlParams::Private => write!(f, "private"),
        }
    }
}
pub trait ToCacheControl {
    fn to_cache_control(&self) -> HeaderValue;
}

impl ToCacheControl for Vec<CacheControlParams> {
    fn to_cache_control(&self) -> HeaderValue {
        let mut cache_control = String::new();
        for param in self {
            cache_control.push_str(&param.to_string());
            cache_control.push(',');
        }
        cache_control.pop();
        HeaderValue::from_str(&cache_control).unwrap()
    }
}

pub trait ResponseType: Digestible + Debug {
    fn last_modified(&self) -> Option<DateTime<FixedOffset>> {
        None
    }

    fn expires(&self) -> Option<DateTime<FixedOffset>> {
        None
    }

    fn cache_control_params(&self) -> Vec<CacheControlParams> {
        vec![]
    }
}
