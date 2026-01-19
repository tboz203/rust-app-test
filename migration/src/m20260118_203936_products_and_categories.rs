use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create products table
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(pk_auto(Products::Id))
                    .col(string_len(Products::Name, 255).not_null())
                    .col(text(Products::Description))
                    .col(decimal_len(Products::Price, 10, 2).not_null())
                    .col(string_len(Products::Sku, 50).unique_key())
                    .col(
                        timestamp_with_time_zone(Products::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Products::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create categories table
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(pk_auto(Categories::Id))
                    .col(string_len(Categories::Name, 100).not_null().unique_key())
                    .col(text(Categories::Description))
                    .col(
                        timestamp_with_time_zone(Categories::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Categories::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create product_categories join table
        manager
            .create_table(
                Table::create()
                    .table(ProductCategories::Table)
                    .if_not_exists()
                    .col(integer(ProductCategories::ProductId).not_null())
                    .col(integer(ProductCategories::CategoryId).not_null())
                    .primary_key(
                        Index::create()
                            .col(ProductCategories::ProductId)
                            .col(ProductCategories::CategoryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategories::Table, ProductCategories::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategories::Table, ProductCategories::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(ProductCategories::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Define identifier enums for better type safety
#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Name,
    Description,
    Price,
    Sku,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProductCategories {
    Table,
    ProductId,
    CategoryId,
}
