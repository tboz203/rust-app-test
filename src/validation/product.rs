use crate::error::ApiError;
use crate::models::product::{CreateProductRequest, UpdateProductRequest};

/// Validate a create product request
pub fn validate_create_product(req: &CreateProductRequest) -> Result<(), ApiError> {
    // Additional validation beyond struct-level validation can be added here
    
    // Ensure there's at least one category for a new product
    if req.category_ids.is_empty() {
        return Err(ApiError::Validation("Product must belong to at least one category".to_string()));
    }

    // Price validation (additional to the struct validation)
    if req.price.is_sign_negative() || req.price.is_zero() {
        return Err(ApiError::Validation("Price must be positive".to_string()));
    }

    Ok(())
}

/// Validate an update product request
pub fn validate_update_product(req: &UpdateProductRequest) -> Result<(), ApiError> {
    // Additional validation beyond struct-level validation
    
    // Check that price is positive if provided
    if let Some(price) = req.price {
        if price.is_sign_negative() || price.is_zero() {
            return Err(ApiError::Validation("Price must be positive".to_string()));
        }
    }
    
    // Ensure category list isn't empty if provided
    if let Some(category_ids) = &req.category_ids {
        if category_ids.is_empty() {
            return Err(ApiError::Validation("Product must belong to at least one category".to_string()));
        }
    }
    
    Ok(())
}