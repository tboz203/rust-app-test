pub mod categories;
pub mod product_categories;
pub mod products;

// Re-export with singular names for readability and domain semantics
pub use categories::{
    ActiveModel as CategoryActiveModel, Column as CategoryColumn, Entity as Category, Model as CategoryModel,
    Relation as CategoryRelation,
};
pub use product_categories::{
    ActiveModel as ProductCategoryActiveModel, Column as ProductCategoryColumn, Entity as ProductCategory,
    Model as ProductCategoryModel, Relation as ProductCategoryRelation,
};
pub use products::{
    ActiveModel as ProductActiveModel, Column as ProductColumn, Entity as Product, Model as ProductModel,
    Relation as ProductRelation,
};
