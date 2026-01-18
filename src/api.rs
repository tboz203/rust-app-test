pub mod product;
pub mod category;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

use crate::repository::{
    product::ProductRepository,
    category::CategoryRepository,
};
use crate::db::Database;

/// Create all routes for the API
pub fn routes(pool: PgPool) -> Router {
    let db = Database::from_pool(pool);
    
    // Create repositories
    let product_repository = ProductRepository::new(db.clone());
    let category_repository = CategoryRepository::new(db.clone());

    // Combine all routes
    Router::new()
        .merge(product_routes(product_repository))
        .merge(category_routes(category_repository))
}

/// Create product routes
fn product_routes(repository: ProductRepository) -> Router {
    Router::new()
        .route("/products", get(product::list_products))
        .route("/products", post(product::create_product))
        .route("/products/:id", get(product::get_product))
        .route("/products/:id", put(product::update_product))
        .route("/products/:id", delete(product::delete_product))
        .with_state(repository)
}

/// Create category routes
fn category_routes(repository: CategoryRepository) -> Router {
    Router::new()
        .route("/categories", get(category::list_categories))
        .route("/categories", post(category::create_category))
        .route("/categories/:id", get(category::get_category))
        .route("/categories/:id", put(category::update_category))
        .route("/categories/:id", delete(category::delete_category))
        .route("/categories/:id/products", get(category::get_category_products))
        .with_state(repository)
}