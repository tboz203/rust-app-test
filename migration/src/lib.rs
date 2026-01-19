pub use sea_orm_migration::prelude::*;

mod m20260118_203936_products_and_categories;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20260118_203936_products_and_categories::Migration,
        )]
    }
}
