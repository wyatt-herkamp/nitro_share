pub mod tracing;

use std::path::PathBuf;

use actix_web::web::Data;
use chrono::Duration;
use config_types::{chrono_types::duration::ConfigDuration, size_config::ConfigSize};
use digestible::Digestible;
use entities::user::permissions::Permissions;
use helper_macros::Rules;
use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{images::ImageRules, paste::PasteRules};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind_address: String,
    pub workers: Option<usize>,
    pub tls: Option<TlsConfig>,
    pub database: Database,
    pub session: SessionConfigFull,
    pub image_rules: Data<ImageRules>,
    pub paste_rules: Data<PasteRules>,
    pub site_rules: Data<SiteRules>,
    pub profile_rules: Data<ProfileRules>,
    pub tracing: tracing::TracingConfiguration,
}
#[derive(Debug, Deserialize, Serialize, Rules, Digestible)]
#[serde(default)]
#[typeshare]
pub struct SiteRules {
    #[rule]
    pub allow_registration: bool,
    #[rule]
    pub require_email_verification: bool,
    #[rule]
    pub name: String,
    #[rule(serialize_with = config_types::size_config::serde_impl::serialize_as_u64)]
    #[typeshare(typescript(type = "bigint"))]
    pub max_payload: ConfigSize,
    #[rule]
    pub anonymous_permissions: Permissions,
}

#[derive(Debug, Deserialize, Serialize, Rules, Digestible)]
#[typeshare]
pub struct ProfileRules {
    #[rule]
    pub show_without_login: bool,
}

impl Default for ProfileRules {
    fn default() -> Self {
        Self {
            show_without_login: true,
        }
    }
}

impl Default for SiteRules {
    fn default() -> Self {
        Self {
            allow_registration: true,
            name: "Nitro Share".to_string(),
            max_payload: ConfigSize::new_from_kibibytes(256),
            require_email_verification: false,
            anonymous_permissions: Permissions::new_anonymous(),
        }
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:5312".to_string(),
            workers: None,
            tls: None,
            database: Database::default(),
            session: SessionConfigFull::default(),
            image_rules: Default::default(),
            paste_rules: Default::default(),
            site_rules: Default::default(),
            profile_rules: Default::default(),
            tracing: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Rules, Digestible)]
#[serde(default)]
#[typeshare]
pub struct SessionConfig {
    pub cookie_name: String,
    #[rule]
    pub allow_in_header: bool,
    #[typeshare(typescript(type = "string"))]
    pub session_lifetime: ConfigDuration,
}
impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: "session".to_string(),
            allow_in_header: true,
            session_lifetime: ConfigDuration {
                duration: Duration::days(1),
                unit: config_types::chrono_types::duration::Unit::Days,
            },
        }
    }
}
// TODO. Add SessionCleaner, and session life.
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SessionConfigFull {
    pub manager: SessionManagerConfig,
    #[serde(default, flatten)]
    pub session_config: SessionConfig,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "settings")]
pub enum SessionManagerConfig {
    Memory {
        #[serde(default)]
        start_size: usize,
    },
    Redb {
        file: PathBuf,
    },
}
impl Default for SessionManagerConfig {
    fn default() -> Self {
        // TODO: Default Redb
        Self::Memory { start_size: 100 }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Database {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
impl Default for Database {
    fn default() -> Self {
        Self {
            user: "".to_string(),
            password: "".to_string(),
            host: "localhost:5432".to_string(),
            database: "nitro_share".to_string(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<ConnectOptions> for Database {
    fn into(self) -> ConnectOptions {
        ConnectOptions::new(format!(
            "postgres://{}:{}@{}/{}",
            self.user, self.password, self.host, self.database
        ))
    }
}
