use axum::{
    body::Body,
    http::{Request, StatusCode},
};
// Import from crate root using the lib.rs exports
use crate::models::product::{
    CreateProductRequest, ProductListResponse, ProductResponse, UpdateProductRequest,
};
use sqlx::types::BigDecimal;
use std::str::FromStr;
use tower::ServiceExt;

// Import from common module
use super::common::{
    cleanup_test_data, create_test_app, create_test_category, create_test_product, initialize,
};

#[tokio::test]
async fn test_list_products() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product1 = create_test_product(&app, category.id).await;

    // Create a second product for pagination testing
    let request_body = CreateProductRequest {
        name: "Second Test Product".to_string(),
        description: Some("Another test product".to_string()),
        price: BigDecimal::from_str("29.99").unwrap(),
        category_id: category.id,
        sku: Some("TEST-SKU-456".to_string()),
        in_stock: true,
        weight: Some(1.5),
        dimensions: Some("5x10x15".to_string()),
    };

    let _ = app
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

    // Test list products with default parameters
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let products: ProductListResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(products.total, 2);
    assert_eq!(products.products.len(), 2);
    assert!(products.products.iter().any(|p| p.name == "Test Product"));
    assert!(products
        .products
        .iter()
        .any(|p| p.name == "Second Test Product"));

    // Test pagination - page 1, limit 1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/products?page=1&page_size=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let products: ProductListResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(products.total, 2);
    assert_eq!(products.products.len(), 1);

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_product() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product = create_test_product(&app, category.id).await;

    // Test get product
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/products/{}", product.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_product: ProductResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_product.id, product.id);
    assert_eq!(response_product.name, "Test Product");
    assert_eq!(response_product.category_id, category.id);

    // Test get non-existent product
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/products/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_create_product() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test category
    let category = create_test_category(&app).await;

    // Test create product with valid data
    let request_body = CreateProductRequest {
        name: "New Product".to_string(),
        description: Some("A brand new product".to_string()),
        price: BigDecimal::from_str("39.99").unwrap(),
        category_id: category.id,
        sku: Some("NEW-SKU-789".to_string()),
        in_stock: true,
        weight: Some(3.5),
        dimensions: Some("15x25x35".to_string()),
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let product: ProductResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(product.name, "New Product");
    assert_eq!(product.description, Some("A brand new product".to_string()));

    // Test create product with invalid data (empty name)
    let invalid_body = CreateProductRequest {
        name: "".to_string(), // Empty name, should fail validation
        description: Some("Invalid product".to_string()),
        price: BigDecimal::from_str("9.99").unwrap(),
        category_id: category.id,
        sku: Some("INV-SKU".to_string()),
        in_stock: true,
        weight: Some(1.0),
        dimensions: Some("5x5x5".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test create product with non-existent category
    let invalid_category_body = CreateProductRequest {
        name: "Invalid Category Product".to_string(),
        description: Some("A product with invalid category".to_string()),
        price: BigDecimal::from_str("19.99").unwrap(),
        category_id: 9999, // Non-existent category
        sku: Some("IC-SKU".to_string()),
        in_stock: true,
        weight: Some(2.0),
        dimensions: Some("10x10x10".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&invalid_category_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_product() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product = create_test_product(&app, category.id).await;

    // Test update product
    let update_body = UpdateProductRequest {
        name: Some("Updated Product".to_string()),
        description: Some("Updated description".to_string()),
        price: Some(BigDecimal::from_str("49.99").unwrap()),
        category_id: Some(category.id),
        sku: Some("UPD-SKU-123".to_string()),
        in_stock: Some(false),
        weight: Some(4.5),
        dimensions: Some("20x30x40".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/products/{}", product.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_product: ProductResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_product.id, product.id);
    assert_eq!(updated_product.name, "Updated Product");
    assert_eq!(
        updated_product.description,
        Some("Updated description".to_string())
    );
    assert_eq!(updated_product.in_stock, false);

    // Test update non-existent product
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/products/9999")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_delete_product() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product = create_test_product(&app, category.id).await;

    // Test delete product
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/products/{}", product.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify product was deleted by trying to get it
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/products/{}", product.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test delete non-existent product
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/products/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Clean up test data
    cleanup_test_data(&pool).await;
}
