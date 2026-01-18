use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use hyper::body::to_bytes;
use tower::ServiceExt;
use bigdecimal::BigDecimal;
use dotenvy::dotenv;
use sea_orm::{DatabaseConnection, Database, ConnectOptions, EntityTrait, DeleteResult, QueryFilter, ColumnTrait};
use std::str::FromStr;
use std::sync::Once;
use std::time::Duration;

use crate::{
    api,
    config::Config,
    db,
    entity::{
        Category, CategoryModel, CategoryActiveModel,
        Product, ProductModel, ProductActiveModel,
        ProductCategory, ProductCategoryModel
    },
    models::{
        category::{CategoryResponse, CreateCategoryRequest},
        product::{CreateProductRequest, ProductResponse},
    },
    repository::{category::CategoryRepository, product::ProductRepository},
};

// Used to initialize environment only once
static INIT: Once = Once::new();

/// Initialize test environment
pub async fn initialize() -> DatabaseConnection {
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

    // Create database connection using sea-orm
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60));

    let db = Database::connect(opt)
        .await
        .expect("Failed to create database connection");

    // Run migrations using our custom Migrator
    // This could use the Migrator::up method in a real implementation
    // or the run_sql_migrations method to run existing SQL files
    //
    // For now we'll assume migrations are run elsewhere to avoid disrupting
    // the database during tests
    // crate::migration::Migrator::run_sql_migrations(&db, "./migrations").await
    //     .expect("Failed to run database migrations");

    db
}

/// Create a test application
pub fn create_test_app(db_conn: DatabaseConnection) -> Router {
    // Use the API routes function directly with the DatabaseConnection
    // This matches how it's used in the main application
    Router::new().nest("/api", api::routes(db_conn))
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
        category_ids: category_ids,
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
    let _ = ProductCategory::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete product categories");
    
    // Then delete products
    let _ = Product::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete products");
    
    // Finally delete categories
    let _ = Category::delete_many()
        .exec(db)
        .await
        .expect("Failed to delete categories");
}
