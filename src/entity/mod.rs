pub mod categories;
pub mod product_categories;
pub mod products;

// Re-export with singular names for readability and domain semantics
pub use categories::ActiveModel as CategoryActiveModel;
pub use categories::Column as CategoryColumn;
pub use categories::Entity as Category;
pub use categories::Model as CategoryModel;
pub use categories::Relation as CategoryRelation;

pub use products::ActiveModel as ProductActiveModel;
pub use products::Column as ProductColumn;
pub use products::Entity as Product;
pub use products::Model as ProductModel;
pub use products::Relation as ProductRelation;

pub use product_categories::ActiveModel as ProductCategoryActiveModel;
pub use product_categories::Column as ProductCategoryColumn;
pub use product_categories::Entity as ProductCategory;
pub use product_categories::Model as ProductCategoryModel;
pub use product_categories::Relation as ProductCategoryRelation;
