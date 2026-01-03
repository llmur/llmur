---
title: Database-Backed Configuration
type: docs
prev: docs/self-hosting/deployment/production
next: docs/self-hosting/configuration/standalone
---

Database-backed deployment is the **only currently available** deployment method for LLMUR. It requires PostgreSQL for data persistence and Redis for caching.

## Configuration File Structure

LLMUR uses a YAML configuration file that must be provided via the `--configuration` command-line argument.

### Required Fields

```yaml
application_secret: string          # Secret used for application-level operations
database_configuration: object      # PostgreSQL database configuration
cache_configuration: object         # Redis cache configuration
```

### Optional Fields

```yaml
log_level: string                   # Logging level (default: "info")
host: string                        # Bind host (default: "0.0.0.0")
port: number                        # Bind port (default: 8082)
master_keys: array<string>          # Master API keys for privileged operations
otel: object                        # OpenTelemetry configuration
```

## Complete Configuration Example

```yaml
application_secret: 7d8fe9e9-04fc-42a8-9577-40ecc40d27d7
log_level: info
host: 0.0.0.0
port: 8082

# Master keys for privileged operations
master_keys:
  - my-master-key-1
  - my-master-key-2

# OpenTelemetry configuration (optional)
otel:
  exporter_otlp_endpoint: http://localhost:4317

# Database configuration (required)
database_configuration:
  engine: postgres
  host: localhost
  port: 5432
  database: llmur
  username: postgres
  password: postgres
  min_connections: 5      # Optional: minimum connection pool size
  max_connections: 20     # Optional: maximum connection pool size

# Cache configuration (required)
cache_configuration:
  engine: redis
  host: localhost
  port: 6379
  username: default
  password: redispassword
```

## Configuration Reference

### `application_secret`

A secret string used for application-level cryptographic operations. This should be a unique, randomly generated value.

**Example**: `7d8fe9e9-04fc-42a8-9577-40ecc40d27d7`

### `log_level`

Controls the verbosity of logging. Valid values: `trace`, `debug`, `info`, `warn`, `error`.

**Default**: `info`

### `host` and `port`

Network binding configuration for the HTTP server.

- **host**: IP address or hostname to bind to (default: `0.0.0.0`)
- **port**: TCP port number (default: `8082`)

### `master_keys`

An array of master API keys that grant privileged access to the system. These keys can be used for administrative operations.

**Example**:
```yaml
master_keys:
  - admin-key-1
  - admin-key-2
```

### `otel` (OpenTelemetry)

Optional OpenTelemetry configuration for distributed tracing and metrics.

```yaml
otel:
  exporter_otlp_endpoint: http://localhost:4317
```

- **exporter_otlp_endpoint**: OTLP endpoint URL for telemetry data export

### `database_configuration`

PostgreSQL database configuration. Currently, only PostgreSQL is supported.

```yaml
database_configuration:
  engine: postgres
  host: string
  port: number
  database: string
  username: string
  password: string
  min_connections: number    # Optional
  max_connections: number    # Optional
```

**Fields**:
- **engine**: Must be `postgres`
- **host**: Database server hostname or IP address
- **port**: Database server port (typically `5432`)
- **database**: Database name
- **username**: Database user
- **password**: Database password
- **min_connections**: Minimum connection pool size (optional)
- **max_connections**: Maximum connection pool size (optional)

**Example for Docker Compose**:
```yaml
database_configuration:
  engine: postgres
  host: db              # Use Docker service name
  port: 5432
  database: llmur
  username: postgres
  password: postgres
```

**Example for Production**:
```yaml
database_configuration:
  engine: postgres
  host: db.example.com
  port: 5432
  database: llmur_prod
  username: llmur_user
  password: secure-password-here
  min_connections: 10
  max_connections: 50
```

### `cache_configuration`

Redis cache configuration. Currently, only Redis is supported.

```yaml
cache_configuration:
  engine: redis
  host: string
  port: number
  username: string
  password: string
```

**Fields**:
- **engine**: Must be `redis`
- **host**: Redis server hostname or IP address
- **port**: Redis server port (typically `6379`)
- **username**: Redis username (often `default`)
- **password**: Redis password

**Example for Docker Compose**:
```yaml
cache_configuration:
  engine: redis
  host: cache           # Use Docker service name
  port: 6379
  username: default
  password: redispassword
```

**Example for Production**:
```yaml
cache_configuration:
  engine: redis
  host: redis.example.com
  port: 6379
  username: default
  password: secure-redis-password
```

## Environment Variables

The `log_level` can also be set via the `LLMUR_LOG_LEVEL` environment variable, which takes precedence over the configuration file value.

## Database Migrations

When LLMUR starts with a database-backed configuration, it automatically runs database migrations to set up the required schema. Ensure your database user has sufficient permissions to create tables and indexes.

## Troubleshooting

### Configuration File Not Found

Ensure the path to your configuration file is correct:
```bash
llmur-proxy --configuration /path/to/config.yaml
```

### Database Connection Errors

- Verify database host, port, and credentials
- Ensure the database service is running
- Check network connectivity (use service names in Docker Compose)
- Verify database user has required permissions

### Cache Connection Errors

- Verify Redis host, port, and password
- Ensure Redis service is running
- Check network connectivity
- Verify Redis authentication credentials
