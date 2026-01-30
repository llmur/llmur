use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::providers::azure::openai::v1::common::PromptCacheRetention;
use crate::providers::azure::openai::v1::common::{ReasoningEffort, Verbosity};

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
    Message(EasyInputMessage),
    Reference(ItemReferenceParam),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EasyInputMessage {
    pub role: EasyInputMessageRole,
    pub content: EasyInputMessageContent,
    #[serde(rename = "type")]
    pub message_type: MessageType,
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
        detail: ImageDetail,
    },
    #[serde(rename = "input_file")]
    File {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Message,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ItemReferenceParam {
    #[serde(rename = "type")]
    pub reference_type: ItemReferenceType,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemReferenceType {
    ItemReference,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<PromptVariables>,
}

pub type PromptVariables = HashMap<String, PromptVariable>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptVariable {
    Text(String),
    Content(InputContent),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConversationParam {
    Id(String),
    Object(ConversationParamObject),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConversationParamObject {
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConversationReference {
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<TextResponseFormatConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
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
pub struct ResponseStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
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
pub enum ReasoningSummaryMode {
    Auto,
    Concise,
    Detailed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Truncation {
    Auto,
    Disabled,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Includable {
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    #[serde(rename = "web_search_call.results")]
    WebSearchCallResults,
    #[serde(rename = "web_search_call.action.sources")]
    WebSearchCallActionSources,
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageImageUrl,
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputOutputImageUrl,
    #[serde(rename = "code_interpreter_call.outputs")]
    CodeInterpreterCallOutputs,
    #[serde(rename = "reasoning.encrypted_content")]
    ReasoningEncryptedContent,
    #[serde(rename = "message.output_text.logprobs")]
    MessageOutputTextLogprobs,
}
// endregion: --- Core request/response shared structs

// region: --- Tool configuration structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    #[serde(rename = "code_interpreter")]
    CodeInterpreter(CodeInterpreterTool),
    #[serde(rename = "function")]
    Function(FunctionTool),
    #[serde(rename = "file_search")]
    FileSearch(FileSearchTool),
    #[serde(rename = "computer_use_preview")]
    ComputerUsePreview(ComputerUsePreviewTool),
    #[serde(rename = "web_search")]
    WebSearch(WebSearchTool),
    #[serde(rename = "mcp")]
    Mcp(McpTool),
    #[serde(rename = "image_generation")]
    ImageGeneration(ImageGenerationTool),
    #[serde(rename = "local_shell")]
    LocalShell(LocalShellTool),
    #[serde(rename = "shell")]
    Shell(ShellTool),
    #[serde(rename = "custom")]
    Custom(CustomTool),
    #[serde(rename = "web_search_preview")]
    WebSearchPreview(WebSearchPreviewTool),
    #[serde(rename = "apply_patch")]
    ApplyPatch(ApplyPatchTool),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterTool {
    pub container: CodeInterpreterContainer,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterContainer {
    Id(String),
    Auto(CodeInterpreterContainerAuto),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterContainerAuto {
    #[serde(rename = "type")]
    pub container_type: CodeInterpreterContainerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<ContainerMemoryLimit>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeInterpreterContainerType {
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ContainerMemoryLimit {
    #[serde(rename = "1g")]
    G1,
    #[serde(rename = "4g")]
    G4,
    #[serde(rename = "16g")]
    G16,
    #[serde(rename = "64g")]
    G64,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub strict: Option<bool>,
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
    pub ranker: Option<RankerVersionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hybrid_search: Option<HybridSearchOptions>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RankerVersionType {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "default-2024-11-15")]
    Default20241115,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct HybridSearchOptions {
    pub embedding_weight: f64,
    pub text_weight: f64,
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
    Array(Vec<FilterArrayValue>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterArrayValue {
    String(String),
    Number(f64),
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
pub struct WebSearchPreviewTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<ApproximateLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<SearchContextSize>,
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
pub struct WebSearchApproximateLocation {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub location_type: Option<ApproximateLocationType>,
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
pub enum SearchContextSize {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<WebSearchToolFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchApproximateLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<SearchContextSize>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchToolFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageGenerationTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageGenerationQuality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageGenerationSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageGenerationOutputFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ImageGenerationModeration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageGenerationBackground>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<InputFidelity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_image_mask: Option<ImageGenerationInputImageMask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<u64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationQuality {
    Low,
    Medium,
    High,
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ImageGenerationSize {
    #[serde(rename = "1024x1024")]
    Size1024x1024,
    #[serde(rename = "1024x1536")]
    Size1024x1536,
    #[serde(rename = "1536x1024")]
    Size1536x1024,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationOutputFormat {
    Png,
    Webp,
    Jpeg,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationModeration {
    Auto,
    Low,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationBackground {
    Transparent,
    Opaque,
    Auto,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputFidelity {
    High,
    Low,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageGenerationInputImageMask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LocalShellTool {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ShellTool {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolParamFormat>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomToolParamFormat {
    #[serde(rename = "text")]
    Text(CustomTextFormatParam),
    #[serde(rename = "grammar")]
    Grammar(CustomGrammarFormatParam),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomTextFormatParam {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomGrammarFormatParam {
    pub syntax: GrammarSyntax,
    pub definition: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSyntax {
    Lark,
    Regex,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub server_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<McpConnectorId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<McpAllowedTools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<McpRequireApproval>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum McpConnectorId {
    #[serde(rename = "connector_dropbox")]
    ConnectorDropbox,
    #[serde(rename = "connector_gmail")]
    ConnectorGmail,
    #[serde(rename = "connector_googlecalendar")]
    ConnectorGoogleCalendar,
    #[serde(rename = "connector_googledrive")]
    ConnectorGoogleDrive,
    #[serde(rename = "connector_microsoftteams")]
    ConnectorMicrosoftTeams,
    #[serde(rename = "connector_outlookcalendar")]
    ConnectorOutlookCalendar,
    #[serde(rename = "connector_outlookemail")]
    ConnectorOutlookEmail,
    #[serde(rename = "connector_sharepoint")]
    ConnectorSharepoint,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpAllowedTools {
    ToolNames(Vec<String>),
    Filter(McpToolFilter),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpRequireApproval {
    Filters(McpToolRequireApproval),
    Mode(McpApprovalMode),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpApprovalMode {
    Always,
    Never,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpToolRequireApproval {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always: Option<McpToolFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub never: Option<McpToolFilter>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpToolFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApplyPatchTool {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Mode(ToolChoiceOptions),
    Allowed(ToolChoiceAllowed),
    Function(ToolChoiceFunction),
    Mcp(ToolChoiceMcp),
    Custom(ToolChoiceCustom),
    ApplyPatch(ToolChoiceApplyPatch),
    Shell(ToolChoiceShell),
    FileSearch(ToolChoiceFileSearch),
    WebSearchPreview(ToolChoiceWebSearchPreview),
    ComputerUsePreview(ToolChoiceComputerUsePreview),
    WebSearchPreview20250311(ToolChoiceWebSearchPreview20250311),
    ImageGeneration(ToolChoiceImageGeneration),
    CodeInterpreter(ToolChoiceCodeInterpreter),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceOptions {
    None,
    Auto,
    Required,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceAllowed {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceAllowedType,
    pub mode: ToolChoiceAllowedMode,
    pub tools: Vec<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceAllowedType {
    AllowedTools,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceAllowedMode {
    Auto,
    Required,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceFunctionType,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceFunctionType {
    Function,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceMcp {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceMcpType,
    pub server_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceMcpType {
    Mcp,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceCustom {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceCustomType,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceCustomType {
    Custom,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceApplyPatch {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceApplyPatchType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceApplyPatchType {
    ApplyPatch,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceShell {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceShellType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceShellType {
    Shell,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFileSearch {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceFileSearchType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceFileSearchType {
    FileSearch,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceWebSearchPreview {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceWebSearchPreviewType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceWebSearchPreviewType {
    WebSearchPreview,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceWebSearchPreview20250311 {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceWebSearchPreview20250311Type,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ToolChoiceWebSearchPreview20250311Type {
    #[serde(rename = "web_search_preview_2025_03_11")]
    WebSearchPreview20250311,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceComputerUsePreview {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceComputerUsePreviewType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceComputerUsePreviewType {
    ComputerUsePreview,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceImageGeneration {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceImageGenerationType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceImageGenerationType {
    ImageGeneration,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ToolChoiceCodeInterpreter {
    #[serde(rename = "type")]
    pub choice_type: ToolChoiceCodeInterpreterType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoiceCodeInterpreterType {
    CodeInterpreter,
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
    CompactionItem(CompactionItem),
    ImageGenerationToolCall(ImageGenerationToolCall),
    CodeInterpreterToolCall(CodeInterpreterToolCall),
    LocalShellToolCall(LocalShellToolCall),
    FunctionShellCall(FunctionShellCall),
    FunctionShellCallOutput(FunctionShellCallOutput),
    ApplyPatchToolCall(ApplyPatchToolCall),
    ApplyPatchToolCallOutput(ApplyPatchToolCallOutput),
    McpToolCall(McpToolCall),
    McpListTools(McpListTools),
    McpApprovalRequest(McpApprovalRequest),
    CustomToolCall(CustomToolCall),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: OutputMessageType,
    pub role: OutputMessageRole,
    pub content: Vec<OutputMessageContent>,
    pub status: ItemStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMessageType {
    OutputMessage,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMessageRole {
    Assistant,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputMessageContent {
    OutputText(OutputTextContent),
    Refusal(RefusalContent),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputContent {
    OutputText(OutputTextContent),
    Refusal(RefusalContent),
    ReasoningText(ReasoningTextContent),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputTextContent {
    #[serde(rename = "type")]
    pub content_type: OutputTextContentType,
    pub text: String,
    pub annotations: Vec<Annotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProb>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputTextContentType {
    OutputText,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RefusalContent {
    #[serde(rename = "type")]
    pub refusal_type: RefusalContentType,
    pub refusal: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefusalContentType {
    Refusal,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasoningTextContent {
    #[serde(rename = "type")]
    pub content_type: ReasoningTextContentType,
    pub text: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningTextContentType {
    ReasoningText,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Summary {
    #[serde(rename = "type")]
    pub summary_type: SummaryType,
    pub text: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryType {
    SummaryText,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasoningSummaryPart {
    #[serde(rename = "type")]
    pub part_type: SummaryType,
    pub text: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    #[serde(rename = "file_citation")]
    FileCitation {
        file_id: String,
        index: u64,
        filename: String,
    },
    #[serde(rename = "url_citation")]
    UrlCitation {
        url: String,
        start_index: u64,
        end_index: u64,
        title: String,
    },
    #[serde(rename = "container_file_citation")]
    ContainerFileCitation {
        container_id: String,
        file_id: String,
        start_index: u64,
        end_index: u64,
        filename: String,
    },
    #[serde(rename = "file_path")]
    FilePath { file_id: String, index: u64 },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<i64>,
    pub top_logprobs: Vec<TopLogProb>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
pub struct WebSearchToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: WebSearchToolCallType,
    pub status: WebSearchToolCallStatus,
    pub action: WebSearchAction,
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
#[serde(tag = "type")]
pub enum WebSearchAction {
    #[serde(rename = "search")]
    Search {
        query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        queries: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sources: Option<Vec<WebSearchActionSearchSource>>,
    },
    #[serde(rename = "open_page")]
    OpenPage { url: String },
    #[serde(rename = "find_in_page")]
    Find { url: String, pattern: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WebSearchActionSearchSource {
    #[serde(rename = "type")]
    pub source_type: WebSearchActionSearchSourceType,
    pub url: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebSearchActionSearchSourceType {
    Url,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputerToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: ComputerToolCallType,
    pub call_id: String,
    pub action: ComputerAction,
    pub pending_safety_checks: Vec<ComputerCallSafetyCheckParam>,
    pub status: ItemStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerToolCallType {
    ComputerCall,
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
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    InProgress,
    Completed,
    Incomplete,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasoningItem {
    #[serde(rename = "type")]
    pub item_type: ReasoningItemType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    pub summary: Vec<Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ReasoningTextContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningItemType {
    Reasoning,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompactionItem {
    #[serde(rename = "type")]
    pub item_type: CompactionItemType,
    pub id: String,
    pub encrypted_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompactionItemType {
    Compaction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ImageGenerationToolCall {
    #[serde(rename = "type")]
    pub tool_type: ImageGenerationToolCallType,
    pub id: String,
    pub status: ImageGenerationCallStatus,
    pub result: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageGenerationToolCallType {
    ImageGenerationCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageGenerationCallStatus {
    InProgress,
    Completed,
    Generating,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCall {
    #[serde(rename = "type")]
    pub tool_type: CodeInterpreterToolCallType,
    pub id: String,
    pub status: CodeInterpreterCallStatus,
    pub container_id: String,
    pub code: Option<String>,
    pub outputs: Option<Vec<CodeInterpreterToolOutput>>,
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
    Completed,
    Incomplete,
    Interpreting,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CodeInterpreterToolOutput {
    #[serde(rename = "logs")]
    Logs { logs: String },
    #[serde(rename = "image")]
    Image { url: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LocalShellToolCall {
    #[serde(rename = "type")]
    pub tool_type: LocalShellToolCallType,
    pub id: String,
    pub call_id: String,
    pub action: LocalShellExecAction,
    pub status: LocalShellCallStatus,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalShellToolCallType {
    LocalShellCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalShellCallStatus {
    InProgress,
    Completed,
    Incomplete,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LocalShellExecAction {
    #[serde(rename = "type")]
    pub action_type: LocalShellExecActionType,
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalShellExecActionType {
    Exec,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionShellCall {
    #[serde(rename = "type")]
    pub tool_type: FunctionShellCallType,
    pub id: String,
    pub call_id: String,
    pub action: FunctionShellAction,
    pub status: LocalShellCallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionShellCallType {
    ShellCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionShellAction {
    pub commands: Vec<String>,
    pub timeout_ms: Option<i64>,
    pub max_output_length: Option<i64>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutput {
    #[serde(rename = "type")]
    pub output_type: FunctionShellCallOutputType,
    pub id: String,
    pub call_id: String,
    pub output: Vec<FunctionShellCallOutputContent>,
    pub max_output_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionShellCallOutputType {
    ShellCallOutput,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputContent {
    pub stdout: String,
    pub stderr: String,
    pub outcome: FunctionShellCallOutputOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FunctionShellCallOutputOutcome {
    #[serde(rename = "exit")]
    Exit { exit_code: i64 },
    #[serde(rename = "timeout")]
    Timeout,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApplyPatchToolCall {
    #[serde(rename = "type")]
    pub tool_type: ApplyPatchToolCallType,
    pub id: String,
    pub call_id: String,
    pub status: ApplyPatchCallStatus,
    pub operation: ApplyPatchFileOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyPatchToolCallType {
    ApplyPatchCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyPatchCallStatus {
    InProgress,
    Completed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ApplyPatchFileOperation {
    #[serde(rename = "create_file")]
    CreateFile { path: String, diff: String },
    #[serde(rename = "delete_file")]
    DeleteFile { path: String },
    #[serde(rename = "update_file")]
    UpdateFile { path: String, diff: String },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApplyPatchToolCallOutput {
    #[serde(rename = "type")]
    pub output_type: ApplyPatchToolCallOutputType,
    pub id: String,
    pub call_id: String,
    pub status: ApplyPatchCallOutputStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyPatchToolCallOutputType {
    ApplyPatchCallOutput,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApplyPatchCallOutputStatus {
    Completed,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    #[serde(rename = "type")]
    pub tool_type: McpToolCallType,
    pub id: String,
    pub server_label: String,
    pub name: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub status: McpToolCallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_request_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpToolCallType {
    McpCall,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpToolCallStatus {
    InProgress,
    Completed,
    Incomplete,
    Calling,
    Failed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpListTools {
    #[serde(rename = "type")]
    pub item_type: McpListToolsType,
    pub id: String,
    pub server_label: String,
    pub tools: Vec<McpListToolsTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpListToolsType {
    McpListTools,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpListToolsTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Map<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct McpApprovalRequest {
    #[serde(rename = "type")]
    pub item_type: McpApprovalRequestType,
    pub id: String,
    pub server_label: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpApprovalRequestType {
    McpApprovalRequest,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CustomToolCall {
    #[serde(rename = "type")]
    pub item_type: CustomToolCallType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub call_id: String,
    pub name: String,
    pub input: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomToolCallType {
    CustomToolCall,
}
// endregion: --- Tool call output structs

// region: --- Streaming logprob structs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseLogProb {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<ResponseLogProbTopLogprobs>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseLogProbTopLogprobs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
}
// endregion: --- Streaming logprob structs
