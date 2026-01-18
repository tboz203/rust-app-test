# Migration Plan: rust_decimal to bigdecimal

This document outlines the step-by-step plan for migrating from `rust_decimal` to `sqlx::types::BigDecimal` in our Rust application.

## 1. Background

The project currently uses:
- `rust_decimal` 1.33.1
- `rust_decimal_macros` 1.33.1 
- SQLx is already configured with the "bigdecimal" feature

We need to replace all usages of `rust_decimal::Decimal` with `sqlx::types::BigDecimal` throughout the codebase.

## 2. Step-by-Step Migration Plan

### Phase 1: Dependency Updates

1. Update `Cargo.toml`:
   ```diff
   # Database
   sqlx = { version = "0.7.3", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "migrate", "bigdecimal"] }
   - rust_decimal = "1.33.1"
   - rust_decimal_macros = "1.33.1"
   ```

   The SQLx crate already includes the `bigdecimal` feature which provides `sqlx::types::BigDecimal`.

### Phase 2: Model Struct Changes

1. Update `src/models/product.rs`:
   ```diff
   - use rust_decimal::Decimal;
   + use sqlx::types::BigDecimal;
   use serde::{Deserialize, Serialize};
   use sqlx::FromRow;
   use validator::Validate;
   ```

2. Update the following structs to use `BigDecimal` instead of `Decimal`:
   
   ```diff
   #[derive(Debug, Serialize, FromRow)]
   pub struct Product {
       pub id: i32,
       pub name: String,
       pub description: Option<String>,
   -   pub price: Decimal,
   +   pub price: BigDecimal,
       pub sku: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }

   #[derive(Debug, Deserialize, Validate)]
   pub struct CreateProductRequest {
       #[validate(length(min = 1, max = 255, message = "Product name cannot be empty and must be less than 256 characters"))]
       pub name: String,
       pub description: Option<String>,
       #[validate(custom = "validate_positive_decimal")]
   -   pub price: Decimal,
   +   pub price: BigDecimal,
       #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
       pub sku: Option<String>,
       #[validate(custom = "validate_non_empty_vec")]
       pub category_ids: Vec<i32>,
   }

   #[derive(Debug, Deserialize, Validate)]
   pub struct UpdateProductRequest {
       #[validate(length(min = 1, max = 255, message = "Product name must be less than 256 characters"))]
       pub name: Option<String>,
       pub description: Option<String>,
       #[validate(custom(function = "validate_optional_decimal"))]
   -   pub price: Option<Decimal>,
   +   pub price: Option<BigDecimal>,
       #[validate(length(max = 50, message = "SKU must be less than 51 characters"))]
       pub sku: Option<String>,
       #[validate(custom(function = "validate_optional_non_empty_vec"))]
       pub category_ids: Option<Vec<i32>>,
   }

   #[derive(Debug, Serialize)]
   pub struct ProductResponse {
       pub id: i32,
       pub name: String,
       pub description: Option<String>,
   -   pub price: Decimal,
   +   pub price: BigDecimal,
       pub sku: Option<String>,
       pub categories: Vec<CategoryBrief>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```

### Phase 3: Validation Function Updates

1. Update `src/validation/product.rs`:
   ```diff
   - use rust_decimal::Decimal;
   + use sqlx::types::BigDecimal;
   use validator::ValidationError;
   ```

2. Modify the validation functions to work with `BigDecimal`:
   
   ```diff
   /// Validates that a Decimal value is positive (greater than zero)
   - pub fn validate_positive_decimal(decimal: &Decimal) -> Result<(), ValidationError> {
   -     if decimal.is_sign_negative() || decimal.is_zero() {
   + pub fn validate_positive_decimal(decimal: &BigDecimal) -> Result<(), ValidationError> {
   +     if decimal.is_sign_negative() || decimal == &BigDecimal::from(0) {
           let mut error = ValidationError::new("positive_decimal");
           error.message = Some(std::borrow::Cow::from("Price must be a positive number"));
           return Err(error);
       }
       Ok(())
   }

   /// Validates that an Option<Decimal> value is positive if it exists
   - pub fn validate_optional_decimal(decimal: &Option<Decimal>) -> Result<(), ValidationError> {
   + pub fn validate_optional_decimal(decimal: &Option<BigDecimal>) -> Result<(), ValidationError> {
       match decimal {
           Some(d) => validate_positive_decimal(d),
           None => Ok(()),
       }
   }
   ```

