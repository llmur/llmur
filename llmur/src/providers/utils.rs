use crate::errors::{ProxyError, ProxyErrorMessage};
use crate::providers::{TransformationContext, TransformationLoss, Transformer};
use crate::routes::openai::response::ProviderResponse;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client};
use serde_json::{from_value};
use std::fmt::Debug;
use futures::StreamExt;
use axum::body::Body;
use std::sync::{Arc, Mutex};

#[tracing::instrument(
    name = "http.request",
    level = "debug",
    skip(client, generate_url_fn, request_headers)
)]
pub(crate) async fn generic_post_proxy_request<
    RequestOriginal: Transformer<RequestTransformed, RequestTransformationContext, RequestTransformationLoss>
        + Debug,
    RequestTransformationContext: TransformationContext<RequestOriginal, RequestTransformed> + Debug,
    RequestTransformationLoss: TransformationLoss<RequestOriginal, RequestTransformed>,
    RequestTransformed: serde::Serialize,
    ResponseProvider: Transformer<ResponseTransformed, ResponseTransformationContext, ResponseTransformationLoss>
        + for<'a> serde::Deserialize<'a>,
    ResponseTransformationContext: TransformationContext<ResponseProvider, ResponseTransformed> + Debug,
    ResponseTransformationLoss: TransformationLoss<ResponseProvider, ResponseTransformed>,
    ResponseTransformed,
>(
    client: &Client,
    request: RequestOriginal,
    request_context: RequestTransformationContext,
    generate_url_fn: impl Fn(RequestTransformationLoss) -> String,
    request_headers: HeaderMap,
    response_context: ResponseTransformationContext,
) -> Result<ProviderResponse<ResponseTransformed>, ProxyError> {
    debug!(
        "Transforming request {:?} into {:?}.",
        std::any::type_name::<RequestOriginal>(),
        std::any::type_name::<RequestTransformed>()
    );
    let request_transformation = request.transform(request_context);

    debug!(
        "Serializing request of type {:?} into {:?}.",
        std::any::type_name::<RequestTransformed>(),
        std::any::type_name::<serde_json::Value>()
    );
    let body = serde_json::to_value(request_transformation.result)?;

    let url = generate_url_fn(request_transformation.loss);

    debug!("Executing POST request against url {:?}.", url);
    let response = client
        .post(url)
        .headers(request_headers)
        .json(&body)
        .send()
        .await?;

    debug!(
        "POST request executed successfully. Got status {:?}.",
        response.status()
    );

    let status = response.status();

    if status.is_success() {
        debug!(
            "Transforming response {:?} into {:?}.",
            std::any::type_name::<ResponseProvider>(),
            std::any::type_name::<ResponseTransformed>()
        );

        let json_value =  response.json::<serde_json::Value>().await?;

        match from_value::<ResponseProvider>(json_value.clone()) {
            Ok(deserialized) => {
                let data = deserialized.transform(response_context);
                Ok(ProviderResponse::DecodedResponse { data: data.result, status_code: status })
            }
            Err(_) => {
                Ok(ProviderResponse::JsonResponse { data: json_value, status_code: status })
            }
        }
    } else {
        debug!(
            "Got error from provider {:?}.",
            std::any::type_name::<ResponseProvider>()
        );
        // Try to get the body as bytes first (doesn't consume response)
        let body_bytes = response.bytes().await?;

        // Try to parse as JSON
        match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            Ok(value) => {
                Err(ProxyError::ProxyReturnError(status, ProxyErrorMessage::Json(value)))?
            }
            Err(_) => {
                // Fall back to text
                let text = String::from_utf8_lossy(&body_bytes).to_string();
                Err(ProxyError::ProxyReturnError(status, ProxyErrorMessage::Text(text)))?
            }
        }
    }
}

#[tracing::instrument(
    name = "http.request.stream",
    level = "debug",
    skip(client, generate_url_fn, request_headers)
)]
pub(crate) async fn generic_post_proxy_request_stream<
    RequestOriginal: Transformer<RequestTransformed, RequestTransformationContext, RequestTransformationLoss>
        + Debug,
    RequestTransformationContext: TransformationContext<RequestOriginal, RequestTransformed> + Debug,
    RequestTransformationLoss: TransformationLoss<RequestOriginal, RequestTransformed>,
    RequestTransformed: serde::Serialize,
    ResponseTransformed,
>(
    client: &Client,
    request: RequestOriginal,
    request_context: RequestTransformationContext,
    generate_url_fn: impl Fn(RequestTransformationLoss) -> String,
    request_headers: HeaderMap,
) -> Result<ProviderResponse<ResponseTransformed>, ProxyError> {
    debug!(
        "Transforming request {:?} into {:?}.",
        std::any::type_name::<RequestOriginal>(),
        std::any::type_name::<RequestTransformed>()
    );
    let request_transformation = request.transform(request_context);

    debug!(
        "Serializing request of type {:?} into {:?}.",
        std::any::type_name::<RequestTransformed>(),
        std::any::type_name::<serde_json::Value>()
    );
    let body = serde_json::to_value(request_transformation.result)?;

    let url = generate_url_fn(request_transformation.loss);

    debug!("Executing POST request against url {:?}.", url);
    let response = client
        .post(url)
        .headers(request_headers)
        .json(&body)
        .send()
        .await?;

    debug!(
        "POST request executed successfully. Got status {:?}.",
        response.status()
    );

    let status = response.status();

    if status.is_success() {
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        let stream = response.bytes_stream().map(|item| {
            item.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
        });
        let body = Body::from_stream(stream);

        Ok(ProviderResponse::Stream {
            body: Arc::new(Mutex::new(Some(body))),
            status_code: status,
            content_type,
        })
    } else {
        debug!(
            "Got error from provider {:?}.",
            std::any::type_name::<ResponseTransformed>()
        );
        let body_bytes = response.bytes().await?;

        match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            Ok(value) => {
                Err(ProxyError::ProxyReturnError(status, ProxyErrorMessage::Json(value)))?
            }
            Err(_) => {
                let text = String::from_utf8_lossy(&body_bytes).to_string();
                Err(ProxyError::ProxyReturnError(status, ProxyErrorMessage::Text(text)))?
            }
        }
    }
}
