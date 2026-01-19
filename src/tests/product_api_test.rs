use std::str::FromStr;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bigdecimal::BigDecimal;
use tower::ServiceExt;

// Import from common module
use super::common::{cleanup_test_data, create_test_app, create_test_category, create_test_product, initialize};
// Import from crate root using the lib.rs exports
use crate::{
    entity::{
        Category, CategoryActiveModel, CategoryModel, Product, ProductActiveModel, ProductCategory,
        ProductCategoryModel, ProductModel,
    },
    models::{
        category::{CategoryResponse, CreateCategoryRequest},
        product::{CategoryBrief, CreateProductRequest, ProductListResponse, ProductResponse, UpdateProductRequest},
    },
};

#[tokio::test]
async fn test_list_products() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product1 = create_test_product(&app, vec![category.id]).await;

    // Create a second product for pagination testing
    let request_body = CreateProductRequest {
        name: "Second Test Product".to_string(),
        description: Some("Another test product".to_string()),
        price: BigDecimal::from_str("29.99").unwrap(),
        category_ids: vec![category.id],
        sku: Some("TEST-SKU-456".to_string()),
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
    assert!(products.products.iter().any(|p| p.name == "Second Test Product"));

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
    let product = create_test_product(&app, vec![category.id]).await;

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
    assert!(response_product.categories.iter().any(|c| c.id == category.id));

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
        category_ids: vec![category.id],
        sku: Some("NEW-SKU-789".to_string()),
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
        category_ids: vec![category.id],
        sku: Some("INV-SKU".to_string()),
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
        category_ids: vec![9999], // Non-existent category
        sku: Some("IC-SKU".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&invalid_category_body).unwrap()))
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
    let product = create_test_product(&app, vec![category.id]).await;

    // Test update product
    let update_body = UpdateProductRequest {
        name: Some("Updated Product".to_string()),
        description: Some("Updated description".to_string()),
        price: Some(BigDecimal::from_str("49.99").unwrap()),
        category_ids: Some(vec![category.id]),
        sku: Some("UPD-SKU-123".to_string()),
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
    assert_eq!(updated_product.description, Some("Updated description".to_string()));

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
    let product = create_test_product(&app, vec![category.id]).await;

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

#[tokio::test]
async fn test_product_category_many_to_many() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test categories
    let category1 = create_test_category(&app).await;

    // Create a second category
    let category2_request = CreateCategoryRequest {
        name: "Second Category".to_string(),
        description: Some("Another test category".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/categories")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&category2_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let category2: CategoryResponse = serde_json::from_slice(&body).unwrap();

    // Create a product with multiple categories
    let product_request = CreateProductRequest {
        name: "Multi-Category Product".to_string(),
        description: Some("A product with multiple categories".to_string()),
        price: BigDecimal::from_str("39.99").unwrap(),
        category_ids: vec![category1.id, category2.id],
        sku: Some("MULTI-CAT-001".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/products")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&product_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let product: ProductResponse = serde_json::from_slice(&body).unwrap();

    // Verify product has both categories
    assert_eq!(product.categories.len(), 2);
    assert!(product.categories.iter().any(|c| c.id == category1.id));
    assert!(product.categories.iter().any(|c| c.id == category2.id));

    // Create a third category
    let category3_request = CreateCategoryRequest {
        name: "Third Category".to_string(),
        description: Some("Yet another test category".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/categories")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&category3_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let category3: CategoryResponse = serde_json::from_slice(&body).unwrap();

    // Update the product to add the third category and remove the first one
    let update_request = UpdateProductRequest {
        name: None,
        description: None,
        price: None,
        category_ids: Some(vec![category2.id, category3.id]),
        sku: None,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/products/{}", product.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_product: ProductResponse = serde_json::from_slice(&body).unwrap();

    // Verify product has been updated with new categories
    assert_eq!(updated_product.categories.len(), 2);
    assert!(!updated_product.categories.iter().any(|c| c.id == category1.id));
    assert!(updated_product.categories.iter().any(|c| c.id == category2.id));
    assert!(updated_product.categories.iter().any(|c| c.id == category3.id));

    // Clean up test data
    cleanup_test_data(&pool).await;
}
