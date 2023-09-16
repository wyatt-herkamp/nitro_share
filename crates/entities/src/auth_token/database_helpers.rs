use sea_orm::{prelude::*, ConnectionTrait};

use crate::{auth_token, AuthTokenEntity, AuthTokenModel};

pub async fn find_by_token_hash(
    connection: &impl ConnectionTrait,
    hash: &str,
) -> Result<Option<AuthTokenModel>, DbErr> {
    AuthTokenEntity::find()
        .filter(auth_token::Column::TokenHash.eq(hash))
        .one(connection)
        .await
}
pub async fn does_token_exist(
    connection: &impl ConnectionTrait,
    hash: &str,
) -> Result<bool, DbErr> {
    AuthTokenEntity::find()
        .filter(auth_token::Column::TokenHash.eq(hash))
        .count(connection)
        .await
        .map(|count| count > 0)
}
