use chrono::Utc;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Category name cannot be empty and must be less than 101 characters"
    ))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Category name cannot be empty and must be less than 101 characters"
    ))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Debug, Serialize)]
pub struct CategoryWithProductsResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub product_count: i64,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Debug, Serialize)]
pub struct CategoryListResponse {
    pub categories: Vec<CategoryWithProductsResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryQueryParams {
    pub include_product_count: Option<bool>,
}

impl CategoryQueryParams {
    pub fn include_product_count(&self) -> bool {
        self.include_product_count.unwrap_or(false)
    }
}
