use common::visibility::{HasVisibility, Visibility};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "paste_posts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub id_str: String,
    pub user_id: i64,
    pub name: String,
    #[sea_orm(default_value = "{}")]
    pub tags: Vec<String>,
    #[sea_orm(default_value = "")]
    pub description: String,
    pub visibility: Visibility,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub last_updated: DateTimeWithTimeZone,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created: DateTimeWithTimeZone,
}
impl HasVisibility for Model {
    fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    fn is_owner(&self, user_id: i64) -> bool {
        self.user_id == user_id as i64
    }
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::user::Entity",
        from = "Column::UserId",
        to = "crate::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
