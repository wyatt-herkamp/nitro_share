use sea_orm_migration::prelude::*;

use crate::sea_orm::Schema;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());
        crate::entities!(schema, manager, entities::AuthTokenEntity);
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuthToken::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AuthToken {
    Table,
}
