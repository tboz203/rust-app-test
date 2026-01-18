use bigdecimal::{BigDecimal, Signed, Zero};
use validator::ValidationError;


/// Validates that a Decimal value is positive (greater than zero)
pub fn validate_positive_decimal(decimal: &BigDecimal) -> Result<(), ValidationError> {
    if decimal.is_negative() || decimal.is_zero() {
        let mut error = ValidationError::new("positive_decimal");
        error.message = Some(std::borrow::Cow::from("Price must be a positive number"));
        return Err(error);
    }
    Ok(())
}

/// Validates that an Option<Decimal> value is positive if it exists
pub fn validate_optional_decimal(decimal: &Option<BigDecimal>) -> Result<(), ValidationError> {
    match decimal {
        Some(d) => validate_positive_decimal(d),
        None => Ok(()),
    }
}

/// Validates that a vector is not empty
pub fn validate_non_empty_vec<T>(vec: &[T]) -> Result<(), ValidationError> {
    if vec.is_empty() {
        let mut error = ValidationError::new("non_empty");
        error.message = Some(std::borrow::Cow::from("Must contain at least one item"));
        return Err(error);
    }
    Ok(())
}

/// Validates that an optional vector is not empty if it exists
pub fn validate_optional_non_empty_vec<T>(vec: &Option<Vec<T>>) -> Result<(), ValidationError> {
    match vec {
        Some(v) => validate_non_empty_vec(v),
        None => Ok(()),
    }
}