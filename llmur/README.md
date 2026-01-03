<div align="center">

<p align="center">
  <img src="/img/banner.png" alt="LLMUR banner" width=1040 />
</p>

# Lightweight LLM Proxy

[docs](https://llmur.github.io/llmur/)

</div>

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

## Local Deployment

1. **Clone the repository** (if you haven't already):
   ```bash
   git clone https://github.com/llmur/llmur.git
   cd llmur
   ```

2. **Create a configuration file** (`config.yaml`):
   ```yaml
   application_secret: your-secret-here
   log_level: info
   host: 0.0.0.0
   port: 8082
   database_configuration:
     engine: postgres
     host: db
     port: 5432
     database: llmur
     username: postgres
     password: postgres
   cache_configuration:
     engine: redis
     host: cache
     port: 6379
     username: default
     password: redispassword
   ```

3. **Start the services**:
   ```bash
   docker-compose up -d db cache
   ```

4. **Build and run the proxy**:
   ```bash
   cargo build --release
   ./target/release/llmur --configuration config.yaml
   ```

## More information

For more information take a look at the [docs](https://llmur.github.io/llmur/)
