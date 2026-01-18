use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post, put, delete},
    Router,
};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Once;
use tower::ServiceExt;

use product_catalog_api::{
    api,
    config::Config,
    db::Database,
    models::{
        category::{CategoryResponse, CreateCategoryRequest},
        product::{CreateProductRequest, ProductResponse},
    },
    repository::{category::CategoryRepository, product::ProductRepository},
};

// Used to initialize environment only once
static INIT: Once = Once::new();

/// Initialize test environment
pub async fn initialize() -> PgPool {
    // Load environment variables
    dotenv().ok();
    
    // Only run initialization once
    INIT.call_once(|| {
        // Initialize tracing for tests
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("info")
            .try_init();
    });
    
    // Get test configuration
    let config = Config::from_env().expect("Failed to load configuration");
    
    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database connection pool");
    
    // Run migrations to ensure database is up-to-date
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    
    pool
}

/// Create a test application
pub fn create_test_app(pool: PgPool) -> Router {
    let db = Database::new(pool.clone());
    
    // Create repositories
    let product_repository = ProductRepository::new(db.clone());
    let category_repository = CategoryRepository::new(db.clone());
    
    // Build router with routes
    Router::new()
        .nest("/api", api::routes(pool))
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
                .body(Body::from(
                    serde_json::to_string(&request_body).unwrap()
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let category: CategoryResponse = serde_json::from_slice(&body).unwrap();
    
    category
}

/// Create a test product
pub async fn create_test_product(app: &Router, category_id: i32) -> ProductResponse {
    let request_body = CreateProductRequest {
        name: "Test Product".to_string(),
        description: Some("A test product".to_string()),
        price: 19.99.into(),
        category_id,
        sku: Some("TEST-SKU-123".to_string()),
        in_stock: true,
        weight: Some(2.5),
        dimensions: Some("10x20x30".to_string()),
    };
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&request_body).unwrap()
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let product: ProductResponse = serde_json::from_slice(&body).unwrap();
    
    product
}

/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) {
    // Delete all products and categories
    let _ = sqlx::query("DELETE FROM products")
        .execute(pool)
        .await;
        
    let _ = sqlx::query("DELETE FROM categories")
        .execute(pool)
        .await;
}