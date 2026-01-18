use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

/// Product entity
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub sku: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Define the relationships
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_category::Entity")]
    ProductCategory,
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        super::product_category::Relation::Category.def()
    }
    
    fn via() -> Option<RelationDef> {
        Some(super::product_category::Relation::Product.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}