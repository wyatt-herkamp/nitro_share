use common::{
    file_location::FileLocation,
    paste::file_type::FileType,
    visibility::{HasVisibility, Visibility},
};
use sea_orm::{
    prelude::*, sea_query::SimpleExpr, ConnectionTrait, FromQueryResult, JoinType, QuerySelect,
    SelectColumns, SelectModel, Selector,
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    paste::{FileColumn, FileRelation, PostColumn},
    PasteFileEntity, PasteFileModel, PastePostEntity, PastePostModel,
};

#[derive(FromQueryResult, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[typeshare]
pub struct PasteAndFileModel {
    pub id_str: String,
    #[typeshare(typescript(type = "bigint"))]
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    #[typeshare(typescript(type = "Date"))]
    pub post_created: DateTimeWithTimeZone,
    #[typeshare(typescript(type = "bigint"))]
    pub post_id: i64,
    #[typeshare(typescript(type = "bigint"))]
    pub file_id: i64,
    pub file_name: String,
    pub file_type: FileType,
    #[typeshare(skip)]
    pub file: FileLocation,
    #[typeshare(typescript(type = "Date"))]
    pub file_created: DateTimeWithTimeZone,
}
impl PasteAndFileModel {
    fn new_request_from_file_with_filter(
        expr: SimpleExpr,
    ) -> Selector<SelectModel<PasteAndFileModel>> {
        Self::new_request_from_file()
            .filter(expr)
            .into_model::<Self>()
    }
    fn new_request_from_file() -> Select<super::file::Entity> {
        PasteFileEntity::find()
            .join(JoinType::InnerJoin, super::file::Relation::Post.def())
            .select_column_as(FileColumn::Created, "file_created")
            .select_column_as(PostColumn::Created, "post_created")
            .select_column_as(PostColumn::Id, "file_id")
    }

    pub async fn get_file_and_post_from_str(
        connections: &impl ConnectionTrait,
        post_id: String,
        file_name: String,
    ) -> Result<Option<Self>, DbErr> {
        PasteAndFileModel::new_request_from_file_with_filter(
            PostColumn::IdStr
                .eq(post_id)
                .and(FileColumn::FileName.eq(file_name)),
        )
        .one(connections)
        .await
    }
    pub async fn get_file_and_post(
        connections: &impl ConnectionTrait,
        post_id: i64,
        file_id: i64,
    ) -> Result<Option<Self>, DbErr> {
        PasteAndFileModel::new_request_from_file_with_filter(
            PostColumn::Id.eq(post_id).and(FileColumn::Id.eq(file_id)),
        )
        .one(connections)
        .await
    }
}

#[derive(FromQueryResult, Clone, Debug, PartialEq, Eq)]
pub struct FileOwnerAndVisibility {
    pub id: i64,
    pub post_id: i64,
    pub file_name: String,
    pub file_type: FileType,
    pub location: FileLocation,
    pub visibility: Visibility,
    pub user_id: i64,
    pub created: DateTimeWithTimeZone,
}
impl HasVisibility for FileOwnerAndVisibility {
    fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    fn is_owner(&self, user_id: i64) -> bool {
        self.user_id == user_id as i64
    }
}
impl FileOwnerAndVisibility {
    pub async fn get_file_by_string_id_and_file_name(
        connections: &impl ConnectionTrait,
        post_id: String,
        file_name: String,
    ) -> Result<Option<FileOwnerAndVisibility>, DbErr> {
        PasteFileEntity::find()
            .column(PostColumn::UserId)
            .column(PostColumn::Visibility)
            .join(JoinType::InnerJoin, FileRelation::Post.def())
            .filter(
                PostColumn::IdStr
                    .eq(post_id)
                    .and(FileColumn::FileName.eq(file_name)),
            )
            .into_model::<FileOwnerAndVisibility>()
            .one(connections)
            .await
    }
}

#[inline(always)]
pub async fn find_post_by_id(
    connections: &impl ConnectionTrait,
    id: i64,
) -> Result<Option<PastePostModel>, DbErr> {
    PastePostEntity::find_by_id(id).one(connections).await
}
#[inline(always)]
pub async fn find_post_by_str_id(
    connections: &impl ConnectionTrait,
    id: String,
) -> Result<Option<PastePostModel>, DbErr> {
    PastePostEntity::find()
        .filter(PostColumn::IdStr.eq(id))
        .one(connections)
        .await
}
#[inline(always)]
pub async fn get_files(
    connections: &impl ConnectionTrait,
    post_id: i64,
) -> Result<Vec<PasteFileModel>, DbErr> {
    PasteFileEntity::find()
        .filter(FileColumn::PostId.eq(post_id))
        .all(connections)
        .await
}

#[inline(always)]
pub async fn does_post_exist(connections: &impl ConnectionTrait, id: i64) -> Result<bool, DbErr> {
    PastePostEntity::find_by_id(id)
        .count(connections)
        .await
        .map(|c| c == 0)
}
