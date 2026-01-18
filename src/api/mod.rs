pub mod category;
pub mod product;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sea_orm::DatabaseConnection;

use crate::db::Database;
use crate::repository::{category::CategoryRepository, product::ProductRepository};

/// Create all routes for the API
pub fn routes(conn: DatabaseConnection) -> Router {
    // Create repositories
    let product_repository = ProductRepository::new(conn.clone());
    let category_repository = CategoryRepository::new(conn.clone());

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
        .route(
            "/categories/:id/products",
            get(category::get_category_products),
        )
        .with_state(repository)
}
