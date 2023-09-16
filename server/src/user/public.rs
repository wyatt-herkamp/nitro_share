use actix_web::{
    http::{header::CACHE_CONTROL, StatusCode},
    post, web,
    web::Data,
    HttpResponse,
};
use common::user_types::{Email, Username};
use entities::{
    user,
    user::{database_helpers::find_by_login_data, permissions::Permissions, user_responses::User},
    UserActiveModel, UserEntity,
};
use sea_orm::{
    prelude::*, sea_query::SimpleExpr, ActiveValue, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use tracing::log::info;
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    config::SiteRules,
    responses::JsonOrErrorResult,
    state::State,
    user::{
        session::{DynSessionManager, SessionManager},
        LoginResponse,
    },
    utils::password::check_password,
    DatabaseConnection,
};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(register).service(register_check);
}

#[derive(Deserialize)]
#[typeshare]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    login: web::Json<LoginRequest>,
    database: Data<DatabaseConnection>,
    session_manager: Data<DynSessionManager>,
) -> JsonOrErrorResult<LoginResponse> {
    let login: LoginRequest = login.into_inner();
    let user = find_by_login_data(&login.username, database.as_ref()).await?;
    let Some(user) = user else {
        return Ok(HttpResponse::Unauthorized().finish().into());
    };
    let Some(password) = &user.password else {
        return Ok(HttpResponse::Unauthorized().finish().into());
    };
    if !check_password(&login.password, password)? {
        return Ok(HttpResponse::Unauthorized().finish().into());
    }
    let session = session_manager.create_session(user.id)?;

    Ok((
        LoginResponse {
            user: user.into(),
            session: Some(session),
        },
        StatusCode::CREATED,
    )
        .into())
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", content = "content")]
#[typeshare]
pub enum CheckRequest {
    Email {
        #[typeshare(typescript(type = "string"))]
        email: Email,
    },
    Username {
        #[typeshare(typescript(type = "string"))]
        username: Username,
    },
}
impl Into<SimpleExpr> for CheckRequest {
    fn into(self) -> SimpleExpr {
        match self {
            CheckRequest::Email { email } => user::Column::Email.eq(email),
            CheckRequest::Username { username } => user::Column::Username.eq(username),
        }
    }
}
#[utoipa::path(post,
impl_for = register_check,
path = "/api/public/register/check",
request_body (content = CheckRequest, content_type = "application/json"),
responses(
(status = 204, description = "The username or email is not taken"),
(status = 400, description = "The username or email is invalid"),
(status = 409, description = "The username or email is already taken"),
),
)]
#[post("/register/check")]
pub async fn register_check(
    check_request: web::Json<CheckRequest>,
    database: Data<DatabaseConnection>,
) -> crate::Result<HttpResponse> {
    let num_of_users = UserEntity::find()
        .filter(<CheckRequest as Into<SimpleExpr>>::into(
            check_request.into_inner(),
        ))
        .count(database.as_ref())
        .await?;
    if num_of_users >= 1 {
        Ok(HttpResponse::Conflict()
            .insert_header((CACHE_CONTROL, "public, max-age=604800"))
            .finish()
            .into())
    } else {
        Ok(HttpResponse::NoContent().finish().into())
    }
}
#[derive(Deserialize)]
#[typeshare]
pub struct RegisterRequest {
    #[typeshare(typescript(type = "string"))]
    pub username: Username,
    pub password: String,
    #[typeshare(typescript(type = "string"))]
    pub email: Email,
}
#[post("/register")]
pub async fn register(
    register: web::Json<RegisterRequest>,
    database: Data<DatabaseConnection>,
    first_user: Data<State>,
    register_rules: Data<SiteRules>,
) -> crate::Result<HttpResponse> {
    if !register_rules.allow_registration {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let RegisterRequest {
        username,
        password,
        email,
    } = register.into_inner();

    let Some(password) = crate::utils::password::encrypt_password(&password) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let is_first_user = first_user.is_first_user();
    let permissions = if is_first_user {
        info!("Creating first user. This user will have admin permissions.");
        Permissions::new_admin()
    } else {
        Permissions::default()
    };
    let user = UserActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set(username.to_string()),
        username: ActiveValue::Set(username),
        password: ActiveValue::Set(Some(password)),
        email: ActiveValue::Set(email),
        email_verified: ActiveValue::NotSet,
        password_changed_at: ActiveValue::NotSet,
        password_reset_required: ActiveValue::NotSet,
        banned: ActiveValue::Set(false),
        permissions: ActiveValue::Set(permissions),
        created: ActiveValue::NotSet,
    };

    let Some(user) = user::database_helpers::add_user(database.as_ref(), user).await? else {
        return Ok(HttpResponse::Conflict().finish());
    };

    if is_first_user {
        first_user.created_first_user();
    }
    Ok(HttpResponse::Created().json(User::from(user)))
}
