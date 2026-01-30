use serde::{Deserialize, Serialize};

use crate::providers::azure::openai::v1::responses::response::Response;
use crate::providers::azure::openai::v1::responses::types::{
    Annotation, OutputContent, OutputItem, ReasoningSummaryPart, ResponseLogProb,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    #[serde(rename = "response.audio.delta")]
    AudioDelta { sequence_number: u64, delta: String },
    #[serde(rename = "response.audio.transcript.delta")]
    AudioTranscriptDelta { sequence_number: u64, delta: String },
    #[serde(rename = "response.code_interpreter_call_code.delta")]
    CodeInterpreterCallCodeDelta {
        output_index: u64,
        item_id: String,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.code_interpreter_call.in_progress")]
    CodeInterpreterCallInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.code_interpreter_call.interpreting")]
    CodeInterpreterCallInterpreting {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        item_id: String,
        output_index: u64,
        content_index: u64,
        part: OutputContent,
        sequence_number: u64,
    },
    #[serde(rename = "response.created")]
    Created {
        response: Response,
        sequence_number: u64,
    },
    #[serde(rename = "response.custom_tool_call_input.delta")]
    CustomToolCallInputDelta {
        output_index: u64,
        item_id: String,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "error")]
    Error {
        code: Option<String>,
        message: String,
        param: Option<String>,
        sequence_number: u64,
    },
    #[serde(rename = "response.failed")]
    Failed {
        response: Response,
        sequence_number: u64,
    },
    #[serde(rename = "response.file_search_call.in_progress")]
    FileSearchCallInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.file_search_call.searching")]
    FileSearchCallSearching {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: String,
        output_index: u64,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.image_generation_call.generating")]
    ImageGenerationCallGenerating {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.image_generation_call.in_progress")]
    ImageGenerationCallInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.image_generation_call.partial_image")]
    ImageGenerationCallPartialImage {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
        partial_image_index: u64,
        partial_image_b64: String,
    },
    #[serde(rename = "response.in_progress")]
    InProgress {
        response: Response,
        sequence_number: u64,
    },
    #[serde(rename = "response.incomplete")]
    Incomplete {
        response: Response,
        sequence_number: u64,
    },
    #[serde(rename = "response.mcp_call_arguments.delta")]
    McpCallArgumentsDelta {
        output_index: u64,
        item_id: String,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.mcp_call.failed")]
    McpCallFailed {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.mcp_call.in_progress")]
    McpCallInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.mcp_list_tools.failed")]
    McpListToolsFailed {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.mcp_list_tools.in_progress")]
    McpListToolsInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        output_index: u64,
        item: OutputItem,
        sequence_number: u64,
    },
    #[serde(rename = "response.output_text.annotation.added")]
    OutputTextAnnotationAdded {
        item_id: String,
        output_index: u64,
        content_index: u64,
        annotation_index: u64,
        annotation: Annotation,
        sequence_number: u64,
    },
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        delta: String,
        sequence_number: u64,
        logprobs: Vec<ResponseLogProb>,
    },
    #[serde(rename = "response.queued")]
    Queued {
        response: Response,
        sequence_number: u64,
    },
    #[serde(rename = "response.reasoning_summary_part.added")]
    ReasoningSummaryPartAdded {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        part: ReasoningSummaryPart,
        sequence_number: u64,
    },
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        delta: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.web_search_call.in_progress")]
    WebSearchCallInProgress {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
    #[serde(rename = "response.web_search_call.searching")]
    WebSearchCallSearching {
        output_index: u64,
        item_id: String,
        sequence_number: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::ResponseStreamEvent;

    #[test]
    fn stream_event_output_text_delta() {
        let json = r#"{
          "type": "response.output_text.delta",
          "item_id": "msg_123",
          "output_index": 0,
          "content_index": 0,
          "delta": "Hi",
          "sequence_number": 1,
          "logprobs": []
        }"#;

        let event: ResponseStreamEvent = serde_json::from_str(json).expect("parse event");
        match event {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => assert_eq!(delta, "Hi"),
            _ => panic!("unexpected event variant"),
        }
    }
}
