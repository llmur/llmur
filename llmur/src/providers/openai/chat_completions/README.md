# OpenAI chat_completions structs

## Overview
This module models OpenAI Chat Completions request/response payloads per `openapi.yaml`, including deprecated fields for backward compatibility.

## Request structs
### `Request`
- `model`: model ID.
- `messages`: ordered conversation messages.
- `modalities`: output types (`text`, `audio`).
- `audio`: audio output config (`voice`, `format`).
- `temperature`, `top_p`, `n`, `seed`, `max_completion_tokens`, `max_tokens`.
- `presence_penalty`, `frequency_penalty`.
- `logprobs`, `top_logprobs`.
- `response_format` (text/json_object/json_schema).
- `stream`, `stream_options`.
- `stop` (string or array).
- `tools`, `tool_choice`, `parallel_tool_calls`.
- `reasoning_effort`.
- `store`.
- `user`.
- `web_search_options` (context size and location).
- Deprecated: `function_call`, `functions`.

### `Message`
- `system`, `developer`, `user`, `assistant`, `tool`, deprecated `function`.
- Each role has `content` (string or content parts) and optional `name`.
- Assistant supports `tool_calls`, `refusal`, and deprecated `function_call`.

### Content parts
- `UserMessageContentPart`:
  - `text`
  - `image_url` (`url`, optional `detail`)
  - `input_audio` (`data`, `format`)
  - `file` (`filename`, `file_data`, `file_id`)
- `AssistantMessageContentPart`: `text` or `refusal`.
- `SystemMessageContentPart` / `ToolMessageContentPart`: `text` only.

### Tools
- `Tool`: function tool definition.
- `ToolFunction`: name, description, parameters (JSON schema), strict.
- `ToolChoice`: `none`/`auto`/`required` or named function selection.
- Deprecated: `FunctionCall`, `FunctionDefinition`.

### Output formats
- `ResponseFormat`: text/json_object/json_schema.
- `ResponseJsonSchema`: name, description, schema, strict.

### Web search
- `WebSearchOptions`: `search_context_size`, `user_location`.
- `UserLocation`: `{ type: "approximate", approximate: { city, region, country, timezone } }`.

## Response structs
### `Response`
- `id`, `object`, `created`, `model`.
- `choices`.
- `usage`.
- `service_tier`, `system_fingerprint`.

### `ResponseChoice`
- `finish_reason`, `index`, `message`, `logprobs`.

### `ResponseChoiceMessage`
- `content`, `refusal`, `role`.
- `tool_calls`.
- `annotations` (URL citations for web search).
- `audio` (when audio output requested).
- Deprecated: `function_call`.

### Usage
- `ResponseUsage`: `prompt_tokens`, `completion_tokens`, `total_tokens`.
- Optional `completion_tokens_details` and `prompt_tokens_details`.

### Logprobs
- `ResponseChoiceLogprob` with `content` and `refusal`.
- `ResponseChoiceLogprobContent` / `ResponseChoiceTopLogprobContent`.

## Examples
### Minimal request
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

### Tool call (function)
```json
{
  "model": "gpt-4o",
  "messages": [{"role": "user", "content": "What is the weather?"}],
  "tools": [{
    "type": "function",
    "function": {
      "name": "get_weather",
      "parameters": {"type": "object", "properties": {"city": {"type": "string"}}}
    }
  }],
  "tool_choice": "auto"
}
```

## Notes
- Enum-like fields are serialized with lowercase strings.
- Deprecated fields are retained for backward compatibility.
