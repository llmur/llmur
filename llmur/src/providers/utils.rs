use reqwest::Client;
use reqwest::header::HeaderMap;
use serde_json::json;
use crate::errors::ProxyRequestError;
use crate::providers::{TransformationContext, TransformationLoss, Transformer};

pub(crate) async fn generic_post_proxy_request<
    RequestOriginal:
    Transformer<RequestTransformed, RequestTransformationContext, RequestTransformationLoss>,
    RequestTransformationContext: TransformationContext<RequestOriginal, RequestTransformed>,
    RequestTransformationLoss: TransformationLoss<RequestOriginal, RequestTransformed>,
    RequestTransformed: serde::Serialize,
    ResponseProvider:
    Transformer<ResponseTransformed, ResponseTransformationContext, ResponseTransformationLoss> +
        for<'a> serde::Deserialize<'a>,
    ResponseTransformationContext: TransformationContext<ResponseProvider, ResponseTransformed>,
    ResponseTransformationLoss: TransformationLoss<ResponseProvider, ResponseTransformed>,
    ResponseTransformed
>(
    client: &Client,
    request: RequestOriginal,
    request_context: RequestTransformationContext,
    generate_url_fn: impl Fn(RequestTransformationLoss) -> String,
    request_headers: HeaderMap,
    response_context: ResponseTransformationContext,
) -> Result<ResponseTransformed, ProxyRequestError> {
    println!("Transforming request {:?} into {:?}.", std::any::type_name::<RequestOriginal>(), std::any::type_name::<RequestTransformed>());
    let request_transformation = request.transform(request_context);

    println!("Serializing request of type {:?} into {:?}.", std::any::type_name::<RequestTransformed>(), std::any::type_name::<serde_json::Value>());
    let body = serde_json::to_value(request_transformation.result)?;

    println!("Generating endpoint url");
    let url = generate_url_fn(request_transformation.loss);

    println!("Executing POST request against url {:?}.", url);
    let response = client
        .post(url)
        .headers(request_headers)
        .json(&body)
        .send()
        .await?;
    println!("POST request executed successfully. Got status {:?}.", response.status());

    if response.status().is_success() {
        println!("Deserializing provider response into {:?}.", std::any::type_name::<ResponseProvider>());
        let deserialized_response = response
            .json::<ResponseProvider>()
            .await.map_err(|e| ProxyRequestError::ReqwestSerdeError(e))?;

        println!("Transforming response {:?} into {:?}.", std::any::type_name::<ResponseProvider>(), std::any::type_name::<ResponseTransformed>());
        let response_transformation = deserialized_response.transform(response_context);

        Ok(response_transformation.result)
    } else {
        let status = response.status().as_u16();

        println!("Getting error response received from provider {:?}.", std::any::type_name::<ResponseProvider>());
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