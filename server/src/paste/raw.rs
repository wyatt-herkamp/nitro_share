use actix_web::{
    body::BoxBody,
    get, head,
    http::header::{CONTENT_LENGTH, DATE},
    web,
    web::Data,
    CustomizeResponder, HttpRequest, HttpResponse, Responder,
};
use common::{file_location::FileLocation, paste::file_type::FileType};
use entities::paste::database_helpers::FileOwnerAndVisibility;
use sea_orm::prelude::*;

#[utoipa::path(head,
    impl_for = head_file,
    path = "/raw/paste/{id}/file/{file_name}",
    responses(
        (status = 200, description = "File Details"),
    ),
security(
(),
("api_key" = [])
)
)]
#[head("/{id}/file/{file_name}")]
pub async fn head_file(
    request: HttpRequest,
    id: web::Path<(String, String)>,
    database: Data<DatabaseConnection>,
) -> crate::Result<HttpResponse> {
    let (id, file_name) = id.into_inner();
    let file = FileOwnerAndVisibility::get_file_by_string_id_and_file_name(
        database.as_ref(),
        id,
        file_name,
    )
    .await?
    .ok_or(crate::Error::NotFound)?;

    let response = HttpResponse::NoContent()
        .append_header((CONTENT_LENGTH, file.location.file_size()))
        .finish();

    Ok(prepare_response(
        &request,
        response.customize(),
        file.created,
        file.file_type,
    ))
}
#[utoipa::path(get,
    impl_for = get_file,
    path = "/raw/paste/{id}/file/{file_name}", 
    responses(
        (status = 200, content_type = "text/plain", description = "File Contents"),
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
    http_request: HttpRequest,
) -> crate::Result<HttpResponse> {
    let (id, file_name) = id.into_inner();

    let file = FileOwnerAndVisibility::get_file_by_string_id_and_file_name(
        database.as_ref(),
        id,
        file_name,
    )
    .await?
    .ok_or(crate::Error::NotFound)?;
    match file.location {
        FileLocation::Local { location, size: _ } => {
            let response = actix_files::NamedFile::open_async(location)
                .await?
                .customize();
            Ok(prepare_response(
                &http_request,
                response,
                file.created,
                file.file_type,
            ))
        }
    }
}
/// Prepares a response for a file
///
/// # Headers Added
/// `Programming-Language` header is added for the purpose of text highlighting. None if not provided. This is not apart of the HTTP standard
///
/// `Content-Language` is added based on the type given during upload. Will be `None` if not given
///
/// `Content-Type` is added based on the type given during upload. Will be `text/plain` if not given
///
/// `Date` is added based on the time the file was uploaded
fn prepare_response<R: Responder>(
    request: &HttpRequest,
    customize: CustomizeResponder<R>,
    created: DateTimeWithTimeZone,
    file_type: FileType,
) -> HttpResponse<BoxBody> {
    let mut customize = customize.insert_header((DATE, created.to_rfc2822()));
    for (name, value) in file_type.headers_owned_unchecked() {
        customize = customize.insert_header((name.expect("Header Name"), value));
    }
    customize.respond_to(request).map_into_boxed_body()
}
