pub mod database_helpers;
pub mod permissions;
pub mod user_responses;

use common::user_types::{Email, Username};
use sea_orm::entity::prelude::*;

use crate::user::permissions::Permissions;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub name: String,
    pub username: Username,
    pub email: Email,
    pub email_verified: Option<DateTimeWithTimeZone>,
    pub permissions: Permissions,
    pub password: Option<String>,
    pub password_changed_at: Option<DateTimeWithTimeZone>,
    pub password_reset_required: bool,
    pub banned: bool,
    pub created: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::auth_token::Entity")]
    AuthToken,
}

impl Related<crate::auth_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}
