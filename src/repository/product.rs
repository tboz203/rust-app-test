use crate::db::Database;
use crate::error::ApiError;
use crate::models::product::{
    CategoryBrief, CreateProductRequest, Product, ProductCategory, ProductListResponse,
    ProductQueryParams, ProductResponse, UpdateProductRequest,
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use sqlx::{postgres::PgRow, query_builder::QueryBuilder, Postgres, Row};

/// Repository for product operations
#[derive(Clone)]
pub struct ProductRepository {
    db: Database,
}

impl ProductRepository {
    /// Create a new product repository
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create a new product
    pub async fn create_product(&self, req: CreateProductRequest) -> Result<ProductResponse, ApiError> {
        self.db
            .transaction::<_, _, ProductResponse, ApiError>(|tx| async move {
                // Insert product
                let product = sqlx::query_as!(
                    Product,
                    r#"
                    INSERT INTO products (name, description, price, sku)
                    VALUES ($1, $2, $3, $4)
                    RETURNING *
                    "#,
                    req.name,
                    req.description,
                    req.price,
                    req.sku
                )
                .fetch_one(&mut **tx)
                .await?;

                // Insert product categories
                if !req.category_ids.is_empty() {
                    let mut query_builder: QueryBuilder<Postgres> =
                        QueryBuilder::new("INSERT INTO product_categories (product_id, category_id) ");

                    query_builder.push_values(req.category_ids.iter(), |mut b, category_id| {
                        b.push_bind(product.id).push_bind(category_id);
                    });

                    query_builder.build().execute(&mut **tx).await?;
                }

                // Fetch categories for response
                let categories = self.get_product_categories(product.id, &mut **tx).await?;

                Ok(ProductResponse {
                    id: product.id,
                    name: product.name,
                    description: product.description,
                    price: product.price,
                    sku: product.sku,
                    categories,
                    created_at: product.created_at,
                    updated_at: product.updated_at,
                })
            })
            .await
    }

    /// Get a product by ID
    pub async fn get_product(&self, id: i32) -> Result<ProductResponse, ApiError> {
        let product = sqlx::query_as!(
            Product,
            r#"
            SELECT * FROM products WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db.pool())
        .await?
        .ok_or_else(|| ApiError::not_found("Product", id))?;

        let categories = self.get_product_categories(id, &self.db.pool()).await?;

        Ok(ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            price: product.price,
            sku: product.sku,
            categories,
            created_at: product.created_at,
            updated_at: product.updated_at,
        })
    }

    /// List products with pagination and filters
    pub async fn list_products(
        &self,
        params: ProductQueryParams,
    ) -> Result<ProductListResponse, ApiError> {
        // Base query with pagination
        let mut conditions = Vec::new();
        let mut param_index = 1;
        
        // Add category filter if provided
        let category_id_param = params.category_id;
        if let Some(category_id) = category_id_param {
            conditions.push(format!(
                "id IN (SELECT product_id FROM product_categories WHERE category_id = ${param_index})"
            ));
            param_index += 1;
        }

        // Build WHERE clause if we have conditions
        let where_clause = if !conditions.is_empty() {
            format!("WHERE {}", conditions.join(" AND "))
        } else {
            String::new()
        };

        // Count total products matching filters
        let count_sql = format!("SELECT COUNT(*) FROM products {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        
        // Bind parameters
        if let Some(category_id) = category_id_param {
            count_query = count_query.bind(category_id);
        }
        
        let total = count_query.fetch_one(self.db.pool()).await?;

        // Get products with pagination
        let limit = params.page_size();
        let offset = params.offset();

        let products_sql = format!(
            "SELECT * FROM products {} ORDER BY id LIMIT ${} OFFSET ${}",
            where_clause,
            param_index,
            param_index + 1
        );

        let mut product_query = sqlx::query(&products_sql);
        
        // Bind parameters
        if let Some(category_id) = category_id_param {
            product_query = product_query.bind(category_id);
        }
        product_query = product_query.bind(limit).bind(offset);

        let products: Vec<Product> = product_query
            .try_map(|row: PgRow| Ok(Product {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                price: row.try_get("price")?,
                sku: row.try_get("sku")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            }))
            .fetch_all(&self.db.pool())
            .await?;

        // Fetch categories for each product
        let mut product_responses = Vec::with_capacity(products.len());
        for product in products {
            let categories = self.get_product_categories(product.id, &self.db.pool()).await?;
            product_responses.push(ProductResponse {
                id: product.id,
                name: product.name,
                description: product.description,
                price: product.price,
                sku: product.sku,
                categories,
                created_at: product.created_at,
                updated_at: product.updated_at,
            });
        }

        Ok(ProductListResponse {
            products: product_responses,
            total,
            page: params.page(),
            page_size: params.page_size(),
        })
    }

    /// Update a product
    pub async fn update_product(
        &self,
        id: i32,
        req: UpdateProductRequest,
    ) -> Result<ProductResponse, ApiError> {
        self.db
            .transaction::<_, _, ProductResponse, ApiError>(|tx| async move {
                // Check if product exists
                let product = sqlx::query_as!(
                    Product,
                    r#"
                    SELECT * FROM products WHERE id = $1
                    "#,
                    id
                )
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| ApiError::not_found("Product", id))?;

                // Update product fields
                let updated_product = sqlx::query_as!(
                    Product,
                    r#"
                    UPDATE products 
                    SET 
                        name = COALESCE($1, name),
                        description = COALESCE($2, description),
                        price = COALESCE($3, price),
                        sku = COALESCE($4, sku),
                        updated_at = NOW()
                    WHERE id = $5
                    RETURNING *
                    "#,
                    req.name,
                    req.description,
                    req.price,
                    req.sku,
                    id
                )
                .fetch_one(&mut **tx)
                .await?;

                // Update categories if provided
                if let Some(category_ids) = req.category_ids {
                    // Delete existing categories
                    sqlx::query!(
                        r#"
                        DELETE FROM product_categories WHERE product_id = $1
                        "#,
                        id
                    )
                    .execute(&mut **tx)
                    .await?;

                    // Insert new categories
                    if !category_ids.is_empty() {
                        let mut query_builder: QueryBuilder<Postgres> =
                            QueryBuilder::new("INSERT INTO product_categories (product_id, category_id) ");

                        query_builder.push_values(category_ids.iter(), |mut b, category_id| {
                            b.push_bind(id).push_bind(category_id);
                        });

                        query_builder.build().execute(&mut **tx).await?;
                    }
                }

                // Fetch categories for response
                let categories = self.get_product_categories(id, &mut **tx).await?;

                Ok(ProductResponse {
                    id: updated_product.id,
                    name: updated_product.name,
                    description: updated_product.description,
                    price: updated_product.price,
                    sku: updated_product.sku,
                    categories,
                    created_at: updated_product.created_at,
                    updated_at: updated_product.updated_at,
                })
            })
            .await
    }

    /// Delete a product
    pub async fn delete_product(&self, id: i32) -> Result<(), ApiError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM products WHERE id = $1
            "#,
            id
        )
        .execute(&self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("Product", id));
        }

        Ok(())
    }

    /// Helper method to get product categories
    async fn get_product_categories<'e, E>(
        &self,
        product_id: i32,
        executor: E,
    ) -> Result<Vec<CategoryBrief>, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            r#"
            SELECT c.id, c.name
            FROM categories c
            JOIN product_categories pc ON c.id = pc.category_id
            WHERE pc.product_id = $1
            ORDER BY c.name
            "#,
            product_id
        )
        .map(|row| CategoryBrief {
            id: row.id,
            name: row.name,
        })
        .fetch_all(executor)
        .await
    }
}