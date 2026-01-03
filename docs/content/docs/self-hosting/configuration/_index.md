---
title: Configuration
type: docs
prev: docs/self-hosting/deployment/production
next: docs/self-hosting/configuration/database-backed
sidebar:
  open: true
---

LLMUR requires a YAML configuration file that specifies database and cache connections, along with optional settings for logging, observability, and security.

## Configuration Types

- **[Database-Backed](./database-backed/)** - Required configuration for database-backed deployment (currently the only available option)
- **[Standalone](./standalone/)** - Not available yet

## Quick Reference

The configuration file is passed to the application via the `--configuration` command-line argument:

```bash
llmur --configuration /path/to/config.yaml
```

All deployments currently require:
- PostgreSQL database configuration
- Redis cache configuration
- Application secret

See the [Database-Backed Configuration](./database-backed/) page for complete details and examples.
