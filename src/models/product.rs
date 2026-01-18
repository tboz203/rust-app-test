use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use validator::Validate;

use crate::validation::validate_decimal_positive;

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub sku: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductCategory {
    pub product_id: i32,
    pub category_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Product name cannot be empty and must be less than 256 characters"
    ))]
    pub name: String,
    pub description: Option<String>,
    #[validate(custom(function = "validate_decimal_positive"))]
    pub price: BigDecimal,
    #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
    pub sku: Option<String>,
    #[validate(length(min = 1, message = "At least one category ID must be provided"))]
    pub category_ids: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Product name cannot be empty and must be less than 256 characters"
    ))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(custom(function = "validate_decimal_positive"))]
    pub price: Option<BigDecimal>,
    #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
    pub sku: Option<String>,
    #[validate(length(
        min = 1,
        message = "At least one category ID must be provided (use null to leave unchanged)"
    ))]
    pub category_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub sku: Option<String>,
    pub categories: Vec<CategoryBrief>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryBrief {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
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
