use std::str::FromStr;
use std::sync::Once;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use bigdecimal::BigDecimal;
use dotenvy::dotenv;
use hyper::body::to_bytes;
use sea_orm::{ColumnTrait, ConnectOptions, Database, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter};
use tower::ServiceExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::Config;
use crate::entity::{categories, product_categories, products};
use crate::models::category::{CategoryResponse, CreateCategoryRequest};
use crate::models::product::{CreateProductRequest, ProductResponse};
use crate::repository::category::CategoryRepository;
use crate::repository::product::ProductRepository;
use crate::{api, database};

// Used to initialize environment only once
static INIT: Once = Once::new();

/// Initialize test environment
pub async fn initialize() -> (DatabaseConnection, Router) {

    // Only run initialization once
    INIT.call_once(|| {
        // Initialize tracing for tests
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();
    });

    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.get_schema_registry("product_catalog_api::entity::*").sync(&db).await.unwrap();

    let router = Router::new().nest("/api", api::routes(db.clone()));

    (db, router)
}

/// Create a test category
pub async fn create_test_category(app: &Router) -> CategoryResponse {
    let request_body = CreateCategoryRequest {
        name: "Test Category".to_string(),
        description: Some("A test category".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/categories")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let category: CategoryResponse = serde_json::from_slice(&body).unwrap();

    category
}

/// Create a test product
pub async fn create_test_product(app: &Router, category_ids: Vec<i32>) -> ProductResponse {
    let request_body = CreateProductRequest {
        name: "Test Product".to_string(),
        description: Some("A test product".to_string()),
        price: BigDecimal::from_str("19.99").unwrap(),
        category_ids,
        sku: Some("TEST-SKU-123".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let product: ProductResponse = serde_json::from_slice(&body).unwrap();

    product
}

/// Clean up test data
pub async fn cleanup_test_data(db: &DatabaseConnection) {
    // Delete all data in the correct order to respect foreign key constraints
    // First delete the product_categories (junction table)
    let _ = product_categories::Entity::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete product categories");

    // Then delete products
    let _ = products::Entity::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete products");

    // Finally delete categories
    let _ = categories::Entity::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete categories");
}
