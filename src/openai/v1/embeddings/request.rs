#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmbeddingsRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a single request, pass an array of strings or array of token arrays. The input must not exceed the max input tokens for the model (8192 tokens for text-embedding-ada-002), cannot be an empty string, and any array must be 2048 dimensions or less. [Example Python](https://cookbook.openai.com/examples/how_to_count_tokens_with_tiktoken) code for counting tokens.
    pub input: EmbeddingsRequestInput,

    /// ID of the model to use. You can use the List models API to see all of your available models, or see our Model overview for descriptions of them.
    pub model: String,

    /// The format to return the embeddings in. Can be either float or base64.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub encoding_format: Option<String>,

    /// The number of dimensions the resulting output embeddings should have. Only supported in text-embedding-3 and later models.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub dimensions: Option<i64>,
    
    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](https://platform.openai.com/docs/guides/safety-best-practices).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
pub enum EmbeddingsRequestInput {
    String(String),
    ArrayString(Vec<String>),
    ArrayInt(Vec<i64>),
    ArrayArrayInt(Vec<Vec<i64>>),
}



#[cfg(test)]
mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early tests.

    use serde_json::json;
    use super::*;

    #[test]
    fn test_embeddings_openai_example_schema_01_decode_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_request = json!({
            "input": "The food was delicious and the waiter...",
            "model": "text-embedding-ada-002",
            "encoding_format": "float"
          }).to_string();
    
        let _: EmbeddingsRequest = serde_json::from_str(&fx_request).unwrap();
        
        Ok(())
    }
}

// endregion:    --- Tests