### Phase 4: Repository Layer Changes

1. Update `src/repository/product.rs`:
   ```diff
   use crate::models::product::{
       CategoryBrief, CreateProductRequest, Product, ProductCategory, ProductListResponse,
       ProductQueryParams, ProductResponse, UpdateProductRequest,
   };
   use anyhow::Result;
   - use rust_decimal::Decimal;
   - use sqlx::{postgres::PgRow, query_builder::QueryBuilder, Postgres, Row, types::BigDecimal};
   + use sqlx::{postgres::PgRow, query_builder::QueryBuilder, Postgres, Row, types::BigDecimal};
   ```

2. Remove the conversion functions since they're no longer needed:
   
   ```diff
   - // Helper function to convert from rust_decimal::Decimal to sqlx::types::BigDecimal
   - fn decimal_to_bigdecimal(decimal: &Decimal) -> BigDecimal {
   -     // Convert through string representation
   -     let decimal_str = decimal.to_string();
   -     decimal_str.parse::<BigDecimal>().unwrap_or_default()
   - }
   - 
   - // Helper function to convert Option<Decimal> to Option<BigDecimal>
   - fn opt_decimal_to_bigdecimal(decimal: &Option<Decimal>) -> Option<BigDecimal> {
   -     decimal.as_ref().map(|d| decimal_to_bigdecimal(d))
   - }
   ```

3. Remove conversion function calls in the repository methods:
   
   ```diff
   // In create_product method:
                   VALUES ($1, $2, $3, $4)
                   RETURNING *
                   "#,
                   req.name,
                   req.description,
   -               decimal_to_bigdecimal(&req.price),
   +               req.price,
                   req.sku
   ```

   ```diff
   // In update_product method:
                   "#,
                   req.name,
                   req.description,
   -               opt_decimal_to_bigdecimal(&req.price),
   +               req.price,
                   req.sku,
                   id
   ```

### Phase 5: Test Updates

1. Update `tests/product_api_test.rs` (and any other test files) to use `BigDecimal` instead of `Decimal`:

   ```diff
   // Import BigDecimal
   + use sqlx::types::BigDecimal;
   ```

2. Update any literal price values:

   ```diff
   // Instead of:
   - price: 29.99.into(),
   
   // Use:
   + price: BigDecimal::from_str("29.99").unwrap(),
   ```

   Make sure to add:
   ```rust
   use std::str::FromStr;
   ```

3. Modify all test cases that create or verify price values to use the `BigDecimal` type and literals.

## 3. Testing Strategy

1. **Unit Tests**:
   - Run existing unit tests after each phase of the migration to catch any immediate issues
   - Ensure validation functions work correctly with `BigDecimal` values
   - Verify conversion behavior is consistent when using BigDecimal directly

2. **Integration Tests**:
   - Run all API tests to ensure the endpoints still work correctly with the new `BigDecimal` type
   - Test CRUD operations on products with various price values
   - Verify serialization/deserialization works correctly

3. **Manual Testing**:
   - Test the API with decimal values of different precisions
   - Test edge cases like very large or small decimal values
   - Verify decimal rounding/formatting is consistent

4. **Database Migration Verification**:
   - Verify that existing data in the database is correctly interpreted when using `BigDecimal`
   - Check that new data written with `BigDecimal` is stored correctly

## 4. Rollback Plan

If issues are encountered during the migration:

1. Revert dependency changes in `Cargo.toml`
2. Restore the original model struct definitions
3. Restore the validation functions
4. Restore the repository layer code with conversion functions
5. Revert test changes

## 5. Conclusion

This migration should be relatively straightforward since SQLx already has BigDecimal support enabled, and we're just replacing one decimal type with another. The biggest challenges will be:

1. Ensuring consistent behavior between the two types (e.g., comparison operations, zero detection)
2. Adapting the validation functions to use BigDecimal's API
3. Making sure all literals and test cases are correctly updated

The benefit will be a simplified codebase without the need for conversion functions between rust_decimal::Decimal and sqlx::types::BigDecimal.