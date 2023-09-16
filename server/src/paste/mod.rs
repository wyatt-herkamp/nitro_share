use std::path::PathBuf;

use actix_web::web;
use common::{file_location::FileLocation, paste::file_type::FileType};
use config_types::size_config::ConfigSize;
use digestible::Digestible;
use entities::paste::database_helpers::FileOwnerAndVisibility;
use helper_macros::{Response, Rules};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

pub mod create_routes;
mod delete_routes;
pub mod get_routes;
pub mod raw;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_routes::get)
        .service(get_routes::get_file)
        .service(create_routes::new)
        .service(create_routes::new_file)
        .service(delete_routes::delete)
        .service(delete_routes::delete_file);
}
pub fn init_raw(cfg: &mut web::ServiceConfig) {
    cfg.service(raw::get_file).service(raw::head_file);
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema, Digestible, Response)]
pub struct PasteFile {
    pub id: i64,
    pub post_id: i64,
    pub file_name: String,
    pub file_type: FileType,
    pub size: u64,
    pub file: String,
    #[schema(value_type = DateTime)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time")]
    #[digestible(digest_with = digest_with_hash)]
    pub created: DateTimeWithTimeZone,
}
impl PasteFile {
    pub async fn new(model: FileOwnerAndVisibility) -> Self {
        let (size, content) = match model.location {
            FileLocation::Local { location, size } => {
                let content = tokio::fs::read_to_string(location).await.unwrap();
                let size = size as u64;
                (size, content)
            }
        };
        Self {
            id: model.id,
            post_id: model.post_id,
            file_name: model.file_name,
            file_type: model.file_type,
            size,
            file: content,
            created: model.created,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Rules, Digestible)]
#[serde(default)]
#[typeshare]
pub struct PasteRules {
    #[rule(serialize_with = config_types::size_config::serde_impl::serialize_as_u64)]
    #[typeshare(typescript(type = "bigint"))]
    pub max_file_size: ConfigSize,
    #[rule(serialize_with_option = config_types::size_config::serde_impl::serialize_as_u64)]
    #[typeshare(typescript(type = "bigint"))]
    pub max_paste_size: Option<ConfigSize>,
    #[rule()]
    pub show_without_login: bool,
    #[rule()]
    pub allow_file_updates: bool,
    #[rule()]
    pub allow_post_creation_without_file: bool,
    #[digestible(skip)]
    #[typeshare(skip)]
    pub location: PathBuf,
}
impl Default for PasteRules {
    fn default() -> Self {
        Self {
            max_file_size: ConfigSize::new_from_mebibytes(1),
            max_paste_size: None,
            show_without_login: true,
            allow_file_updates: true,
            allow_post_creation_without_file: true,
            location: PathBuf::from("pastes"),
        }
    }
}
