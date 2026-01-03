---
title: Local Deployment
type: docs
prev: docs/self-hosting/deployment
next: docs/self-hosting/deployment/production
sidebar:
  open: true
---

The easiest way to get started with LLMUR is using the provided Docker Compose setup. This will start all required services including PostgreSQL, Redis, and optional observability tools.

## Quick Start

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

## Docker Compose Services

The `docker-compose.yaml` file includes the following services:

- **db** - PostgreSQL 16.4 database
- **cache** - Redis 7.4 cache
- **otel-collector** - OpenTelemetry Collector (optional)
- **jaeger** - Distributed tracing UI (optional)
- **prometheus** - Metrics collection (optional)
- **loki** - Log aggregation (optional)
- **grafana** - Observability dashboard (optional)

## Configuration for Local Deployment

When running locally with Docker Compose, use these connection settings:

- **Database host**: `db` (Docker service name)
- **Database port**: `5432`
- **Cache host**: `cache` (Docker service name)
- **Cache port**: `6379`
- **Cache password**: `redispassword` (as configured in docker-compose.yaml)

See the [Configuration](../configuration/database-backed/) section for detailed configuration options.

