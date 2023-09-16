pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20230123_091026_create_auth_tokens;
mod m20230123_113217_create_uploads;
mod m20230822_185310_init;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230822_185310_init::Migration),
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20230123_091026_create_auth_tokens::Migration),
            Box::new(m20230123_113217_create_uploads::Migration),
        ]
    }
}

macro_rules! entities {
    ($schema:ident,$manager:ident, $($entity_type:path),*) => {
        $(
        {
            let statement = $schema.create_table_from_entity($entity_type);
            $manager.create_table(statement).await?;
        }
        )*
    };
}
pub(crate) use entities;
