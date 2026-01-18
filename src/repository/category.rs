use std::str::FromStr;

use crate::db::DatabaseConnection;
use crate::entity::{
    Category, CategoryActiveModel, CategoryColumn, CategoryModel, CategoryRelation, Product,
    ProductCategory, ProductCategoryColumn, ProductCategoryModel, ProductColumn, ProductModel,
    ProductRelation,
};
use crate::error::ApiError;
use crate::models::{
    category::{
        CategoryListResponse, CategoryQueryParams, CategoryResponse, CategoryWithProductsResponse,
        CreateCategoryRequest, UpdateCategoryRequest,
    },
    product::ProductResponse,
};

use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::{FixedOffset, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait,
};

/// Repository for category operations
#[derive(Clone)]
pub struct CategoryRepository {
    conn: DatabaseConnection,
}

impl CategoryRepository {
    /// Create a new category repository
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Create a new category
    pub async fn create_category(
        &self,
        req: CreateCategoryRequest,
    ) -> Result<CategoryResponse, ApiError> {
        // Using Sea-ORM's transaction
        let result = self
            .conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Create category active model
                    let category = CategoryActiveModel {
                        name: Set(req.name.clone()),
                        description: Set(req.description.clone()),
                        ..Default::default()
                    };

                    // Insert category
                    let category_model = category.insert(txn).await.map_err(ApiError::Database)?;

                    Ok(CategoryResponse {
                        id: category_model.id,
                        name: category_model.name,
                        description: category_model.description,
                        created_at: category_model.created_at,
                        updated_at: category_model.updated_at,
                    })
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::Database(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })?;

        Ok(result)
    }

    /// Get a category by ID
    pub async fn get_category(&self, id: i32) -> Result<CategoryResponse, ApiError> {
        // Find category by ID
        let category = Category::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::not_found_simple("Category not found"))?;

        // Create the response directly without timezone conversion
        Ok(CategoryResponse {
            id: category.id,
            name: category.name,
            description: category.description,
            created_at: category.created_at,
            updated_at: category.updated_at,
        })
    }

    /// List all categories
    pub async fn list_categories(
        &self,
        params: CategoryQueryParams,
    ) -> Result<CategoryListResponse, ApiError> {
        let categories = Category::find()
            .order_by_asc(CategoryColumn::Name)
            .all(&self.conn)
            .await
            .map_err(ApiError::Database)?;

        let mut category_responses = Vec::with_capacity(categories.len());

        for category in categories {
            // If requested, get product count for each category
            let product_count = if params.include_product_count() {
                self.count_products_in_category(category.id).await?
            } else {
                0 // Default value if not requested
            };

            category_responses.push(CategoryWithProductsResponse {
                id: category.id,
                name: category.name,
                description: category.description,
                product_count: Some(product_count),
                created_at: category.created_at,
                updated_at: category.updated_at,
            });
        }

        Ok(CategoryListResponse {
            categories: category_responses,
        })
    }

    /// Update a category
    pub async fn update_category(
        &self,
        id: i32,
        req: UpdateCategoryRequest,
    ) -> Result<CategoryResponse, ApiError> {
        // Using Sea-ORM's transaction
        let result = self
            .conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Find category by ID
                    let category = Category::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(ApiError::Database)?
                        .ok_or_else(|| ApiError::not_found_simple("Category not found"))?;

                    // Create active model for update
                    let mut category_active: CategoryActiveModel = category.clone().into();

                    // Update fields if provided
                    if let Some(name) = req.name {
                        category_active.name = Set(name);
                    }

                    if let Some(description) = req.description {
                        category_active.description = Set(Some(description));
                    }

                    // Update the category
                    let category_model = category_active
                        .update(txn)
                        .await
                        .map_err(ApiError::Database)?;

                    Ok(CategoryResponse {
                        id: category_model.id,
                        name: category_model.name,
                        description: category_model.description,
                        created_at: category_model.created_at,
                        updated_at: category_model.updated_at,
                    })
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::Database(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })?;

        Ok(result)
    }

    /// Delete a category
    pub async fn delete_category(&self, id: i32) -> Result<(), ApiError> {
        // Using Sea-ORM's transaction
        self.conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Check if category exists
                    let category_exists = Category::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(ApiError::Database)?
                        .is_some();

                    if !category_exists {
                        return Err(ApiError::not_found_simple("Category not found"));
                    }

                    // Delete product categories
                    ProductCategory::delete_many()
                        .filter(ProductCategoryColumn::CategoryId.eq(id))
                        .exec(txn)
                        .await
                        .map_err(ApiError::Database)?;

                    // Delete category
                    Category::delete_by_id(id)
                        .exec(txn)
                        .await
                        .map_err(ApiError::Database)?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::Database(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })
    }

    /// Get products by category ID
    pub async fn get_products_by_category(
        &self,
        category_id: i32,
    ) -> Result<Vec<ProductResponse>, ApiError> {
        // First check if category exists
        let category_exists = Category::find_by_id(category_id)
            .one(&self.conn)
            .await
            .map_err(ApiError::Database)?
            .is_some();

        if !category_exists {
            return Err(ApiError::not_found_simple("Category not found"));
        }

        // Find all products in this category using the product_categories relation
        let products = Product::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                ProductRelation::ProductCategories.def(),
            )
            .filter(ProductCategoryColumn::CategoryId.eq(category_id))
            .all(&self.conn)
            .await
            .map_err(ApiError::Database)?;

        // Convert to product response objects
        let mut product_responses = Vec::with_capacity(products.len());

        for product in products {
            // Get categories for each product
            let categories = self.get_product_categories(product.id).await?;

            // Convert price to BigDecimal
            let price_str = product.price.to_string();
            let price = BigDecimal::from_str(&price_str)
                .map_err(|_| ApiError::internal_server_error("Invalid price format"))?;

            product_responses.push(ProductResponse {
                id: product.id,
                name: product.name,
                description: product.description,
                price,
                sku: product.sku,
                categories,
                created_at: product.created_at,
                updated_at: product.updated_at,
            });
        }

        Ok(product_responses)
    }

    /// Helper method to count products in a category
    async fn count_products_in_category(&self, category_id: i32) -> Result<i64, ApiError> {
        // Count products using the product_categories relation
        let count = ProductCategory::find()
            .filter(ProductCategoryColumn::CategoryId.eq(category_id))
            .count(&self.conn)
            .await
            .map_err(ApiError::Database)?;

        Ok(count as i64)
    }

    /// Helper method to get product categories (used for product responses)
    async fn get_product_categories(
        &self,
        product_id: i32,
    ) -> Result<Vec<crate::models::product::CategoryBrief>, ApiError> {
        // Using Sea-ORM relations to fetch related categories
        let categories = Category::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                CategoryRelation::ProductCategories.def(),
            )
            .filter(ProductCategoryColumn::ProductId.eq(product_id))
            .all(&self.conn)
            .await
            .map_err(ApiError::Database)?;

        // Map to CategoryBrief
        let category_briefs = categories
            .into_iter()
            .map(|category| crate::models::product::CategoryBrief {
                id: category.id,
                name: category.name,
            })
            .collect();

        Ok(category_briefs)
    }
}
