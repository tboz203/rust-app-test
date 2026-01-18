use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

/// Category entity
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Define the relationships
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_category::Entity")]
    ProductCategory,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        super::product_category::Relation::Product.def()
    }
    
    fn via() -> Option<RelationDef> {
        Some(super::product_category::Relation::Category.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}