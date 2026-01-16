# Gemini generateContent structs

## Overview
This module models the Gemini `generateContent` request/response payloads using serde-friendly Rust structs that track the API schema.

## Request structs
### `Request`
- `contents`: conversation history and current user input.
- `tools`: optional tool definitions (function declarations, code execution).
- `tool_config`: tool calling behavior (modes, allowlists).
- `safety_settings`: per-category safety thresholds.
- `system_instruction`: developer system content (text-only currently).
- `generation_config`: decoding, schema, and modality settings.
- `cached_content`: cached content reference (`cachedContents/...`).

### `Content`
- `role`: optional role (`user`, `model`, etc.).
- `parts`: ordered content parts.

### `Part`
- `text`: plain text content.
- `inline_data`: base64 media data.
- `file_data`: URI reference to uploaded media.
- `function_call`: function name + JSON args.
- `function_response`: function response payload.
- `executable_code` / `code_execution_result`: code execution artifacts (JSON).

### Tooling
- `Tool`: function declarations or code execution toggle.
- `FunctionDeclaration`: name, description, parameters (JSON schema).
- `ToolConfig` / `FunctionCallingConfig`: mode and allowlist settings.

### Safety
- `SafetySetting`: `category` + `threshold`.
- `HarmCategory` / `HarmBlockThreshold`: enum values as defined in the API.

### Generation config
- `stop_sequences`, `response_mime_type`, `response_schema`, `response_json_schema`.
- `response_modalities`, `candidate_count`, `max_output_tokens`.
- `temperature`, `top_p`, `top_k`, `seed`.
- `presence_penalty`, `frequency_penalty`.
- `response_logprobs`, `logprobs`.
- `speech_config`, `thinking_config`, `image_config`, `media_resolution`.

## Response structs
### `Response`
- `candidates`: generated outputs (may be absent when blocked).
- `prompt_feedback`: prompt safety feedback.
- `usage_metadata`: token usage.
- `model_version`, `response_id`, `model_status`.

### `Candidate`
- `content`: generated content.
- `finish_reason`, `finish_message`.
- `safety_ratings`.
- `citation_metadata`.
- `token_count`.
- `grounding_attributions`, `grounding_metadata`.
- `avg_logprobs`, `logprobs_result`.
- `url_context_metadata`.
- `index`.

### Grounding and citations
- `CitationMetadata`, `CitationSource`.
- `GroundingAttribution`, `AttributionSourceId`, `GroundingPassageId`, `SemanticRetrieverChunk`.
- `GroundingMetadata`, `GroundingChunk`, `GroundingSupport`, `SearchEntryPoint`, `RetrievalMetadata`, `Segment`.
- `Web`, `RetrievedContext`, `Maps`, `PlaceAnswerSources`, `ReviewSnippet`.

### Logprobs
- `LogprobsResult`, `TopCandidates`, `LogprobsCandidate`.

### URL context
- `UrlContextMetadata`, `UrlMetadata`.

## Examples
### Minimal request
```json
{
  "contents": [{
    "parts": [{"text": "Write a story about a magic backpack."}]
  }]
}
```

### Inline image
```json
{
  "contents": [{
    "parts": [
      {"text": "Tell me about this instrument."},
      {"inline_data": {"mime_type": "image/jpeg", "data": "BASE64"}}
    ]
  }]
}
```

## Notes
- Enum-like fields are represented as Rust enums with serde renames.
- Free-form JSON payloads (schemas, tool args/results) use `serde_json::Value`.

## Transforms
### Gemini response -> OpenAI chat_completions
The `to_openai_transform` module converts Gemini `Response` into OpenAI chat_completions response structs. The conversion is opinionated and lossy.

Assumptions:
- `content.parts` order is preserved; all text parts are concatenated into a single OpenAI `message.content`.
- `finishReason` values are mapped to the closest OpenAI `finish_reason` semantics.

Unmatched cases:
- Parts without `text` or `function_call` are ignored (no OpenAI equivalent in chat_completions).
- If `candidates` or `usageMetadata` are missing, OpenAI usage counts default to zero.

Context added:
- Creates OpenAI-required envelope fields (`id`, `object`, `created`) when Gemini does not provide equivalents.
- Synthesizes tool call IDs using the candidate/part index.

Context loss:
- Safety ratings, grounding metadata, citations, and prompt feedback are dropped.
- Non-text parts without `function_call` are omitted from the OpenAI message.

### OpenAI chat_completions request -> Gemini request
The `from_openai_transform` module converts OpenAI request structs into Gemini `generateContent` request structs. The conversion is opinionated and lossy.

Assumptions:
- `system` and `developer` messages are merged into a single Gemini `system_instruction`.
- Assistant `function_call` and `tool_calls` become Gemini `function_call` parts.
- Tool responses are sent back as `function_response` parts using the OpenAI `tool_call_id` as the function name.

Unmatched cases:
- OpenAI content parts without Gemini equivalents (input audio, files) are dropped.
- `image_url` inputs without a data URL or a known file extension are ignored.

Context added:
- Maps `system`/`developer` roles into `system_instruction` content.
- Tool choice mode becomes Gemini `functionCallingConfig.mode`.

Context loss:
- OpenAI-only fields (e.g., `metadata`, `logit_bias`, `service_tier`, `web_search_options`) are not represented.
