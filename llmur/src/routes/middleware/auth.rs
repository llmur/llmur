use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::errors::AuthenticationError;

#[derive(Clone, Debug)]
pub enum AuthorizationHeader {
    Bearer(String),
}

pub type AuthorizationHeaderExtractionResult = Result<AuthorizationHeader, Arc<AuthenticationError>>;

pub(crate) fn auth_token_extraction_mw(
    mut request: Request,
    next: Next,
) -> Pin<Box<dyn Future<Output=Response> + Send + 'static>> {
    println!("Executing Auth Token Extraction Middleware");
    Box::pin(async move {
        let headers = request.headers();
        let result: AuthorizationHeaderExtractionResult = match headers.get("Authorization") {
            None => {
                Err(Arc::new(AuthenticationError::AuthHeaderNotProvided))
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

pub fn extract_bearer_token(header: &HeaderValue) -> Result<&str, Arc<AuthenticationError>> {
    let input = header.to_str().map_err(|_| AuthenticationError::InvalidAuthBearer)?;

    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(Arc::new(AuthenticationError::InvalidAuthBearer));
    }

    if parts[0] != "Bearer" {
        return Err(Arc::new(AuthenticationError::InvalidAuthBearer));
    }

    Ok(parts[1])
}

