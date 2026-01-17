# API Documentation

This document provides detailed information about the Product Catalog API endpoints, request/response formats, and usage examples.

## Table of Contents

- [Base URL](#base-url)
- [Response Format](#response-format)
- [Common Error Codes](#common-error-codes)
- [Product Endpoints](#product-endpoints)
  - [List Products](#list-products)
  - [Get Product](#get-product)
  - [Create Product](#create-product)
  - [Update Product](#update-product)
  - [Delete Product](#delete-product)
- [Category Endpoints](#category-endpoints)
  - [List Categories](#list-categories)
  - [Get Category](#get-category)
  - [Create Category](#create-category)
  - [Update Category](#update-category)
  - [Delete Category](#delete-category)
  - [Get Category Products](#get-category-products)

## Base URL

All API requests should be made to:

```
http://your-server:port/api
```

## Response Format

All successful responses return JSON data with appropriate HTTP status codes.

## Common Error Codes

| Status Code | Description |
|-------------|-------------|
| 400 | Bad Request - Invalid input or validation errors |
| 404 | Not Found - Resource doesn't exist |
| 500 | Internal Server Error - Something went wrong on the server |

Error responses have the following format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

---

## Product Endpoints

### List Products

Returns a paginated list of products.

- **URL**: `/products`
- **Method**: `GET`
- **Query Parameters**:

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| page | integer | No | 1 | Page number |
| page_size | integer | No | 10 | Items per page |
| category_id | integer | No | - | Filter by category ID |
| in_stock | boolean | No | - | Filter by in-stock status |

#### Example Request

```
GET /api/products?page=1&page_size=5&category_id=2
```

#### Example Response

```json
{
  "products": [
    {
      "id": 1,
      "name": "Classic T-Shirt",
      "description": "Comfortable cotton t-shirt",
      "price": "19.99",
      "category_id": 2,
      "sku": "TS-CL-001",
      "in_stock": true,
      "weight": 0.25,
      "dimensions": "25x35x3",
      "created_at": "2026-01-15T10:30:00Z",
      "updated_at": "2026-01-15T10:30:00Z"
    },
    {
      "id": 2,
      "name": "Denim Jeans",
      "description": "Classic denim jeans",
      "price": "59.99",
      "category_id": 2,
      "sku": "DN-JN-002",
      "in_stock": true,
      "weight": 0.8,
      "dimensions": "30x40x5",
      "created_at": "2026-01-15T11:15:00Z",
      "updated_at": "2026-01-15T11:15:00Z"
    }
  ],
  "page": 1,
  "page_size": 5,
  "total": 2,
  "total_pages": 1
}
```

---

### Get Product

Returns detailed information about a specific product.

- **URL**: `/products/:id`
- **Method**: `GET`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Product ID |

#### Example Request

```
GET /api/products/1
```

#### Example Response

```json
{
  "id": 1,
  "name": "Classic T-Shirt",
  "description": "Comfortable cotton t-shirt",
  "price": "19.99",
  "category_id": 2,
  "category_name": "Clothing",
  "sku": "TS-CL-001",
  "in_stock": true,
  "weight": 0.25,
  "dimensions": "25x35x3",
  "created_at": "2026-01-15T10:30:00Z",
  "updated_at": "2026-01-15T10:30:00Z"
}
```

#### Error Responses

- **404 Not Found** - If the product doesn't exist

```json
{
  "error": {
    "code": "PRODUCT_NOT_FOUND",
    "message": "Product with ID 999 not found"
  }
}
```

---

### Create Product

Creates a new product.

- **URL**: `/products`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Product name (1-100 chars) |
| description | string | No | Product description |
| price | decimal | Yes | Product price (>= 0) |
| category_id | integer | Yes | Category ID |
| sku | string | No | Stock keeping unit |
| in_stock | boolean | Yes | Whether the product is in stock |
| weight | decimal | No | Product weight in kg |
| dimensions | string | No | Product dimensions (format: LxWxH) |

#### Example Request

```json
{
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones with noise cancellation",
  "price": "129.99",
  "category_id": 3,
  "sku": "WL-HP-001",
  "in_stock": true,
  "weight": 0.3,
  "dimensions": "20x15x8"
}
```

#### Example Response

```json
{
  "id": 3,
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones with noise cancellation",
  "price": "129.99",
  "category_id": 3,
  "category_name": "Electronics",
  "sku": "WL-HP-001",
  "in_stock": true,
  "weight": 0.3,
  "dimensions": "20x15x8",
  "created_at": "2026-01-17T14:25:30Z",
  "updated_at": "2026-01-17T14:25:30Z"
}
```

#### Error Responses

- **400 Bad Request** - If validation fails

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": {
      "name": ["Name cannot be empty"],
      "price": ["Price must be a positive number"]
    }
  }
}
```

---

### Update Product

Updates an existing product.

- **URL**: `/products/:id`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Product ID |

- **Request Body**: All fields are optional. Only provided fields will be updated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | Product name (1-100 chars) |
| description | string | No | Product description |
| price | decimal | No | Product price (>= 0) |
| category_id | integer | No | Category ID |
| sku | string | No | Stock keeping unit |
| in_stock | boolean | No | Whether the product is in stock |
| weight | decimal | No | Product weight in kg |
| dimensions | string | No | Product dimensions (format: LxWxH) |

#### Example Request

```json
{
  "price": "149.99",
  "in_stock": false
}
```

#### Example Response

```json
{
  "id": 3,
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones with noise cancellation",
  "price": "149.99",
  "category_id": 3,
  "category_name": "Electronics",
  "sku": "WL-HP-001",
  "in_stock": false,
  "weight": 0.3,
  "dimensions": "20x15x8",
  "created_at": "2026-01-17T14:25:30Z",
  "updated_at": "2026-01-17T14:30:45Z"
}
```

#### Error Responses

- **404 Not Found** - If the product doesn't exist
- **400 Bad Request** - If validation fails

---

### Delete Product

Deletes a product.

- **URL**: `/products/:id`
- **Method**: `DELETE`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Product ID |

#### Example Request

```
DELETE /api/products/3
```

#### Example Response

```json
{
  "message": "Product deleted successfully"
}
```

#### Error Responses

- **404 Not Found** - If the product doesn't exist

---

## Category Endpoints

### List Categories

Returns a list of all categories.

- **URL**: `/categories`
- **Method**: `GET`
- **Query Parameters**:

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| include_product_count | boolean | No | false | Include product count for each category |

#### Example Request

```
GET /api/categories?include_product_count=true
```

#### Example Response

```json
{
  "categories": [
    {
      "id": 1,
      "name": "Home & Kitchen",
      "description": "Home appliances and kitchen accessories",
      "product_count": 15,
      "created_at": "2026-01-15T09:00:00Z",
      "updated_at": "2026-01-15T09:00:00Z"
    },
    {
      "id": 2,
      "name": "Clothing",
      "description": "Men's and women's clothing",
      "product_count": 24,
      "created_at": "2026-01-15T09:05:00Z",
      "updated_at": "2026-01-15T09:05:00Z"
    },
    {
      "id": 3,
      "name": "Electronics",
      "description": "Electronic devices and accessories",
      "product_count": 18,
      "created_at": "2026-01-15T09:10:00Z",
      "updated_at": "2026-01-15T09:10:00Z"
    }
  ]
}
```

---

### Get Category

Returns detailed information about a specific category.

- **URL**: `/categories/:id`
- **Method**: `GET`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Category ID |

#### Example Request

```
GET /api/categories/2
```

#### Example Response

```json
{
  "id": 2,
  "name": "Clothing",
  "description": "Men's and women's clothing",
  "created_at": "2026-01-15T09:05:00Z",
  "updated_at": "2026-01-15T09:05:00Z"
}
```

#### Error Responses

- **404 Not Found** - If the category doesn't exist

---

### Create Category

Creates a new category.

- **URL**: `/categories`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Category name (1-50 chars) |
| description | string | No | Category description |

#### Example Request

```json
{
  "name": "Sports & Outdoors",
  "description": "Sports equipment and outdoor gear"
}
```

#### Example Response

```json
{
  "id": 4,
  "name": "Sports & Outdoors",
  "description": "Sports equipment and outdoor gear",
  "created_at": "2026-01-17T14:40:00Z",
  "updated_at": "2026-01-17T14:40:00Z"
}
```

#### Error Responses

- **400 Bad Request** - If validation fails

---

### Update Category

Updates an existing category.

- **URL**: `/categories/:id`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Category ID |

- **Request Body**: All fields are optional. Only provided fields will be updated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | Category name (1-50 chars) |
| description | string | No | Category description |

#### Example Request

```json
{
  "name": "Sports & Outdoor Activities",
  "description": "Equipment for sports and outdoor activities"
}
```

#### Example Response

```json
{
  "id": 4,
  "name": "Sports & Outdoor Activities",
  "description": "Equipment for sports and outdoor activities",
  "created_at": "2026-01-17T14:40:00Z",
  "updated_at": "2026-01-17T14:45:30Z"
}
```

#### Error Responses

- **404 Not Found** - If the category doesn't exist
- **400 Bad Request** - If validation fails

---

### Delete Category

Deletes a category.

- **URL**: `/categories/:id`
- **Method**: `DELETE`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Category ID |

#### Example Request

```
DELETE /api/categories/4
```

#### Example Response

```json
{
  "message": "Category deleted successfully"
}
```

#### Error Responses

- **404 Not Found** - If the category doesn't exist
- **400 Bad Request** - If the category has products associated with it (depending on implementation)

---

### Get Category Products

Returns all products belonging to a specific category.

- **URL**: `/categories/:id/products`
- **Method**: `GET`
- **URL Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| id | integer | Yes | Category ID |

#### Example Request

```
GET /api/categories/2/products
```

#### Example Response

```json
[
  {
    "id": 1,
    "name": "Classic T-Shirt",
    "description": "Comfortable cotton t-shirt",
    "price": "19.99",
    "category_id": 2,
    "sku": "TS-CL-001",
    "in_stock": true,
    "weight": 0.25,
    "dimensions": "25x35x3",
    "created_at": "2026-01-15T10:30:00Z",
    "updated_at": "2026-01-15T10:30:00Z"
  },
  {
    "id": 2,
    "name": "Denim Jeans",
    "description": "Classic denim jeans",
    "price": "59.99",
    "category_id": 2,
    "sku": "DN-JN-002",
    "in_stock": true,
    "weight": 0.8,
    "dimensions": "30x40x5",
    "created_at": "2026-01-15T11:15:00Z",
    "updated_at": "2026-01-15T11:15:00Z"
  }
]
```

#### Error Responses

- **404 Not Found** - If the category doesn't exist