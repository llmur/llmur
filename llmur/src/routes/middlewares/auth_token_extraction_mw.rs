use crate::errors::AuthorizationHeaderExtractionError;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug)]
pub enum AuthorizationHeader {
    Bearer(String),
}

pub type AuthorizationHeaderExtractionResult = Result<AuthorizationHeader, AuthorizationHeaderExtractionError>;

pub(crate) fn auth_token_extraction_mw(
    mut request: Request,
    next: Next,
) -> Pin<Box<dyn Future<Output=Response> + Send + 'static>> {
    println!("Executing Auth Token Extraction Middleware");
    Box::pin(async move {
        let headers = request.headers();
        let result: AuthorizationHeaderExtractionResult = match headers.get("Authorization") {
            None => {
                Err(AuthorizationHeaderExtractionError::AuthorizationHeaderNotProvided)
            }
            Some(auth_header) => {
                extract_bearer_token(auth_header)
                    .map(|v| AuthorizationHeader::Bearer(v.to_string()))
            }
        };
        println!("Auth Token Extraction Result: {:?}", result);
        request.extensions_mut().insert(result);
        next.run(request).await
    })
}

pub fn extract_bearer_token(header: &HeaderValue) -> Result<&str, AuthorizationHeaderExtractionError> {
    let input = header.to_str().map_err(|_| AuthorizationHeaderExtractionError::InvalidAuthorizationHeader)?;

    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(AuthorizationHeaderExtractionError::InvalidAuthorizationHeader);
    }

    if parts[0] != "Bearer" {
        return Err(AuthorizationHeaderExtractionError::InvalidAuthorizationHeader);
    }

    Ok(parts[1])
}

