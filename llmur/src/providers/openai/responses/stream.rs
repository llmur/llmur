use serde::{Deserialize, Serialize};

use crate::providers::openai::responses::response::Response;
use crate::providers::openai::responses::types::{
    Annotation, CodeInterpreterToolCall, OutputContent, OutputItem, ReasoningSummaryPart,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    #[serde(rename = "response.audio.delta")]
    AudioDelta {
        #[serde(skip_serializing_if = "Option::is_none")]
        response_id: Option<String>,
        delta: String,
    },
    #[serde(rename = "response.audio.done")]
    AudioDone { response_id: String },
    #[serde(rename = "response.audio.transcript.delta")]
    AudioTranscriptDelta { response_id: String, delta: String },
    #[serde(rename = "response.audio.transcript.done")]
    AudioTranscriptDone { response_id: String },
    #[serde(rename = "response.code_interpreter_call.code.delta")]
    CodeInterpreterCallCodeDelta {
        response_id: String,
        output_index: u64,
        delta: String,
    },
    #[serde(rename = "response.code_interpreter_call.code.done")]
    CodeInterpreterCallCodeDone {
        response_id: String,
        output_index: u64,
        code: String,
    },
    #[serde(rename = "response.code_interpreter_call.completed")]
    CodeInterpreterCallCompleted {
        response_id: String,
        output_index: u64,
        code_interpreter_call: CodeInterpreterToolCall,
    },
    #[serde(rename = "response.code_interpreter_call.in_progress")]
    CodeInterpreterCallInProgress {
        response_id: String,
        output_index: u64,
        code_interpreter_call: CodeInterpreterToolCall,
    },
    #[serde(rename = "response.code_interpreter_call.interpreting")]
    CodeInterpreterCallInterpreting {
        response_id: String,
        output_index: u64,
        code_interpreter_call: CodeInterpreterToolCall,
    },
    #[serde(rename = "response.completed")]
    Completed { response: Response },
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        item_id: String,
        output_index: u64,
        content_index: u64,
        part: OutputContent,
    },
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        item_id: String,
        output_index: u64,
        content_index: u64,
        part: OutputContent,
    },
    #[serde(rename = "response.created")]
    Created { response: Response },
    #[serde(rename = "error")]
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        param: Option<String>,
    },
    #[serde(rename = "response.failed")]
    Failed { response: Response },
    #[serde(rename = "response.in_progress")]
    InProgress { response: Response },
    #[serde(rename = "response.incomplete")]
    Incomplete { response: Response },
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded { output_index: u64, item: OutputItem },
    #[serde(rename = "response.output_item.done")]
    OutputItemDone { output_index: u64, item: OutputItem },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: String,
        output_index: u64,
        delta: String,
    },
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        item_id: String,
        output_index: u64,
        arguments: String,
    },
    #[serde(rename = "response.file_search_call.in_progress")]
    FileSearchCallInProgress { output_index: u64, item_id: String },
    #[serde(rename = "response.file_search_call.searching")]
    FileSearchCallSearching { output_index: u64, item_id: String },
    #[serde(rename = "response.file_search_call.completed")]
    FileSearchCallCompleted { output_index: u64, item_id: String },
    #[serde(rename = "response.web_search_call.in_progress")]
    WebSearchCallInProgress { output_index: u64, item_id: String },
    #[serde(rename = "response.web_search_call.searching")]
    WebSearchCallSearching { output_index: u64, item_id: String },
    #[serde(rename = "response.web_search_call.completed")]
    WebSearchCallCompleted { output_index: u64, item_id: String },
    #[serde(rename = "response.reasoning_summary_part.added")]
    ReasoningSummaryPartAdded {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        part: ReasoningSummaryPart,
    },
    #[serde(rename = "response.reasoning_summary_part.done")]
    ReasoningSummaryPartDone {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        part: ReasoningSummaryPart,
    },
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        delta: String,
    },
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        item_id: String,
        output_index: u64,
        summary_index: u64,
        text: String,
    },
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        delta: String,
    },
    #[serde(rename = "response.refusal.done")]
    RefusalDone {
        item_id: String,
        output_index: u64,
        content_index: u64,
        refusal: String,
    },
    #[serde(rename = "response.output_text.annotation.added")]
    TextAnnotationDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        annotation_index: u64,
        annotation: Annotation,
    },
    #[serde(rename = "response.output_text.delta")]
    TextDelta {
        item_id: String,
        output_index: u64,
        content_index: u64,
        delta: String,
    },
    #[serde(rename = "response.output_text.done")]
    TextDone {
        item_id: String,
        output_index: u64,
        content_index: u64,
        text: String,
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
          "delta": "Hi"
        }"#;

        let event: ResponseStreamEvent = serde_json::from_str(json).expect("parse event");
        match event {
            ResponseStreamEvent::TextDelta { delta, .. } => assert_eq!(delta, "Hi"),
            _ => panic!("unexpected event variant"),
        }
    }
}
