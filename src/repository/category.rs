use crate::db::Database;
use crate::entity::{categories, product_categories, products};
use crate::entity::prelude::{Category, Product, ProductCategory};
use crate::error::ApiError;
use crate::models::category::{
    Category as CategoryModel, CategoryListResponse, CategoryQueryParams, CategoryResponse, CategoryWithProductsResponse,
    CreateCategoryRequest, UpdateCategoryRequest,
};
use crate::models::product::{Product as ProductModel, ProductResponse};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, 
    RelationTrait, Set, TransactionTrait, QuerySelect, Condition, PaginatorTrait,
};
use std::str::FromStr;
use sqlx::types::BigDecimal;
use sea_orm::prelude::Decimal;

/// Repository for category operations
#[derive(Clone)]
pub struct CategoryRepository {
    db: Database,
}

impl CategoryRepository {
    /// Create a new category repository
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new category
    pub async fn create_category(
        &self,
        req: CreateCategoryRequest,
    ) -> Result<CategoryResponse, ApiError> {
        let conn = self.db.conn();
        
        // Using Sea-ORM's transaction
        let result = conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Create category active model
                    let category = categories::ActiveModel {
                        name: Set(req.name.clone()),
                        description: Set(req.description.clone()),
                        ..Default::default()
                    };
                    
                    // Insert category
                    let category_model = category
                        .insert(txn)
                        .await
                        .map_err(ApiError::SeaOrmDatabase)?;
                    
                    // Convert timezone-aware datetime to Utc
                    let created_at = chrono::DateTime::<chrono::Utc>::from_utc(
                        category_model.created_at.naive_utc(),
                        chrono::Utc,
                    );
                    let updated_at = chrono::DateTime::<chrono::Utc>::from_utc(
                        category_model.updated_at.naive_utc(),
                        chrono::Utc,
                    );
                    
                    Ok(CategoryResponse {
                        id: category_model.id,
                        name: category_model.name,
                        description: category_model.description,
                        created_at,
                        updated_at,
                    })
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::SeaOrmDatabase(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })?;
            
