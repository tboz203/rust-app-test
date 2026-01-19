use std::str::FromStr;

use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set, TransactionTrait,
};

use crate::database::DatabaseConnection;
use crate::entity::{
    Category, CategoryModel, CategoryRelation, Product, ProductActiveModel, ProductCategory,
    ProductCategoryActiveModel, ProductCategoryColumn, ProductCategoryModel, ProductColumn, ProductModel,
    ProductRelation,
};
use crate::error::ApiError;
use crate::models::product::{
    CategoryBrief, CreateProductRequest, ProductListResponse, ProductQueryParams, ProductResponse,
    UpdateProductRequest,
};

/// Repository for product operations
#[derive(Clone)]
pub struct ProductRepository {
    conn: DatabaseConnection,
}

impl ProductRepository {
    /// Create a new product repository
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Create a new product
    pub async fn create_product(&self, req: CreateProductRequest) -> Result<ProductResponse, ApiError> {
        // Start transaction
        let result = self
            .conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Convert BigDecimal to Decimal
                    let price_str = req.price.to_string();
                    let sea_orm_price = Decimal::from_str(&price_str)
                        .map_err(|_| ApiError::internal_server_error("Invalid price format"))?;

                    let product = ProductActiveModel {
                        name: Set(req.name.clone()),
                        description: Set(req.description.clone()),
                        price: Set(sea_orm_price),
                        sku: Set(req.sku.clone()),
                        ..Default::default()
                    };

                    // Insert product
                    let product_model = product.insert(txn).await.map_err(ApiError::Database)?;

                    // Insert product categories
                    for category_id in &req.category_ids {
                        let product_category = ProductCategoryActiveModel {
                            product_id: Set(product_model.id),
                            category_id: Set(*category_id),
                        };

                        product_category.insert(txn).await.map_err(ApiError::Database)?;
                    }

                    // Fetch categories for response
                    let categories = Self::get_product_categories(product_model.id, txn)
                        .await
                        .map_err(ApiError::Database)?;

                    Ok(ProductResponse {
                        id: product_model.id,
                        name: product_model.name,
                        description: product_model.description,
                        price: req.price,
                        sku: product_model.sku,
                        categories,
                        created_at: product_model.created_at,
                        updated_at: product_model.updated_at,
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

    /// Get a product by ID
    pub async fn get_product(&self, id: i32) -> Result<ProductResponse, ApiError> {
        // Find product by ID
        let product = Product::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::not_found_simple("Product not found"))?;

        // Fetch categories
        let categories = Self::get_product_categories(id, &self.conn)
            .await
            .map_err(ApiError::Database)?;

        // Convert price from Sea-ORM Decimal to BigDecimal for the response
        let price_str = product.price.to_string();
        let price =
            BigDecimal::from_str(&price_str).map_err(|_| ApiError::internal_server_error("Invalid price format"))?;

        Ok(ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            price,
            sku: product.sku,
            categories,
            created_at: product.created_at,
            updated_at: product.updated_at,
        })
    }

    /// List products with pagination and filters
    pub async fn list_products(&self, params: ProductQueryParams) -> Result<ProductListResponse, ApiError> {
        let page = params.page();
        let page_size = params.page_size();

        // Build query
        let mut query = Product::find();

        // Apply category filter if present
        if let Some(category_id) = params.category_id {
            // Create a join with product_categories to filter by category
            query = query
                .join(sea_orm::JoinType::InnerJoin, ProductRelation::ProductCategories.def())
                .filter(ProductCategoryColumn::CategoryId.eq(category_id));
        }

        // Count total records for pagination
        let total = query.clone().count(&self.conn).await.map_err(ApiError::Database)?;

        // Apply pagination and ordering
        // Convert i64 values to u64 to match Sea-ORM's expectation
        let offset = ((page - 1) * page_size) as u64;
        let limit = page_size as u64;

        let products = query
            .order_by_asc(ProductColumn::Id)
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await
            .map_err(ApiError::Database)?;

        // Convert to response objects
        let mut product_responses = Vec::with_capacity(products.len());
        for product in products {
            let categories = Self::get_product_categories(product.id, &self.conn)
                .await
                .map_err(ApiError::Database)?;

            // Convert price from Sea-ORM Decimal to BigDecimal for the response
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

        Ok(ProductListResponse {
            products: product_responses,
            total: total as i64, // Convert u64 to i64 to match expected type
            page,
            page_size,
        })
    }

    /// Update a product
    pub async fn update_product(&self, id: i32, req: UpdateProductRequest) -> Result<ProductResponse, ApiError> {
        // Start transaction
        let result = self
            .conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Find product by ID
                    let product = Product::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(ApiError::Database)?
                        .ok_or_else(|| ApiError::not_found_simple("Product not found"))?;

                    // Create active model for update
                    let mut product_active: ProductActiveModel = product.clone().into();

                    // Update fields if provided
                    if let Some(name) = req.name {
                        product_active.name = Set(name);
                    }

                    if let Some(description) = req.description {
                        product_active.description = Set(Some(description));
                    }

                    if let Some(price) = &req.price {
                        let price_str: String = price.to_string();
                        let sea_orm_price: Decimal = Decimal::from_str(&price_str)
                            .map_err(|_| ApiError::internal_server_error("Invalid price format"))?;
                        product_active.price = Set(sea_orm_price);
                    }

                    if let Some(sku) = req.sku {
                        product_active.sku = Set(Some(sku));
                    }

                    // Update the product
                    let product_model = product_active.update(txn).await.map_err(ApiError::Database)?;

                    // Update categories if provided
                    if let Some(category_ids) = &req.category_ids {
                        // Delete existing product categories
                        ProductCategory::delete_many()
                            .filter(ProductCategoryColumn::ProductId.eq(id))
                            .exec(txn)
                            .await
                            .map_err(ApiError::Database)?;

                        // Insert new product categories
                        for category_id in category_ids {
                            let product_category = ProductCategoryActiveModel {
                                product_id: Set(id),
                                category_id: Set(*category_id),
                            };

                            product_category.insert(txn).await.map_err(ApiError::Database)?;
                        }
                    }

                    // Fetch categories for response
                    let categories = Self::get_product_categories(id, txn)
                        .await
                        .map_err(ApiError::Database)?;

                    // Convert price for the response
                    // Use original price if provided, otherwise convert from the model
                    let price = if let Some(p) = req.price {
                        p
                    } else {
                        let price_str = product_model.price.to_string();
                        BigDecimal::from_str(&price_str)
                            .map_err(|_| ApiError::internal_server_error("Invalid price format"))?
                    };

                    Ok(ProductResponse {
                        id: product_model.id,
                        name: product_model.name,
                        description: product_model.description,
                        price,
                        sku: product_model.sku,
                        categories,
                        created_at: product_model.created_at,
                        updated_at: product_model.updated_at,
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

    /// Delete a product
    pub async fn delete_product(&self, id: i32) -> Result<(), ApiError> {
        // Start transaction
        self.conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Check if product exists
                    let product_exists = Product::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(ApiError::Database)?
                        .is_some();

                    if !product_exists {
                        return Err(ApiError::not_found_simple("Product not found"));
                    }

                    // Delete product categories (would be handled by foreign key cascade, but being
                    // explicit)
                    ProductCategory::delete_many()
                        .filter(ProductCategoryColumn::ProductId.eq(id))
                        .exec(txn)
                        .await
                        .map_err(ApiError::Database)?;

                    // Delete the product
                    Product::delete_by_id(id).exec(txn).await.map_err(ApiError::Database)?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(db_err) => ApiError::Database(db_err),
                sea_orm::TransactionError::Transaction(api_err) => api_err,
            })
    }

    /// Helper method to get product categories
    async fn get_product_categories(
        product_id: i32,
        executor: &impl sea_orm::ConnectionTrait,
    ) -> Result<Vec<CategoryBrief>, sea_orm::DbErr> {
        // Using Sea-ORM relations to fetch related categories
        let categories = Category::find()
            .join(sea_orm::JoinType::InnerJoin, CategoryRelation::ProductCategories.def())
            .filter(ProductCategoryColumn::ProductId.eq(product_id))
            .all(executor)
            .await?;

        // Map to CategoryBrief
        let category_briefs = categories
            .into_iter()
            .map(|category| CategoryBrief {
                id: category.id,
                name: category.name,
            })
            .collect();

        Ok(category_briefs)
    }
}
