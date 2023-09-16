use common::{paste::file_type::FileType, visibility::Visibility};
use entities::{
    image::ImagePermissions,
    paste::{Paste, PastePermissions},
    user::{
        permissions::{Permissions, UserPermissions},
        user_responses::{User, UserProfile},
    },
};
use utoipa::{
    openapi::{
        security::{Http, HttpAuthScheme, SecurityScheme},
        Components, ComponentsBuilder, InfoBuilder, OpenApiBuilder, Paths, PathsBuilder,
    },
    OpenApi,
};

use crate::{
    paste::{
        create_routes as paste_create_routes,
        create_routes::{FileUploadError, NewFile, NewPaste, NewPasteResponse, NewPost},
        get_routes as paste_get_routes, raw as paste_raw, PasteFile,
    },
    user::{me, public, public::CheckRequest},
};

pub const API_KEY: &str = "api_key";
pub struct ApiDoc;
impl ApiDoc {
    fn components() -> Components {
        ComponentsBuilder::new()
            .schema_from::<UserPermissions>()
            .schema_from::<PastePermissions>()
            .schema_from::<ImagePermissions>()
            .schema_from::<Permissions>()
            .schema_from::<User>()
            .schema_from::<UserProfile>()
            .schema_from::<Paste>()
            .schema_from::<Visibility>()
            .schema_from::<NewFile>()
            .schema_from::<NewPaste>()
            .schema_from::<NewPost>()
            .schema_from::<PasteFile>()
            .schema_from::<NewPasteResponse>()
            .schema_from::<FileType>()
            .schema_from::<FileUploadError>()
            .schema_from::<CheckRequest>()
            .security_scheme(
                API_KEY,
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
            .build()
    }
    fn paths() -> Paths {
        PathsBuilder::new()
            .path_from::<me::me>()
            .path_from::<public::register_check>()
            .path_from::<paste_get_routes::get>()
            .path_from::<paste_get_routes::get_file>()
            .path_from::<paste_raw::get_file>()
            .path_from::<paste_raw::head_file>()
            .path_from::<paste_create_routes::new>()
            .build()
    }
}
impl OpenApi for ApiDoc {
    fn openapi() -> utoipa::openapi::OpenApi {
        OpenApiBuilder::new()
            .info(
                InfoBuilder::new()
                    .title("nitro_share")
                    .version(env!("CARGO_PKG_VERSION"))
                    .description(option_env!("CARGO_PKG_DESCRIPTION"))
                    .license(None),
            )
            .paths(Self::paths())
            .components(Some(Self::components()))
            .build()
    }
}
