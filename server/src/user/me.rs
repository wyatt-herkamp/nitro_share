use actix_web::{get, http::StatusCode, web, web::Data, HttpResponse};
use entities::{user::user_responses::User, AuthTokenActiveModel, AuthTokenEntity, AuthTokenModel};
use sea_orm::{ActiveValue, ActiveValue::Set, EntityTrait, InsertResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    responses::{JsonOrError, JsonResponse},
    user::{
        session::{DynSessionManager, Session, SessionManager},
        Authentication,
    },
    utils::{sha256, token},
    DatabaseConnection,
};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(me)
        .service(get_session)
        .service(logout)
        .service(create_token)
        .service(revoke_token);
}
#[utoipa::path(get,
    impl_for=me,
    path = "/api/me",
    responses(
        (status = 200, description = "You are Logged In", body = User),
        (status = 401, description = "You are not logged in")
    ),
    security(
        ("api_key" = [])
    )
)]
#[get("")]
pub async fn me(auth: Authentication) -> JsonResponse<User> {
    JsonResponse::from(Into::<User>::into(auth))
}
#[get("/session")]
pub async fn get_session(auth: Authentication) -> JsonOrError<Session> {
    let Authentication::Session { session, user: _ } = auth else {
        return JsonOrError::Error(HttpResponse::BadRequest().finish());
    };
    JsonOrError::from((session, StatusCode::OK))
}
#[get("/logout")]
pub async fn logout(
    auth: Authentication,
    session_manager: Data<DynSessionManager>,
) -> crate::Result<HttpResponse> {
    let Authentication::Session { session, user: _ } = auth else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    if let Err(error) = session_manager.delete_session(&session.session_id) {
        error!("Failed to delete session: {}", error);
        Ok(HttpResponse::InternalServerError().finish())
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub token_name: String,
}
#[derive(Serialize)]
pub struct NewToken {
    pub token_id: i64,
    pub token_value: String,
}
#[get("/create_token")]
pub async fn create_token(
    create_token: web::Json<CreateTokenRequest>,
    auth: Authentication,
    database: Data<DatabaseConnection>,
) -> crate::Result<HttpResponse> {
    // Only sessions can create tokens
    if !auth.is_session() {
        warn!("Non-session tried to create token");
        return Ok(HttpResponse::BadRequest().finish());
    }
    let create_token = create_token.into_inner();
    let token_name = create_token.token_name;
    let token_value = token::generate_token();
    let hash = sha256::encode_to_string(&token_value);
    if entities::auth_token::database_helpers::does_token_exist(database.as_ref(), &hash).await? {
        warn!("Token collision detected!");
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let token = AuthTokenActiveModel {
        id: ActiveValue::NotSet,
        token_hash: Set(hash),
        user_id: Set(auth.id()),
        token_name: Set(token_name),
        created: Set(chrono::Utc::now().into()),
        revoked: Set(false),
    };

    let model: InsertResult<AuthTokenActiveModel> = AuthTokenEntity::insert(token)
        .exec(database.as_ref())
        .await?;
    let id: i64 = model.last_insert_id;
    info!("Created token");
    Ok(HttpResponse::Ok().json(NewToken {
        token_id: id,
        token_value,
    }))
}
#[get("/revoke_token/{token}")]
pub async fn revoke_token(
    token_id: web::Path<i64>,
    database: Data<DatabaseConnection>,
    auth: Authentication,
) -> crate::Result<HttpResponse> {
    let token: Option<AuthTokenModel> = AuthTokenEntity::find_by_id(token_id.into_inner())
        .one(database.as_ref())
        .await?;
    if let Some(token) = token {
        if token.user_id != auth.as_ref().id {
            warn!("User tried to revoke token that doesn't belong to them");
            return Ok(HttpResponse::BadRequest().finish());
        }
        let mut active_token: AuthTokenActiveModel = token.into();
        active_token.revoked = Set(true);
        AuthTokenEntity::update(active_token)
            .exec(database.as_ref())
            .await?;
        info!("Revoked token");
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
