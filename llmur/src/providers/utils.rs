use std::fmt::Debug;
use log::debug;
use reqwest::{Client, StatusCode};
use reqwest::header::HeaderMap;
use serde_json::json;
use crate::providers::{TransformationContext, TransformationLoss, Transformer};


#[tracing::instrument(
    name = "http.request",
    level = "debug",
    skip(client, generate_url_fn, request_headers)
)]
pub(crate) async fn generic_post_proxy_request<
    RequestOriginal:
    Transformer<RequestTransformed, RequestTransformationContext, RequestTransformationLoss> + Debug,
    RequestTransformationContext: TransformationContext<RequestOriginal, RequestTransformed> + Debug,
    RequestTransformationLoss: TransformationLoss<RequestOriginal, RequestTransformed>,
    RequestTransformed: serde::Serialize,
    ResponseProvider:
    Transformer<ResponseTransformed, ResponseTransformationContext, ResponseTransformationLoss> +
        for<'a> serde::Deserialize<'a>,
    ResponseTransformationContext: TransformationContext<ResponseProvider, ResponseTransformed> + Debug,
    ResponseTransformationLoss: TransformationLoss<ResponseProvider, ResponseTransformed>,
    ResponseTransformed
>(
    client: &Client,
    request: RequestOriginal,
    request_context: RequestTransformationContext,
    generate_url_fn: impl Fn(RequestTransformationLoss) -> String,
    request_headers: HeaderMap,
    response_context: ResponseTransformationContext,
) -> Result<(ResponseTransformed, StatusCode), ProxyRequestError> {
    debug!("Transforming request {:?} into {:?}.", std::any::type_name::<RequestOriginal>(), std::any::type_name::<RequestTransformed>());
    let request_transformation = request.transform(request_context);

    debug!("Serializing request of type {:?} into {:?}.", std::any::type_name::<RequestTransformed>(), std::any::type_name::<serde_json::Value>());
    let body = serde_json::to_value(request_transformation.result).map_err(|e| ProxyRequestError::SerdeError(e.to_string()))?;

    let url = generate_url_fn(request_transformation.loss);

    debug!("Executing POST request against url {:?}.", url);
    let response = client
        .post(url)
        .headers(request_headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| ProxyRequestError::ReqwestError(e.to_string()))?;

    debug!("POST request executed successfully. Got status {:?}.", response.status());
    
    let status = response.status();

    if status.is_success() {
        debug!("Deserializing provider response into {:?}.", std::any::type_name::<ResponseProvider>());
        let deserialized_response = response
            .json::<ResponseProvider>()
            .await.map_err(|e| ProxyRequestError::ReqwestSerdeError(e.to_string()))?;

        debug!("Transforming response {:?} into {:?}.", std::any::type_name::<ResponseProvider>(), std::any::type_name::<ResponseTransformed>());
        let response_transformation = deserialized_response.transform(response_context);

        Ok((response_transformation.result, status))
    } else {
        let status = status.as_u16();

        debug!("Got error from provider {:?}.", std::any::type_name::<ResponseProvider>());
        match response.text().await {
            Ok(text_error) => {
                let deserialized_error: serde_json::Value = serde_json::from_str(&text_error).unwrap_or(json!({"error": text_error.trim()}));
                Err(ProxyRequestError::ProxyReturnError(status, deserialized_error))?
            }
            Err(_) => {
                Err(ProxyRequestError::ProxyReturnError(status, json!({"error": "unknown"})))?
            }
        }
    }
}