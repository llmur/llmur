---
title: Self-Hosting
type: docs
prev: docs
next: docs/self-hosting/deployment
sidebar:
  open: true
---

LLMUR can be self-hosted in two ways: locally for development using Docker Compose, or in production using a Dockerfile. Currently, only database-backed deployment is available.

## Deployment Options

- **[Local Deployment](./deployment/local/)** - Quick setup using Docker Compose for development and testing
- **[Production Deployment](./deployment/production/)** - Production-ready deployment using Dockerfile

## Configuration

- **[Database-Backed Configuration](./configuration/database-backed/)** - Required configuration for database-backed deployment
- **[Standalone Configuration](./configuration/standalone/)** - Not available yet

## Prerequisites

Before deploying LLMUR, ensure you have:

- Docker and Docker Compose installed (for local deployment)
- PostgreSQL 16.4 or later
- Redis 7.4 or later
- A YAML configuration file (see [Configuration](./configuration/database-backed/))
