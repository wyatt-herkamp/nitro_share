use common::user_types::{Email, Username};
use digestible::Digestible;
use helper_macros::Response;
use sea_orm::{prelude::*, FromQueryResult};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{user::permissions::Permissions, UserModel};

#[derive(
    Clone, Debug, PartialEq, Eq, FromQueryResult, ToSchema, Serialize, Response, Digestible,
)]
#[private]
pub struct User {
    pub id: i64,
    pub name: String,
    #[schema(value_type = String)]
    pub username: Username,
    #[schema(value_type = String)]
    pub email: Email,
    #[schema(value_type = DateTime, nullable )]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time_optional")]
    #[digestible(digest_with = digest_with_hash)]
    pub email_verified: Option<DateTimeWithTimeZone>,
    pub permissions: Permissions,
    #[schema(value_type = DateTime, nullable)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time_optional")]
    #[digestible(digest_with = digest_with_hash)]
    pub password_changed_at: Option<DateTimeWithTimeZone>,
    pub password_reset_required: bool,
    pub banned: bool,
    #[schema(value_type = DateTime)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time")]
    #[digestible(digest_with = digest_with_hash)]
    pub created: DateTimeWithTimeZone,
}
impl From<UserModel> for User {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id,
            name: user.name,
            username: user.username,
            email: user.email,
            email_verified: user.email_verified,
            permissions: user.permissions,
            password_changed_at: user.password_changed_at,
            password_reset_required: user.password_reset_required,
            banned: user.banned,
            created: user.created,
        }
    }
}
#[derive(
    Clone, Debug, PartialEq, Eq, FromQueryResult, ToSchema, Serialize, Response, Digestible,
)]
#[public]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    #[schema(value_type = String)]
    pub username: Username,
    pub banned: bool,
    #[schema(value_type = DateTime)]
    #[serde(serialize_with = "common::serde_chrono::serialize_date_time")]
    #[digestible(digest_with = digest_with_hash)]
    pub created: DateTimeWithTimeZone,
}
