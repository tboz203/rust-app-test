use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub sku: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProductCategory {
    pub product_id: i32,
    pub category_id: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Product name cannot be empty and must be less than 256 characters"))]
    pub name: String,
    pub description: Option<String>,
    #[validate(range(min = 0.0, message = "Price must be a positive number"))]
    pub price: Decimal,
    #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
    pub sku: Option<String>,
    pub category_ids: Vec<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Product name must be less than 256 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0.0, message = "Price must be a positive number"))]
    pub price: Option<Decimal>,
    #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
    pub sku: Option<String>,
    pub category_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub sku: Option<String>,
    pub categories: Vec<CategoryBrief>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CategoryBrief {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Deserialize)]
pub struct ProductQueryParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub category_id: Option<i32>,
}

impl ProductQueryParams {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> i64 {
        self.page_size.unwrap_or(10).min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.page_size()
    }
}