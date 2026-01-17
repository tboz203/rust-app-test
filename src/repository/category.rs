use crate::db::Database;
use crate::error::ApiError;
use crate::models::category::{
    Category, CategoryListResponse, CategoryQueryParams, CategoryResponse, CategoryWithProductsResponse,
    CreateCategoryRequest, UpdateCategoryRequest,
};
use crate::models::product::{Product, ProductResponse};
use anyhow::Result;

/// Repository for category operations
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
        // Check for duplicate category name
        let existing = sqlx::query_scalar!(
            r#"
            SELECT 1 FROM categories WHERE name = $1
            "#,
            req.name
        )
        .fetch_optional(self.db.pool())
        .await?;

        if existing.is_some() {
            return Err(ApiError::Conflict(format!(
                "Category with name '{}' already exists",
                req.name
            )));
        }

        // Create category
        let category = sqlx::query_as!(
            Category,
            r#"
            INSERT INTO categories (name, description)
            VALUES ($1, $2)
            RETURNING *
            "#,
            req.name,
            req.description
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(CategoryResponse {
            id: category.id,
            name: category.name,
            description: category.description,
            created_at: category.created_at,
            updated_at: category.updated_at,
        })
    }

    /// Get a category by ID
    pub async fn get_category(&self, id: i32) -> Result<CategoryResponse, ApiError> {
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT * FROM categories WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| ApiError::not_found("Category", id))?;

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
        let categories = if params.include_product_count() {
            sqlx::query!(
                r#"
                SELECT 
                    c.*,
                    COUNT(pc.product_id) AS product_count
                FROM 
                    categories c
                LEFT JOIN 
                    product_categories pc ON c.id = pc.category_id
                GROUP BY 
                    c.id
                ORDER BY 
                    c.name
                "#
            )
            .map(|row| CategoryWithProductsResponse {
                id: row.id,
                name: row.name,
                description: row.description,
                product_count: row.product_count.unwrap_or(0),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as!(
                Category,
                r#"
                SELECT * FROM categories ORDER BY name
                "#
            )
            .fetch_all(self.db.pool())
            .await?
            .into_iter()
            .map(|c| CategoryWithProductsResponse {
                id: c.id,
                name: c.name,
                description: c.description,
                product_count: 0, // Not counting products
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect()
        };

        Ok(CategoryListResponse { categories })
    }

    /// Update a category
    pub async fn update_category(
        &self,
        id: i32,
        req: UpdateCategoryRequest,
    ) -> Result<CategoryResponse, ApiError> {
        // Check if category exists
        let category = sqlx::query_as!(
            Category,
            r#"
            SELECT * FROM categories WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| ApiError::not_found("Category", id))?;

        // Check for name conflict if name is being updated
        if let Some(name) = &req.name {
            if name != &category.name {
                let existing = sqlx::query_scalar!(
                    r#"
                    SELECT 1 FROM categories WHERE name = $1 AND id != $2
                    "#,
                    name,
                    id
                )
                .fetch_optional(self.db.pool())
                .await?;

                if existing.is_some() {
                    return Err(ApiError::Conflict(format!(
                        "Category with name '{}' already exists",
                        name
                    )));
                }
            }
        }

        // Update category
        let updated_category = sqlx::query_as!(
            Category,
            r#"
            UPDATE categories
            SET 
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
            req.name,
            req.description,
            id
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(CategoryResponse {
            id: updated_category.id,
            name: updated_category.name,
            description: updated_category.description,
            created_at: updated_category.created_at,
            updated_at: updated_category.updated_at,
        })
    }

    /// Delete a category
    pub async fn delete_category(&self, id: i32) -> Result<(), ApiError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM categories WHERE id = $1
            "#,
            id
        )
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("Category", id));
        }

        Ok(())
    }

    /// Get products by category ID
    pub async fn get_products_by_category(&self, category_id: i32) -> Result<Vec<ProductResponse>, ApiError> {
        // First check if the category exists
        let category_exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1)
            "#,
            category_id
        )
        .fetch_one(self.db.pool())
        .await?;

        if !category_exists.unwrap_or(false) {
            return Err(ApiError::not_found("Category", category_id));
        }

        // Get all products in this category
        let products = sqlx::query_as!(
            Product,
            r#"
            SELECT p.*
            FROM products p
            JOIN product_categories pc ON p.id = pc.product_id
            WHERE pc.category_id = $1
            ORDER BY p.name
            "#,
            category_id
        )
        .fetch_all(self.db.pool())
        .await?;

        // Get categories for each product
        let mut product_responses = Vec::with_capacity(products.len());
        for product in products {
            let categories = sqlx::query!(
                r#"
                SELECT c.id, c.name
                FROM categories c
                JOIN product_categories pc ON c.id = pc.category_id
                WHERE pc.product_id = $1
                ORDER BY c.name
                "#,
                product.id
            )
            .map(|row| crate::models::product::CategoryBrief {
                id: row.id,
                name: row.name,
            })
            .fetch_all(self.db.pool())
            .await?;

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

        Ok(product_responses)
    }
}