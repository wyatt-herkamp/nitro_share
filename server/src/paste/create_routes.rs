use std::{collections::HashMap, io::Read};

use actix_multipart::form::{json::Json as JsonForm, tempfile::TempFile, MultipartForm};
use actix_web::{http::header::LOCATION, post, web, web::Data, HttpResponse};
use common::{file_location::FileLocation, paste::file_type::FileType, visibility::Visibility};
use entities::{
    paste,
    paste::{database_helpers::find_post_by_id, generate_post_paste_id},
    PasteFileActiveModel, PasteFileEntity, PastePostActiveModel, PastePostEntity, PastePostModel,
};
use sea_orm::{prelude::*, ActiveValue::Set, EntityTrait, NotSet, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tracing::debug;
use utoipa::{
    openapi::{
        AllOfBuilder, ArrayBuilder, KnownFormat, ObjectBuilder, Ref, RefOr, Schema, SchemaFormat,
        SchemaType,
    },
    ToSchema,
};

use crate::{error::WebsiteError, paste::PasteRules, user::Authentication, DatabaseConnection};
#[derive(Deserialize, Serialize, Default, Debug, ToSchema)]
pub struct NewFile(pub Option<FileType>);
#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(default)]
pub struct NewPaste {
    #[schema(nullable)]
    pub name: String,
    #[schema(nullable)]
    pub description: String,
    #[schema(nullable)]
    pub tags: Vec<String>,
    #[schema(nullable)]
    pub visibility: Visibility,
    #[schema(nullable)]
    pub file_details: ahash::HashMap<String, NewFile>,
}
impl Default for NewPaste {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            description: String::new(),
            tags: vec![],
            visibility: Default::default(),
            file_details: HashMap::default(),
        }
    }
}
#[derive(Debug, MultipartForm)]
pub struct NewPost {
    pub details: Option<JsonForm<NewPaste>>,
    pub files: Vec<TempFile>,
}
impl<'a> ToSchema<'a> for NewPost {
    fn schema() -> (&'a str, RefOr<Schema>) {
        let schema = ObjectBuilder::new()
            .property(
                "details",
                AllOfBuilder::new()
                    .nullable(true)
                    .item(Ref::from_schema_name("NewPaste")),
            )
            .property(
                "files",
                ArrayBuilder::new().items(RefOr::T(
                    ObjectBuilder::new()
                        .schema_type(SchemaType::String)
                        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Binary)))
                        .into(),
                )),
            )
            .into();
        ("NewPost", RefOr::T(schema))
    }
}

/// Handles a file upload to a specific post
///
/// # Parameters
/// - `post_id` - The id of the post to upload to
/// - `database` - The database connection
/// - `file_details` - A function that takes a file name and returns the file details
/// - `upload` - The file to upload
/// - `file_index` - The index of the file in the multipart form
/// - `rules` - The rules for the server
/// # Returns
/// - `Ok(())` - If the file was uploaded successfully
/// - `Err((String, WebsiteError))` - If there was an error uploading the file. String is the file name. WebsiteError is the error
async fn handle_file_upload<D>(
    post_id: i64,
    database: &impl ConnectionTrait,
    file_details: D,
    mut upload: TempFile,
    file_index: usize,
    rules: &PasteRules,
) -> Result<(), FileUploadError>
where
    D: FnOnce(&str) -> NewFile,
{
    let (file_name, details) = if let Some(file_name) = upload.file_name {
        let details = file_details(&file_name);
        (file_name, details)
    } else {
        // No File name. Will be saved as file_{index}. File details will be default
        let name = format!("file_{}", file_index);
        let details = file_details(&name);
        (name, details)
    };
    debug!("Uploading file: {file_name:?}");
    let mime_type = if let Some(file_type) = upload.content_type {
        file_type.to_string()
    } else {
        "text/plain".to_string()
    };
    let mut file_type = details.0.unwrap_or_default();
    file_type.mime_type = mime_type;
    if rules.max_file_size.get_as_bytes() > upload.size {
        return Err((file_name, WebsiteError::ExceedsMaxLength).into());
    }

    let mut as_text = Vec::with_capacity(upload.size);
    if let Err(e) = upload.file.read_to_end(&mut as_text) {
        return Err((file_name, WebsiteError::IoError(e)).into());
    }
    if let Err(e) = simdutf8::basic::from_utf8(&as_text) {
        return Err((file_name, WebsiteError::UTF8ERROR(e)).into());
    }
    let file_location = rules.location.join(post_id.to_string());
    if let Err(e) = tokio::fs::create_dir_all(&file_location).await {
        return Err((file_name, WebsiteError::IoError(e)).into());
    }
    let file_location = file_location.join(file_name.clone());
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_location)
        .await
        .map_err(|e| (file_name.clone(), WebsiteError::IoError(e)))?;
    if let Err(e) = tokio::io::copy(&mut &as_text[..], &mut file).await {
        return Err((file_name, WebsiteError::IoError(e)).into());
    }
    let location = FileLocation::Local {
        location: file_location,
        size: upload.size,
    };

    let file = PasteFileActiveModel {
        id: NotSet,
        post_id: Set(post_id),
        file_name: Set(file_name.clone()),
        file_type: Set(file_type),
        location: Set(location),
        created: NotSet,
    };

    if let Err(e) = PasteFileEntity::insert(file).exec(database).await {
        return Err((file_name, WebsiteError::from(e)).into());
    }
    Ok(())
}
#[derive(Debug, Serialize, ToSchema)]
pub struct FileUploadError {
    pub file_name: String,
    pub error: WebsiteError,
}
impl From<(String, WebsiteError)> for FileUploadError {
    fn from((file_name, error): (String, WebsiteError)) -> Self {
        Self { file_name, error }
    }
}
#[derive(Debug, Serialize, ToSchema)]
pub struct NewPasteResponse {
    pub id: i64,
    pub paste_id: String,
    pub errors: Vec<FileUploadError>,
}
impl NewPasteResponse {
    pub fn new_request(id: i64, paste_id: String, errors: Vec<FileUploadError>) -> HttpResponse {
        HttpResponse::Created()
            .insert_header((LOCATION, format!("/api/paste/{paste_id}")))
            .json(NewPasteResponse {
                id,
                paste_id,
                errors,
            })
    }
}
#[utoipa::path(post,
    impl_for = new,
    path = "/api/paste/new",
    request_body (content = NewPost, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Paste Contents", body = NewPasteResponse)
    ),
