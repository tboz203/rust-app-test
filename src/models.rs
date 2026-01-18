pub mod product;
pub mod category;

pub use product::{Product, CreateProductRequest, UpdateProductRequest, ProductResponse};
pub use category::{Category, CreateCategoryRequest, UpdateCategoryRequest, CategoryResponse};