---
title: Production Deployment
type: docs
prev: docs/self-hosting/deployment/local
next: docs/self-hosting/configuration
sidebar:
  open: true
---

For production deployments, you'll need to build a Docker image and deploy it alongside your database and cache services.

## Creating a Dockerfile

Create a `Dockerfile` in the project root:

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy dependency files
COPY llmur/Cargo.toml ./llmur/

# Copy source code
COPY llmur ./llmur

# Build the application
RUN cargo build --release --manifest-path ./llmur/Cargo.toml

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/llmur /usr/local/bin/llmur

# Copy configuration file (or mount as volume)
COPY config.yaml /app/config.yaml

EXPOSE 8082

CMD ["llmur", "--configuration", "/app/config.yaml"]
```

## Building the Docker Image

```bash
docker build -t llmur:latest .
```

## Running the Container

```bash
docker run -d \
  --name llmur \
  -p 8082:8082 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  --network app-network \
  llmur:latest
```

## Production Configuration

For production, ensure your `config.yaml` points to your production database and cache:

```yaml
application_secret: your-production-secret
log_level: info
host: 0.0.0.0
port: 8082
database_configuration:
  engine: postgres
  host: your-postgres-host
  port: 5432
  database: llmur
  username: your-db-user
  password: your-db-password
  min_connections: 5
  max_connections: 20
cache_configuration:
  engine: redis
  host: your-redis-host
  port: 6379
  username: default
  password: your-redis-password
```

See the [Configuration](../configuration/database-backed/) section for detailed configuration options.

