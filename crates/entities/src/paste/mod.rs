use digestible::Digestible;
use helper_macros::Response;
use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use sea_orm::{
    prelude::*, sea_query::SimpleExpr, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

pub use crate::paste::{
    file::{Column as FileColumn, Relation as FileRelation},
    post::{Column as PostColumn, Relation as PostRelation},
};
use crate::{PasteFileEntity, PastePostEntity};

pub mod database_helpers;
pub mod file;
pub mod post;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema, Digestible, Response)]
#[typeshare]
pub struct Paste {
    #[typeshare(typescript(type = "bigint"))]
    pub id: i64,
    pub id_str: String,
    #[typeshare(typescript(type = "bigint"))]
    pub user_id: i64,
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    #[schema(value_type = DateTime)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time")]
    #[digestible(digest_with = digest_with_hash)]
    #[typeshare(typescript(type = "Date"))]
    pub last_updated: DateTimeWithTimeZone,
    #[schema(value_type = DateTime)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time")]
    #[digestible(digest_with = digest_with_hash)]
    #[typeshare(typescript(type = "Date"))]
    pub created: DateTimeWithTimeZone,
    /// Not included in a response that does not include files
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}
impl Paste {
    pub async fn get(
        connections: &impl ConnectionTrait,
        filter: SimpleExpr,
        get_files: bool,
    ) -> Result<Option<Self>, DbErr> {
        let result = PastePostEntity::find()
            .filter(filter)
            .one(connections)
            .await?;
        if let Some(result) = result {
            let files: Vec<String> = if !get_files {
                PasteFileEntity::find()
                    .select_only()
                    .column(file::Column::FileName)
                    .filter(file::Column::PostId.eq(result.id))
                    .into_tuple()
                    .all(connections)
                    .await?
            } else {
                vec![]
            };
            Ok(Some(Self {
                id: result.id,
                id_str: result.id_str,
                user_id: result.user_id,
                name: result.name,
                tags: result.tags,
                description: result.description,
                last_updated: result.last_updated,
                files,
                created: result.created,
            }))
        } else {
            Ok(None)
        }
    }
    pub async fn get_by_id(
        connections: &impl ConnectionTrait,
        id: i64,
        get_files: bool,
    ) -> Result<Option<Self>, DbErr> {
        Self::get(connections, PostColumn::Id.eq(id), get_files).await
    }

    pub async fn get_by_id_str(
        connections: &impl ConnectionTrait,
        id_str: String,
        get_files: bool,
    ) -> Result<Option<Self>, DbErr> {
        Self::get(connections, PostColumn::IdStr.eq(id_str), get_files).await
    }
}
pub async fn generate_post_paste_id(connections: &impl ConnectionTrait) -> Result<String, DbErr> {
    let mut rand = StdRng::from_entropy();
    loop {
        let post_id: String = (0..8).map(|_| rand.sample(Alphanumeric) as char).collect();
        if PastePostEntity::find()
            .filter(post::Column::IdStr.eq(post_id.clone()))
            .count(connections)
            .await?
            == 0
        {
            return Ok(post_id);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Digestible, ToSchema)]
#[serde(default)]
#[typeshare]
pub struct PastePermissions {
    pub create: bool,
    /// View and Delete Other Images
    pub admin: bool,
    pub view_public: bool,
}
impl Default for PastePermissions {
    fn default() -> Self {
        Self {
            create: true,
            admin: false,
            view_public: true,
        }
    }
}
