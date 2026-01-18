use crate::models::category::{
    CategoryListResponse, CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest,
};
use crate::models::product::ProductResponse;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};

// Import from common module
use super::common::{
    cleanup_test_data, create_test_app, create_test_category, create_test_product, initialize,
};

#[tokio::test]
async fn test_list_categories() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category1 = create_test_category(&app).await;

    // Create a second category
    let request_body = CreateCategoryRequest {
        name: "Second Test Category".to_string(),
        description: Some("Another test category".to_string()),
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

    // Test list categories
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let categories: CategoryListResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(categories.categories.len(), 2);
    assert!(categories
        .categories
        .iter()
        .any(|c| c.name == "Test Category"));
    assert!(categories
        .categories
        .iter()
        .any(|c| c.name == "Second Test Category"));

    // Test list categories with product count
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/categories?include_product_count=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let categories: CategoryListResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(categories.categories.len(), 2);
    assert!(categories
        .categories
        .iter()
        .all(|c| c.product_count.is_some()));

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_get_category() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;

    // Test get category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/categories/{}", category.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_category: CategoryResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_category.id, category.id);
    assert_eq!(response_category.name, "Test Category");
    assert_eq!(
        response_category.description,
        Some("A test category".to_string())
    );

    // Test get non-existent category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/categories/9999")
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
async fn test_create_category() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Test create category with valid data
    let request_body = CreateCategoryRequest {
        name: "New Category".to_string(),
        description: Some("A brand new category".to_string()),
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let category: CategoryResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(category.name, "New Category");
    assert_eq!(
        category.description,
        Some("A brand new category".to_string())
    );

    // Test create category with invalid data (empty name)
    let invalid_body = CreateCategoryRequest {
        name: "".to_string(), // Empty name, should fail validation
        description: Some("Invalid category".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/categories")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Clean up test data
    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_update_category() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;

    // Test update category
    let update_body = UpdateCategoryRequest {
        name: Some("Updated Category".to_string()),
        description: Some("Updated description".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/categories/{}", category.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated_category: CategoryResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_category.id, category.id);
    assert_eq!(updated_category.name, "Updated Category");
    assert_eq!(
        updated_category.description,
        Some("Updated description".to_string())
    );

    // Test update non-existent category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/categories/9999")
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
async fn test_delete_category() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;

    // Test delete category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/categories/{}", category.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify category was deleted by trying to get it
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/categories/{}", category.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test delete non-existent category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/categories/9999")
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
async fn test_get_category_products() {
    // Initialize test environment
    let pool = initialize().await;
    let app = create_test_app(pool.clone());

    // Create test data
    let category = create_test_category(&app).await;
    let product1 = create_test_product(&app, category.id).await;

    // Create a second product in the same category
    let request_body = product_catalog_api::models::product::CreateProductRequest {
        name: "Second Product".to_string(),
        description: Some("Another product in the category".to_string()),
        price: 29.99.into(),
        category_id: category.id,
        sku: Some("CAT-SKU-456".to_string()),
        in_stock: true,
        weight: Some(1.5),
        dimensions: Some("5x10x15".to_string()),
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

    // Test get category products
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/categories/{}/products", category.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let products: Vec<ProductResponse> = serde_json::from_slice(&body).unwrap();

    assert_eq!(products.len(), 2);
    assert!(products.iter().any(|p| p.name == "Test Product"));
    assert!(products.iter().any(|p| p.name == "Second Product"));

    // Test get products for non-existent category
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/categories/9999/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Clean up test data
    cleanup_test_data(&pool).await;
}
