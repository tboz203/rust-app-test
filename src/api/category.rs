use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::{info, instrument};

use crate::{
    error::ApiError,
    models::category::{CategoryListResponse, CategoryQueryParams, CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest},
    models::product::ProductResponse,
    repository::category::CategoryRepository,
    validation::validate_json,
};

/// List all categories
///
/// GET /api/categories
#[instrument(skip(repository))]
pub async fn list_categories(
    State(repository): State<CategoryRepository>,
    Query(params): Query<CategoryQueryParams>,
) -> Result<Json<CategoryListResponse>, ApiError> {
    info!("Listing categories with product count: {}", params.include_product_count());
    
    let response = repository.list_categories(params).await?;
    
    info!("Found {} categories", response.categories.len());
    Ok(Json(response))
}

/// Get a category by ID
///
/// GET /api/categories/:id
#[instrument(skip(repository))]
pub async fn get_category(
    State(repository): State<CategoryRepository>,
    Path(id): Path<i32>,
) -> Result<Json<CategoryResponse>, ApiError> {
    info!("Getting category with ID: {}", id);
    
    let category = repository.get_category(id).await?;
    
    info!("Found category: {}", category.name);
    Ok(Json(category))
}

/// Create a new category
///
/// POST /api/categories
#[instrument(skip(repository, payload))]
pub async fn create_category(
    State(repository): State<CategoryRepository>,
    payload: Json<CreateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    info!("Creating new category: {}", payload.name);
    
    // Validate the request
    let category_req = validate_json(payload).await?;
    
    // Create the category
    let category = repository.create_category(category_req).await?;
    
    info!("Created category with ID: {}", category.id);
    Ok(Json(category))
}

/// Update an existing category
///
/// PUT /api/categories/:id
#[instrument(skip(repository, payload))]
pub async fn update_category(
    State(repository): State<CategoryRepository>,
    Path(id): Path<i32>,
    payload: Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    info!("Updating category with ID: {}", id);
    
    // Validate the request
    let category_req = validate_json(payload).await?;
    
    // Update the category
    let category = repository.update_category(id, category_req).await?;
    
    info!("Updated category: {}", category.name);
    Ok(Json(category))
}

/// Delete a category
///
/// DELETE /api/categories/:id
#[instrument(skip(repository))]
pub async fn delete_category(
    State(repository): State<CategoryRepository>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Deleting category with ID: {}", id);
    
    repository.delete_category(id).await?;
    
    info!("Category deleted successfully");
    Ok(Json(serde_json::json!({ "message": "Category deleted successfully" })))
}

/// Get products by category ID
///
/// GET /api/categories/:id/products
#[instrument(skip(repository))]
pub async fn get_category_products(
    State(repository): State<CategoryRepository>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<ProductResponse>>, ApiError> {
    info!("Getting products for category ID: {}", id);
    
    let products = repository.get_products_by_category(id).await?;
    
    info!("Found {} products in category", products.len());
    Ok(Json(products))
}