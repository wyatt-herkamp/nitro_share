use actix_web::{delete, web, web::Data, HttpResponse};
use entities::{
    paste, user::user_responses::User, PasteFileEntity, PastePostEntity, PastePostModel,
};
use sea_orm::{prelude::*, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{user::Authentication, DatabaseConnection};
#[delete("/{id}")]
pub async fn delete(
    auth: Authentication,
    path: web::Path<i64>,
    database: Data<DatabaseConnection>,
) -> crate::Result<HttpResponse> {
    let Some(paste) = PastePostEntity::find_by_id(path.into_inner())
        .one(database.as_ref())
        .await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let user: User = auth.into();
    if paste.user_id != user.id || !user.permissions.admin {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let id = paste.id;
    PastePostEntity::delete(paste.into_active_model())
        .exec(database.as_ref())
        .await?;
    // I am pretty sure this is not needed
    PasteFileEntity::delete_many()
        .filter(paste::file::Column::PostId.eq(id))
        .exec(database.as_ref())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("/{id}/file/{file_name}")]
pub async fn delete_file(
    auth: Authentication,
    path: web::Path<(i64, String)>,
    database: Data<DatabaseConnection>,
) -> crate::Result<HttpResponse> {
    let (id, file_id) = path.into_inner();

    let post: PastePostModel = PastePostEntity::find_by_id(id)
        .one(database.as_ref())
        .await?
        .ok_or(crate::Error::NotFound)?;
    let user: User = auth.into();

    if post.user_id != user.id || !user.permissions.admin {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let result = PasteFileEntity::delete_many()
        .filter(paste::file::Column::PostId.eq(post.id))
        .filter(paste::file::Column::FileName.eq(file_id))
        .exec(database.as_ref())
        .await?;
    if result.rows_affected == 0 {
        return Ok(HttpResponse::NotFound().finish());
    }
    Ok(HttpResponse::NoContent().finish())
}
