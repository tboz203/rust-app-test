pub mod product;

use axum::Json;
use validator::Validate;

use crate::error::ApiError;

/// Validates a request body against its validation rules
pub fn validate_request<T>(value: &T) -> Result<(), ApiError>
where
    T: Validate,
{
    if let Err(validation_errors) = value.validate() {
        let error_message = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_msgs: Vec<String> = errors
                    .iter()
                    .map(|error| error.message.as_ref().map_or_else(
                        || format!("{} is invalid", field),
                        |msg| msg.to_string(),
                    ))
                    .collect();
                format!("{}: {}", field, error_msgs.join(", "))
            })
            .collect::<Vec<String>>()
            .join("; ");

        return Err(ApiError::Validation(error_message));
    }

    Ok(())
}

/// Extracts and validates JSON from request
pub async fn validate_json<T>(json: Json<T>) -> Result<T, ApiError>
where
    T: Validate,
{
    // Extract inner value and validate it, not the Json wrapper
    validate_request(&json.0)?;
    Ok(json.0)
}