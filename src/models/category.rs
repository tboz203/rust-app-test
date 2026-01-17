use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100, message = "Category name cannot be empty and must be less than 101 characters"))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(min = 1, max = 100, message = "Category name must be less than 101 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CategoryWithProductsResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub product_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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