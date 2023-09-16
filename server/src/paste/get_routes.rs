use actix_web::{get, web, web::Data};
use common::visibility::HasVisibility;
use entities::{
    paste::{database_helpers::FileOwnerAndVisibility, Paste},
    user::{permissions::Permissions, user_responses::User},
};

use crate::{
    paste::PasteFile, responses::JsonResponse, user::OptionalAuthentication, DatabaseConnection,
};

#[utoipa::path(get,
    impl_for = get,
    path = "/api/paste/{id}",
    params(
        ("id", description = "The id of the paste")
    ),
    responses(
        (status = 200, description = "Paste Contents", body = Paste),
        (status = 404, description = "Paste Not Found")
    ),
security(
(),
("api_key" = [])
)
)]
#[get("/{id}")]
pub async fn get(
    id: web::Path<String>,
    database: Data<DatabaseConnection>,
) -> crate::Result<JsonResponse<Paste>> {
    Paste::get_by_id_str(database.as_ref(), id.into_inner(), true)
        .await?
        .ok_or(crate::Error::NotFound)
        .map(|paste| JsonResponse::from(paste))
}

#[utoipa::path(get,
impl_for = get_file,
    path = "/api/paste/{id}/file/{file_name}",
    responses(
        (status = 200, description = "File Contents", body = PasteFile),
        (status = 404, description = "File Not Found")
    ),
security(
(),
("api_key" = [])
)
)]
#[get("/{id}/file/{file_name}")]
pub async fn get_file(
    id: web::Path<(String, String)>,
    database: Data<DatabaseConnection>,
    auth: OptionalAuthentication,
) -> crate::Result<JsonResponse<PasteFile>> {
    let (id, file_name) = id.into_inner();
    let file = FileOwnerAndVisibility::get_file_by_string_id_and_file_name(
        database.as_ref(),
        id,
        file_name,
    )
    .await?
    .ok_or(crate::Error::NotFound)?;

    if file.requires_auth() {
        let OptionalAuthentication::Auth(auth) = auth else {
            return Err(crate::Error::Unauthorized);
        };
        let user: User = auth.into();
        if !user.permissions.is_paste_admin() && !file.is_visible_to(user.id) {
            return Err(crate::Error::Unauthorized);
        }
    } else if !<OptionalAuthentication as AsRef<Permissions>>::as_ref(&auth)
        .paste_permissions
        .view_public
    {
        return Err(crate::Error::Unauthorized);
    }

    let file = PasteFile::new(file).await;
    Ok(JsonResponse::from(file))
}
