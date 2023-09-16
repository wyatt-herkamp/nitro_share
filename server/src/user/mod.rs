pub mod me;
pub mod middleware;
pub mod profile;
pub mod public;
pub mod session;

use std::fmt::Debug;

use actix_web::{dev::Payload, web::Data, FromRequest, HttpMessage, HttpRequest};
use digestible::Digestible;
use entities::{
    user,
    user::{permissions::Permissions, user_responses::User},
    AuthTokenModel,
};
use futures_util::future::LocalBoxFuture;
use helper_macros::Response;
use serde::Serialize;
use strum::EnumIs;
use tracing::{instrument, Span};
use typeshare::typeshare;

use crate::{
    config::SiteRules, error::WebsiteError, user::session::Session, DatabaseConnection, Error,
};

#[derive(Serialize, Digestible, Debug, Response)]
#[expires(self.session.as_ref().map(|session| session.expires.into()))]
#[private]
#[typeshare]
pub struct LoginResponse {
    user: User,
    #[digestible(skip)]
    session: Option<Session>,
}

impl From<Authentication> for LoginResponse {
    fn from(auth: Authentication) -> Self {
        match auth {
            Authentication::Session { user, session } => Self {
                user,
                session: Some(session),
            },
            Authentication::APIToken { user, .. } => Self {
                user,
                session: None,
            },
        }
    }
}
impl AsRef<Permissions> for OptionalAuthentication {
    fn as_ref(&self) -> &Permissions {
        match self {
            OptionalAuthentication::Auth(auth) => &auth.as_ref().permissions,
            OptionalAuthentication::Anonymous { site_rules } => &site_rules.anonymous_permissions,
        }
    }
}
/// The raw authentication data.
/// Pulled from the middleware.
/// Will be converted to an [Authentication] type.
#[derive(Debug, Clone)]
pub enum AuthenticationRaw {
    Session(Session),
    APIToken(String),
}

/// The authorized user.
/// Containing the user model and any additional data to the authentication method.
#[derive(Debug, Clone, EnumIs)]
pub enum OptionalAuthentication {
    Auth(Authentication),
    Anonymous { site_rules: Data<SiteRules> },
}
impl OptionalAuthentication {
    pub fn as_ref(&self) -> Option<&User> {
        match self {
            OptionalAuthentication::Auth(auth) => Some(auth.as_ref()),
            OptionalAuthentication::Anonymous { .. } => None,
        }
    }
}
#[derive(Debug, Clone, EnumIs)]
pub enum Authentication {
    Session { user: User, session: Session },
    APIToken { user: User, token: AuthTokenModel },
}
impl Into<User> for Authentication {
    fn into(self) -> User {
        match self {
            Authentication::Session { user, .. } => user,
            Authentication::APIToken { user, .. } => user,
        }
    }
}
impl AsRef<User> for Authentication {
    fn as_ref(&self) -> &User {
        match self {
            Authentication::Session { user, .. } => user,
            Authentication::APIToken { user, .. } => user,
        }
    }
}
impl Authentication {
    pub async fn new(
        database: Data<DatabaseConnection>,
        raw: AuthenticationRaw,
    ) -> Result<Option<Authentication>, WebsiteError> {
        let result = match raw {
            AuthenticationRaw::Session(session) => {
                let user =
                    user::database_helpers::find_by_id(database.as_ref(), session.user_id).await?;
                if let Some(user) = user {
                    Ok(Some(Authentication::Session { user, session }))
                } else {
                    Ok(None)
                }
            }
            AuthenticationRaw::APIToken(token) => {
                let token = user::database_helpers::get_user_and_auth_token_from_token(
                    database.as_ref(),
                    &token,
                )
                .await?;
                if let Some((token, user)) = token {
                    Ok(Some(Authentication::APIToken { user, token }))
                } else {
                    Ok(None)
                }
            }
        };
        Span::current().record("auth_result", &format!("{:?}", result.as_ref()));
        result
    }
    /// Copies the id from the UserModel.
    pub fn id(&self) -> i64 {
        match self {
            Authentication::Session { user, .. } => user.id,
            Authentication::APIToken { user, .. } => user.id,
        }
    }
}

impl FromRequest for OptionalAuthentication {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;
    #[instrument(skip(req))]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<AuthenticationRaw>().cloned();
        Span::current().record("auth", &format!("{:?}", model.as_ref()));
        let site_rules = req
            .app_data::<Data<SiteRules>>()
            .expect("Unable to get SiteRules Ref")
            .clone();
        if let Some(model) = model {
            let database = req
                .app_data::<Data<DatabaseConnection>>()
                .expect("Unable to get Database Ref")
                .clone();
            return Box::pin(async move {
                return if let Some(auth) = Authentication::new(database, model).await? {
                    Ok(OptionalAuthentication::Auth(auth))
                } else {
                    Ok(OptionalAuthentication::Anonymous {
                        site_rules: site_rules.clone(),
                    })
                };
            });
        }
        Box::pin(async move {
            Ok(OptionalAuthentication::Anonymous {
                site_rules: site_rules.clone(),
            })
        })
    }
}

impl FromRequest for Authentication {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    /// Extracts the authentication data from the request.
    #[instrument(skip(req))]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let model = req.extensions_mut().get::<AuthenticationRaw>().cloned();
        Span::current().record("auth", &format!("{:?}", model.as_ref()));
        if let Some(model) = model {
            let database = req
                .app_data::<Data<DatabaseConnection>>()
                .expect("Unable to get Database Ref")
                .clone();
            return Box::pin(async move {
                let model = Authentication::new(database, model).await?;
                if let Some(model) = model {
                    return Ok(model);
                }
                Err(Error::Unauthorized)
            });
        }
        Box::pin(async move { Err(Error::Unauthorized) })
    }
}
