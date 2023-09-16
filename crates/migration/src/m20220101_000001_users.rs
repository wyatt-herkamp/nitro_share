use entities::COLLATE_IGNORE_CASE;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Self::table_statement()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

impl Migration {
    pub fn table_statement() -> TableCreateStatement {
        Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(User::Id)
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(User::Name).string().not_null())
            .col(
                ColumnDef::new(User::Username)
                    .text()
                    .extra(COLLATE_IGNORE_CASE)
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(User::Email)
                    .text()
                    .extra(COLLATE_IGNORE_CASE)
                    .not_null()
                    .unique_key(),
            )
            .col(
                ColumnDef::new(User::EmailVerified)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(
                ColumnDef::new(User::Permissions)
                    .json()
                    .not_null()
                    .default("{}"),
            )
            .col(ColumnDef::new(User::Password).string().null())
            .col(
                ColumnDef::new(User::PasswordChangedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(
                ColumnDef::new(User::PasswordResetRequired)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(User::Banned)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(User::Created)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .to_owned()
    }
}
#[derive(DeriveIden)]
pub enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    Name,
    Username,
    Email,
    EmailVerified,
    Permissions,
    Password,
    PasswordChangedAt,
    PasswordResetRequired,
    Banned,
    Created,
}
#[cfg(test)]
mod tests {
    use sea_orm_migration::prelude::PostgresQueryBuilder;

    #[test]
    pub fn print_table_statement() {
        println!(
            "{}",
            super::Migration::table_statement().to_string(PostgresQueryBuilder::default())
        );
    }
}
