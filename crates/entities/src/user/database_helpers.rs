use sea_orm::{prelude::*, ConnectionTrait, QuerySelect};

use super::Column as UserColumn;
use crate::{
    auth_token, user::user_responses::User, AuthTokenEntity, AuthTokenModel, UserEntity, UserModel,
};

pub async fn add_user(
    connection: &impl ConnectionTrait,
    active_model: super::ActiveModel,
) -> Result<Option<UserModel>, DbErr> {
    let result = UserEntity::insert(active_model)
        .exec_with_returning(connection)
        .await
        .map(Some);
    if let Err(DbErr::RecordNotInserted) = &result {
        Ok(None)
    } else {
        result
    }
}
#[inline(always)]
pub async fn find_by_id(
    connections: &impl ConnectionTrait,
    id: i64,
) -> Result<Option<User>, DbErr> {
    UserEntity::find()
        .filter(crate::user::Column::Id.eq(id))
        .into_model()
        .one(connections)
        .await
}

#[inline(always)]
pub async fn find_by_login_data(
    username_or_email: &str,
    connection: &impl ConnectionTrait,
) -> Result<Option<UserModel>, DbErr> {
    UserEntity::find()
        .filter(
            crate::user::Column::Username
                .eq(username_or_email)
                .or(crate::user::Column::Email.eq(username_or_email)),
        )
        .one(connection)
        .await
}
/// Uses a Join to get the AuthTokenModel and UserModel
pub async fn get_user_and_auth_token_from_token(
    connections: &impl ConnectionTrait,
    token: &str,
) -> Result<Option<(AuthTokenModel, User)>, DbErr> {
    AuthTokenEntity::find()
        .find_also_related(UserEntity)
        .filter(
            auth_token::Column::TokenHash
                .eq(token)
                .and(auth_token::Column::Revoked.eq(false)),
        )
        .into_model()
        .one(connections)
        .await
        .map(|result| {
            // Map the result to a tuple of (AuthTokenModel, UserModel)
            if let Some((token, user)) = result {
                if let Some(user) = user {
                    Some((token, user))
                } else {
                    None
                }
            } else {
                None
            }
        })
}
#[inline(always)]
pub async fn first_user(connections: &impl ConnectionTrait) -> Result<bool, DbErr> {
    UserEntity::find()
        .select_only()
        .column(UserColumn::Id)
        .count(connections)
        .await
        .map(|count| count == 0)
}
/// Checks if a user exists by checking if the username or email is already in use
pub async fn does_user_exist(
    connections: &impl ConnectionTrait,
    username: &str,
    email: &str,
) -> Result<bool, DbErr> {
    UserEntity::find()
        .filter(
            crate::user::Column::Username
                .eq(username)
                .or(crate::user::Column::Email.eq(email)),
        )
        .count(connections)
        .await
        .map(|count| count > 0)
}
