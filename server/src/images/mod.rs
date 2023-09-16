use std::path::PathBuf;

use actix_web::web;
use config_types::size_config::ConfigSize;
use digestible::Digestible;
use helper_macros::Rules;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

pub fn init(_cfg: &mut web::ServiceConfig) {}
#[derive(Debug, Deserialize, Serialize, Rules, Digestible)]
#[serde(default)]
#[typeshare]
pub struct ImageRules {
    #[rule(serialize_with = config_types::size_config::serde_impl::serialize_as_u64)]
    #[typeshare(typescript(type = "bigint"))]
    pub max_image_size: ConfigSize,
    #[rule]
    pub show_without_login: bool,
    #[digestible(skip)]
    #[typeshare(skip)]
    pub location: PathBuf,
}

impl Default for ImageRules {
    fn default() -> Self {
        Self {
            max_image_size: ConfigSize::new_from_mebibytes(5),
            show_without_login: true,
            location: PathBuf::from("images"),
        }
    }
}
