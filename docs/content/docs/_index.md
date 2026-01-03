---
title: Documentation
next: docs/self-hosting
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

## Getting Started

### Quick Start

1. **[Self-Hosting Guide](./self-hosting/)** - Learn how to deploy LLMUR locally or in production
2. **[Configuration](./self-hosting/configuration/)** - Understand how to configure the service
3. **[Deployment](./self-hosting/deployment/)** - Choose between local or production deployment

### Documentation Structure

This documentation is organized into the following sections:

- **Self-Hosting** - Complete guide to deploying and configuring LLMUR
    - [Local Deployment](./self-hosting/deployment/local/) - Quick setup with Docker Compose
    - [Production Deployment](./self-hosting/deployment/production/) - Production-ready deployment
    - [Configuration](./self-hosting/configuration/) - Configuration reference and examples

## Architecture

LLMUR is built with:

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL for persistent data storage
- **Cache**: Redis for caching and session management
- **Observability**: OpenTelemetry for distributed tracing and metrics

## API Endpoints

### OpenAI-Compatible Endpoints
- `POST /v1/chat/completions` - Chat completions endpoint compatible with OpenAI's API

### Admin Endpoints
- `/admin/user` - User management
- `/admin/project` - Project management
- `/admin/connection` - Provider connection management
- `/admin/deployment` - Deployment configuration
- `/admin/virtual-key` - API key management
- `/admin/graph/{key}/{deployment}` - Usage graph visualization

### System Endpoints
- `GET /health` - Health check endpoint

## Next Steps

Ready to get started? Head over to the [Self-Hosting Guide](./self-hosting/) to learn how to deploy LLMUR.
