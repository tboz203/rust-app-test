use anyhow::Result;
use bigdecimal::{BigDecimal, Signed};
use validator::ValidationError;

/// Validates that a Decimal value is positive
pub fn validate_decimal_positive(decimal: &BigDecimal) -> Result<(), ValidationError> {
    if !decimal.is_positive() {
        let mut error = ValidationError::new("positive_decimal");
        error.message = Some(std::borrow::Cow::from("Price must be a positive number"));
        return Err(error);
    }
    Ok(())
}
