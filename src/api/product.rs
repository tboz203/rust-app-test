use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::{info, instrument};
use validator::Validate;

use crate::{
    error::ApiError,
    models::product::{CreateProductRequest, ProductListResponse, ProductQueryParams, ProductResponse, UpdateProductRequest},
    repository::product::ProductRepository,
};

/// List all products with pagination
///
/// GET /api/products
#[instrument(skip(repository))]
pub async fn list_products(
    State(repository): State<ProductRepository>,
    Query(params): Query<ProductQueryParams>,
) -> Result<Json<ProductListResponse>, ApiError> {
    info!("Listing products with params: page={}, page_size={}", params.page().to_string(), params.page_size().to_string());
    
    let response = repository.list_products(params).await?;
    
    info!("Found {} products", response.total);
    Ok(Json(response))
}

/// Get a product by ID
///
/// GET /api/products/:id
#[instrument(skip(repository))]
pub async fn get_product(
    State(repository): State<ProductRepository>,
    Path(id): Path<i32>,
) -> Result<Json<ProductResponse>, ApiError> {
    info!("Getting product with ID: {}", id);
    
    let product = repository.get_product(id).await?;
    
    info!("Found product: {}", product.name);
    Ok(Json(product))
}

/// Create a new product
///
/// POST /api/products
#[instrument(skip(repository, request))]
pub async fn create_product(
    State(repository): State<ProductRepository>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, ApiError> {
    info!("Creating new product: {}", request.name);
    
    // Validate the request
    request.validate()?;

    // Create the product
    let product = repository.create_product(request).await?;

    info!("Created product with ID: {}", product.id);
    Ok(Json(product))
}

/// Update an existing product
///
/// PUT /api/products/:id
#[instrument(skip(repository, request))]
pub async fn update_product(
    State(repository): State<ProductRepository>,
    Path(id): Path<i32>,
    Json(request): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, ApiError> {
    info!("Updating product with ID: {}", id);
    
    // Validate the request
    request.validate()?;
    
    // Update the product
    let product = repository.update_product(id, request).await?;

    info!("Updated product: {}", product.name);
    Ok(Json(product))
}

/// Delete a product
///
/// DELETE /api/products/:id
#[instrument(skip(repository))]
pub async fn delete_product(
    State(repository): State<ProductRepository>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Deleting product with ID: {}", id);
    
    repository.delete_product(id).await?;
    
    info!("Product deleted successfully");
    Ok(Json(serde_json::json!({ "message": "Product deleted successfully" })))
}