        Ok(result)
    }

    /// Get a category by ID
    pub async fn get_category(&self, id: i32) -> Result<CategoryResponse, ApiError> {
        let conn = self.db.conn();
        
        // Find category by ID
        let category = Category::find_by_id(id)
            .one(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?
            .ok_or_else(|| ApiError::not_found_simple("Category not found"))?;
        
        // Convert timezone-aware datetime to Utc
        let created_at = chrono::DateTime::<chrono::Utc>::from_utc(
            category.created_at.naive_utc(),
            chrono::Utc,
        );
        let updated_at = chrono::DateTime::<chrono::Utc>::from_utc(
            category.updated_at.naive_utc(),
            chrono::Utc,
        );
        
        Ok(CategoryResponse {
            id: category.id,
            name: category.name,
            description: category.description,
            created_at,
            updated_at,
        })
    }

    /// List all categories
    pub async fn list_categories(
        &self,
        params: CategoryQueryParams,
    ) -> Result<CategoryListResponse, ApiError> {
        let conn = self.db.conn();
        
        let categories = Category::find()
            .order_by_asc(categories::Column::Name)
            .all(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?;
            
        let mut category_responses = Vec::with_capacity(categories.len());
        
        for category in categories {
            // If requested, get product count for each category
            let product_count = if params.include_product_count() {
                self.count_products_in_category(category.id).await?
            } else {
                0 // Default value if not requested
            };
            
            // Convert timezone-aware datetime to Utc
            let created_at = chrono::DateTime::<chrono::Utc>::from_utc(
                category.created_at.naive_utc(),
                chrono::Utc,
            );
            let updated_at = chrono::DateTime::<chrono::Utc>::from_utc(
                category.updated_at.naive_utc(),
                chrono::Utc,
            );
            
            category_responses.push(CategoryWithProductsResponse {
                id: category.id,
                name: category.name,
                description: category.description,
                product_count,
                created_at,
                updated_at,
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
        let conn = self.db.conn();
        
        // Using Sea-ORM's transaction
        let result = conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Find category by ID
                    let category = Category::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(ApiError::SeaOrmDatabase)?
                        .ok_or_else(|| ApiError::not_found_simple("Category not found"))?;
                    
                    // Create active model for update
                    let mut category_active: categories::ActiveModel = category.clone().into();
                    
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
                        .map_err(ApiError::SeaOrmDatabase)?;
                    
                    // Convert timezone-aware datetime to Utc
                    let created_at = chrono::DateTime::<chrono::Utc>::from_utc(
                        category_model.created_at.naive_utc(),
                        chrono::Utc,
                    );
                    let updated_at = chrono::DateTime::<chrono::Utc>::from_utc(
                        category_model.updated_at.naive_utc(),
                        chrono::Utc,
                    );
                    
                    Ok(CategoryResponse {
                        id: category_model.id,
                        name: category_model.name,
                        description: category_model.description,
                        created_at,
                        updated_at,
                    })
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::SeaOrmDatabase(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })?;
            
        Ok(result)
    }

    /// Delete a category
    pub async fn delete_category(&self, id: i32) -> Result<(), ApiError> {
        let conn = self.db.conn();
        
        // Using Sea-ORM's transaction
        conn.transaction(|txn| {
            Box::pin(async move {
                // Check if category exists
                let category_exists = Category::find_by_id(id)
                    .one(txn)
                    .await
                    .map_err(ApiError::SeaOrmDatabase)?
                    .is_some();
                
                if !category_exists {
                    return Err(ApiError::not_found_simple("Category not found"));
                }
                
                // Delete product categories
                product_categories::Entity::delete_many()
                    .filter(product_categories::Column::CategoryId.eq(id))
                    .exec(txn)
                    .await
                    .map_err(ApiError::SeaOrmDatabase)?;
                
                // Delete category
                Category::delete_by_id(id)
                    .exec(txn)
                    .await
                    .map_err(ApiError::SeaOrmDatabase)?;
                
                Ok(())
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(db_err) => ApiError::SeaOrmDatabase(db_err),
            sea_orm::TransactionError::Transaction(api_err) => api_err,
        })
    }

    /// Get products by category ID
    pub async fn get_products_by_category(&self, category_id: i32) -> Result<Vec<ProductResponse>, ApiError> {
        let conn = self.db.conn();
        
        // First check if category exists
        let category_exists = Category::find_by_id(category_id)
            .one(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?
            .is_some();
            
        if !category_exists {
            return Err(ApiError::not_found_simple("Category not found"));
        }
        
        // Find all products in this category using the product_categories relation
        let products = Product::find()
            .join(sea_orm::JoinType::InnerJoin, products::Relation::ProductCategories.def())
            .filter(product_categories::Column::CategoryId.eq(category_id))
            .all(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?;
            
        // Convert to product response objects
        let mut product_responses = Vec::with_capacity(products.len());
        
        for product in products {
            // Get categories for each product
            let categories = self.get_product_categories(product.id).await?;
            
            // Convert price to BigDecimal
            let price_str = product.price.to_string();
            let price = BigDecimal::from_str(&price_str)
                .map_err(|_| ApiError::internal_server_error("Invalid price format"))?;
                
            // Convert timezone-aware datetime to Utc
            let created_at = chrono::DateTime::<chrono::Utc>::from_utc(
                product.created_at.naive_utc(),
                chrono::Utc,
            );
            let updated_at = chrono::DateTime::<chrono::Utc>::from_utc(
                product.updated_at.naive_utc(),
                chrono::Utc,
            );
            
            product_responses.push(ProductResponse {
                id: product.id,
                name: product.name,
                description: product.description,
                price,
                sku: product.sku,
                categories,
                created_at,
                updated_at,
            });
        }
        
        Ok(product_responses)
    }

    /// Helper method to count products in a category
    async fn count_products_in_category(&self, category_id: i32) -> Result<i64, ApiError> {
        let conn = self.db.conn();
        
        // Count products using the product_categories relation
        let count = product_categories::Entity::find()
            .filter(product_categories::Column::CategoryId.eq(category_id))
            .count(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?;
            
        Ok(count as i64)
    }
    
    /// Helper method to get product categories (used for product responses)
    async fn get_product_categories(&self, product_id: i32) -> Result<Vec<crate::models::product::CategoryBrief>, ApiError> {
        let conn = self.db.conn();
        
        // Using Sea-ORM relations to fetch related categories
        let categories = Category::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                categories::Relation::ProductCategories.def(),
            )
            .filter(product_categories::Column::ProductId.eq(product_id))
            .all(conn)
            .await
            .map_err(ApiError::SeaOrmDatabase)?;
        
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