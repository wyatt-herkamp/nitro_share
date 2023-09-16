pub mod auth_token;
pub mod image;
pub mod paste;
pub mod user;

pub use auth_token::{
    ActiveModel as AuthTokenActiveModel, Entity as AuthTokenEntity, Model as AuthTokenModel,
};
use common::visibility::Visibility;
pub use image::{
    image::{
        ActiveModel as ImageFileActiveModel, Entity as ImageFileEntity, Model as ImageFileModel,
    },
    post::{
        ActiveModel as ImagePostActiveModel, Entity as ImagePostEntity, Model as ImagePostModel,
    },
};
pub use paste::{
    file::{
        ActiveModel as PasteFileActiveModel, Entity as PasteFileEntity, Model as PasteFileModel,
    },
    post::{
        ActiveModel as PastePostActiveModel, Entity as PastePostEntity, Model as PastePostModel,
    },
};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
pub use user::{ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel};

pub static COLLATE_IGNORE_CASE: &str = "COLLATE ignoreCase";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromQueryResult)]
pub struct OwnerAndVisibility {
    pub user_id: u64,
    pub visibility: Visibility,
}
