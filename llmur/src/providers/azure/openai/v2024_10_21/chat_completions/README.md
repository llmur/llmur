# Azure OpenAI chat_completions structs

## Overview
This module models Azure OpenAI Chat Completions request/response payloads per `v2024_10_21/openapi.yaml`, and provides transforms to OpenAI chat_completions types.

## Request structs
- `Request`: top-level request with `messages` and common options (`temperature`, `top_p`, `max_tokens`, `stream`, etc.).
- `Message`: tagged enum for `system`, `user`, `assistant`, `tool`, and deprecated `function` roles.
- Content parts:
  - `UserMessageContentPart` supports `text` and `image_url` parts.
  - `ImageUrlContentPart` models image URL plus optional `detail`.
  - `AssistantMessageContentPart` supports `text` and `refusal`.
- Tooling:
  - `Tool`, `ToolFunction` for function tools.
  - `ToolChoice`, `ToolChoiceMode`, `ToolChoiceFunction`.
  - Deprecated `FunctionCall` and `FunctionDefinition` fields.
- Extensions:
  - `AzureChatExtensionConfiguration` for `azure_search` and `azure_cosmos_db` extension configs.
- Streaming:
  - `StreamOptions` with `include_usage`.

## Response structs
- `Response`: top-level response with `choices`, `usage`, and `system_fingerprint`.
- `ResponseChoice`: finish reason, message, content filters, and optional logprobs.
- `ResponseChoiceMessage`: content, refusal, tool calls, deprecated `function_call`, and optional extension `context`.
- Usage:
  - `ResponseUsage`, `CompletionTokensDetails`.
- Tool calls:
  - `ResponseChoiceToolCall`, `ResponseChoiceFunctionToolCall`.
- Content filtering:
  - `ResponseContentFilterResults`, `ContentFilterSeverityResult`, `ContentFilterDetectedResult`, `ContentFilterDetectedWithCitationResult`, `ContentFilterCitation`, `ErrorBase`.
- Logprobs:
  - `ResponseChoiceLogprob`, `ResponseChoiceLogprobContent`, `ResponseChoiceTopLogprobContent`.
- Extension context:
  - `AzureMessageContext`, `AzureContextCitation`.

## Transform to OpenAI format
The `to_openai_transform` module converts Azure responses into OpenAI chat_completions response structs:
- `Response` -> `openai::chat_completions::response::Response`
- `ResponseUsage` maps token counts and, when present, reasoning tokens into OpenAI usage details.
- `ResponseChoiceMessage` maps `tool_calls`, `refusal`, and deprecated `function_call` into OpenAI equivalents.

### Context added
- Azure extension `context` (citations/metadata) is not represented in OpenAI responses, so it is not added.

### Context loss
- `ResponseChoiceMessage.context` is dropped during conversion because OpenAI chat_completions has no equivalent field.
- Azure content filter results (`content_filter_results`, `prompt_filter_results`) are dropped.

The `from_openai_transform` module converts OpenAI request structs into Azure request structs:
- `openai::chat_completions::request::Request` -> `Request`
- Content parts are filtered to Azure-supported types (text/image_url).
- Tool choice modes and deprecated function call fields are mapped when present.

### Context added
- No new fields are synthesized; only OpenAI-provided fields are mapped.

### Context loss
- OpenAI content parts that Azure does not support (e.g., input audio or other non-text/image parts) are dropped.