security(
("api_key" = [])
)
)]
#[post("/new")]
pub async fn new(
    auth: Authentication,
    upload: MultipartForm<NewPost>,
    database: Data<DatabaseConnection>,
    rules: Data<PasteRules>,
) -> crate::Result<HttpResponse> {
    if !rules.allow_post_creation_without_file && upload.files.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let NewPost { details, files } = upload.into_inner();
    let NewPaste {
        name,
        description,
        tags,
        visibility,
        mut file_details,
    } = details
        .map(|details| details.into_inner())
        .unwrap_or_default();

    let string_id = generate_post_paste_id(database.as_ref()).await?;
    let post = PastePostActiveModel {
        id: NotSet,
        id_str: Set(string_id.clone()),
        name: Set(name),
        description: Set(description),
        visibility: Set(visibility),
        tags: Set(tags),
        user_id: Set(auth.id()),
        created: NotSet,
        last_updated: NotSet,
    };
    let id = PastePostEntity::insert(post)
        .exec(database.as_ref())
        .await?
        .last_insert_id;
    let mut file_errors = Vec::with_capacity(files.len());
    for (index, file) in files.into_iter().enumerate() {
        let file = handle_file_upload(
            id,
            database.as_ref(),
            |file_name| file_details.remove(file_name).unwrap_or_default(),
            file,
            index,
            rules.as_ref(),
        )
        .await;
        if let Err(err) = file {
            file_errors.push(err);
        }
    }
    Ok(NewPasteResponse::new_request(id, string_id, file_errors))
}
#[derive(Debug, MultipartForm)]
pub struct NewFileUpload {
    pub details: Option<JsonForm<ahash::HashMap<String, NewFile>>>,
    pub files: Vec<TempFile>,
}

/// Adds new files to a paste
#[post("/{id}/new/file")]
pub async fn new_file(
    auth: Authentication,
    path: web::Path<i64>,
    upload: MultipartForm<NewFileUpload>,
    database: Data<DatabaseConnection>,
    rules: Data<PasteRules>,
) -> crate::Result<HttpResponse> {
    let post: PastePostModel = find_post_by_id(database.as_ref(), path.into_inner())
        .await?
        .ok_or(WebsiteError::NotFound)?;
    if post.user_id != auth.id() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let NewFileUpload { details, mut files } = upload.into_inner();

    let number_of_files_in_post = PasteFileEntity::find()
        .filter(paste::file::Column::PostId.eq(post.id))
        .count(database.as_ref())
        .await? as usize;
    if files.len() == 1 {
        let file = files.remove(0);
        let details: NewFile = if let Some(details) = details {
            details.0.into_iter().next().unwrap_or_default().1
        } else {
            NewFile::default()
        };
        let file = handle_file_upload(
            post.id,
            database.as_ref(),
            |_| details,
            file,
            number_of_files_in_post,
            rules.as_ref(),
        )
        .await;
        if let Err(err) = file {
            Ok(HttpResponse::BadRequest().json(err))
        } else {
            Ok(NewPasteResponse::new_request(post.id, post.id_str, vec![]))
        }
    } else {
        let mut file_errors = Vec::with_capacity(files.len());

        let mut map = if let Some(details) = details {
            details.into_inner()
        } else {
            HashMap::default()
        };
        for (index, file) in files.into_iter().enumerate() {
            let file = handle_file_upload(
                post.id,
                database.as_ref(),
                |name| map.remove(name).unwrap_or_default(),
                file,
                index + number_of_files_in_post,
                rules.as_ref(),
            )
            .await;
            if let Err(err) = file {
                file_errors.push(err);
            }
        }
        Ok(NewPasteResponse::new_request(
            post.id,
            post.id_str,
            file_errors,
        ))
    }
}
