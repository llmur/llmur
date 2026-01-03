---
title: Standalone Configuration
type: docs
prev: docs/self-hosting/configuration/database-backed
---

> **⚠️ Not Available Yet**
> 
> Standalone deployment is currently not available. Please use [database-backed deployment](./database-backed/) instead.

## Planned Features

Standalone deployment will allow you to deploy LLMUR as a single service using static configuration files or environment variables, with no external dependencies.

**This deployment will be ideal when:**
- You want a minimal setup
- API keys and providers are defined ahead of time
- No runtime changes or persistence are required
- You are running locally, at the edge, or in a lightweight environment

## Current Status

Standalone deployment is under development. For now, all deployments require:
- PostgreSQL database
- Redis cache

See [Database-Backed Configuration](./database-backed/) for the currently available deployment method.
