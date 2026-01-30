use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// region: --- Core request/response shared structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    Text(String),
    Items(Vec<InputItem>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputItem {
    EasyMessage(EasyInputMessage),
    Item(Item),
    Reference(ItemReferenceParam),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EasyInputMessage {
    pub role: EasyInputMessageRole,
    pub content: EasyInputMessageContent,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EasyInputMessageRole {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EasyInputMessageContent {
    Text(String),
    ContentList(Vec<InputContent>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub role: InputMessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    pub content: Vec<InputContent>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMessageRole {
    User,
    System,
    Developer,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputContent {
    #[serde(rename = "input_text")]
    Text { text: String },
    #[serde(rename = "input_image")]
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        detail: InputImageDetail,
    },
    #[serde(rename = "input_file")]
    File {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputImageDetail {
    Low,
    High,
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    InProgress,
    Completed,
    Incomplete,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Item {
    InputMessage(InputMessage),
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ComputerCallOutputItem(ComputerCallOutputItemParam),
    WebSearchToolCall(WebSearchToolCall),
    FunctionToolCall(FunctionToolCall),
    FunctionCallOutputItem(FunctionCallOutputItemParam),
    ReasoningItem(ReasoningItem),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ItemReferenceParam {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub reference_type: Option<ItemReferenceType>,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemReferenceType {
    ItemReference,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextResponseFormatConfiguration>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TextResponseFormatConfiguration {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_object")]
    JsonObject,
    #[serde(rename = "json_schema")]
    JsonSchema {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        name: String,
        schema: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Includable {
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageImageUrl,
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputOutputImageUrl,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummaryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_summary: Option<ReasoningSummaryMode>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummaryMode {
    Auto,
    Concise,
    Detailed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasoningItem {
    #[serde(rename = "type")]
    pub item_type: ReasoningItemType,
    pub id: String,
    pub summary: Vec<ReasoningSummaryPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningItemType {
    Reasoning,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasoningSummaryPart {
    #[serde(rename = "type")]
    pub part_type: ReasoningSummaryPartType,
    pub text: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSummaryPartType {
    SummaryText,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Truncation {
    Auto,
    Disabled,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Auto,
    Default,
    Flex,
}
// endregion: --- Core request/response shared structs

// region: --- Tool configuration structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "file_search")]
    FileSearch(FileSearchTool),
    #[serde(rename = "function")]
    Function(FunctionTool),
    #[serde(rename = "web_search_preview")]
    WebSearchPreview(WebSearchPreviewTool),
    #[serde(rename = "web_search_preview_2025_03_11")]
    WebSearchPreview20250311(WebSearchPreviewTool),
    #[serde(rename = "computer_use_preview")]
    ComputerUsePreview(ComputerUsePreviewTool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileSearchTool {
    pub vector_store_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<RankingOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Filters>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RankingOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<RankingRanker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RankingRanker {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "default-2024-11-15")]
    Default20241115,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filters {
    Comparison(ComparisonFilter),
    Compound(CompoundFilter),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComparisonFilter {
    #[serde(rename = "type")]
    pub operator: ComparisonOperator,
    pub key: String,
    pub value: FilterValue,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompoundFilter {
    #[serde(rename = "type")]
    pub operator: CompoundOperator,
    pub filters: Vec<Filters>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompoundOperator {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchPreviewTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<ApproximateLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<WebSearchContextSize>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApproximateLocation {
    #[serde(rename = "type")]
    pub location_type: ApproximateLocationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApproximateLocationType {
    Approximate,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerUsePreviewTool {
    pub environment: ComputerEnvironment,
    pub display_width: u64,
    pub display_height: u64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputerEnvironment {
    Windows,
    Mac,
    Linux,
    Ubuntu,
    Browser,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceMode),
    Tool(ToolChoiceType),
    Function(ToolChoiceFunction),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceType {
    #[serde(rename = "type")]
    pub tool_type: ToolChoiceTypeValue,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ToolChoiceTypeValue {
    #[serde(rename = "file_search")]
    FileSearch,
    #[serde(rename = "web_search_preview")]
    WebSearchPreview,
    #[serde(rename = "web_search_preview_2025_03_11")]
    WebSearchPreview20250311,
    #[serde(rename = "computer_use_preview")]
    ComputerUsePreview,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    #[serde(rename = "type")]
    pub tool_type: ToolChoiceFunctionType,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceFunctionType {
    Function,
}
// endregion: --- Tool configuration structs

// region: --- Output content structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputItem {
    OutputMessage(OutputMessage),
    FileSearchToolCall(FileSearchToolCall),
    FunctionToolCall(FunctionToolCall),
    WebSearchToolCall(WebSearchToolCall),
    ComputerToolCall(ComputerToolCall),
    ReasoningItem(ReasoningItem),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputMessage {
    pub id: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
    pub role: OutputMessageRole,
    pub content: Vec<OutputContent>,
    pub status: ItemStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMessageRole {
    Assistant,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputContent {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        annotations: Vec<Annotation>,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    #[serde(rename = "file_citation")]
    FileCitation { file_id: String, index: u64 },
    #[serde(rename = "url_citation")]
    UrlCitation {
        url: String,
        start_index: u64,
        end_index: u64,
        title: String,
    },
    #[serde(rename = "file_path")]
    FilePath { file_id: String, index: u64 },
}
// endregion: --- Output content structs

// region: --- Tool call output structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: FileSearchToolCallType,
    pub status: FileSearchToolCallStatus,
    pub queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<FileSearchToolCallResult>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileSearchToolCallType {
    FileSearchCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileSearchToolCallStatus {
    InProgress,
    Searching,
    Completed,
    Incomplete,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileSearchToolCallResult {
    pub file_id: String,
    pub text: String,
    pub filename: String,
    pub attributes: Option<VectorStoreFileAttributes>,
    pub score: f64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileAttributes {
    #[serde(flatten)]
    pub attributes: HashMap<String, VectorStoreFileAttributeValue>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorStoreFileAttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: CodeInterpreterToolCallType,
    pub code: String,
    pub status: CodeInterpreterCallStatus,
    pub results: Vec<CodeInterpreterToolOutput>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeInterpreterToolCallType {
    CodeInterpreterCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeInterpreterCallStatus {
    InProgress,
    Interpreting,
    Completed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterToolOutput {
    Logs(CodeInterpreterTextOutput),
    Files(CodeInterpreterFileOutput),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterTextOutput {
    #[serde(rename = "type")]
    pub output_type: CodeInterpreterTextOutputType,
    pub logs: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeInterpreterTextOutputType {
    Logs,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterFileOutput {
    #[serde(rename = "type")]
    pub output_type: CodeInterpreterFileOutputType,
    pub files: Vec<CodeInterpreterFile>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeInterpreterFileOutputType {
    Files,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterFile {
    pub mime_type: String,
    pub file_id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: WebSearchToolCallType,
    pub status: WebSearchToolCallStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolCallType {
    WebSearchCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolCallStatus {
    InProgress,
    Searching,
    Completed,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: FunctionToolCallType,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionToolCallType {
    FunctionCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ComputerToolCallType,
    pub call_id: String,
    pub action: ComputerAction,
    pub pending_safety_checks: Vec<ComputerToolCallSafetyCheck>,
    pub status: ItemStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerToolCallType {
    ComputerCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerToolCallSafetyCheck {
    pub id: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComputerAction {
    #[serde(rename = "click")]
    Click { button: MouseButton, x: i64, y: i64 },
    #[serde(rename = "double_click")]
    DoubleClick { x: i64, y: i64 },
    #[serde(rename = "drag")]
    Drag { path: Vec<Coordinate> },
    #[serde(rename = "keypress")]
    KeyPress { keys: Vec<String> },
    #[serde(rename = "move")]
    Move { x: i64, y: i64 },
    #[serde(rename = "screenshot")]
    Screenshot,
    #[serde(rename = "scroll")]
    Scroll {
        x: i64,
        y: i64,
        scroll_x: i64,
        scroll_y: i64,
    },
    #[serde(rename = "type")]
    Type { text: String },
    #[serde(rename = "wait")]
    Wait,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Wheel,
    Back,
    Forward,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerCallOutputItemParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    #[serde(rename = "type")]
    pub output_type: ComputerCallOutputItemType,
    pub output: ComputerScreenshotImage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: Option<Vec<ComputerCallSafetyCheckParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerCallOutputItemType {
    ComputerCallOutput,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerCallSafetyCheckParam {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerScreenshotImage {
    #[serde(rename = "type")]
    pub image_type: ComputerScreenshotType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerScreenshotType {
    ComputerScreenshot,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionCallOutputItemParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    #[serde(rename = "type")]
    pub output_type: FunctionCallOutputItemType,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallOutputItemType {
    FunctionCallOutput,
}
// endregion: --- Tool call output structs
