---
title: About
type: about
---


**LLMUR** (Lightweight LLM Proxy) is a self-hostable proxy service that provides a unified interface for interacting with multiple Large Language Model (LLM) providers. It offers OpenAI-compatible API endpoints while adding powerful features like rate limiting, load balancing, and multi-tenant management.

## What is LLMUR?

LLMUR acts as a middleware layer between your applications and LLM providers, giving you:

- **Unified API Interface** - Use OpenAI-compatible endpoints (`/v1/chat/completions`) regardless of the underlying provider
- **Multi-Provider Support** - Connect to multiple providers including OpenAI and Azure OpenAI
- **Rate Limiting & Usage Management** - Control usage with configurable rate limits and usage windows
- **Load Balancing** - Distribute requests across multiple connections and deployments
- **Multi-Tenant Architecture** - Manage users, projects, and API keys with fine-grained access control
- **Observability** - Built-in tracing, metrics, and logging with OpenTelemetry support
- **Self-Hostable** - Deploy on your own infrastructure with full control

## Key Features

### Provider Management
- Support for multiple LLM providers (OpenAI, Azure OpenAI)
- Connection pooling and management
- Deployment-based routing

### Access Control
- User and project management
- Virtual API keys for secure access
- Session token authentication
- Project-based access control

### Operational Features
- Health check endpoints
- Automatic database migrations
- Redis caching for performance
- Request logging and monitoring

