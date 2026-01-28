# Setup (uv)

```bash
uv sync
```

# Export environment variables

```bash
set -a
source .env
set +a
```

# Run all tests

```bash
uv run python run_tests.py
```

# Run specific test suites
```bash
uv run python run_tests.py --suite users
uv run python run_tests.py --suite projects
uv run python run_tests.py --suite memberships
```

# Run specific file
```bash
uv run python run_tests.py --file test_users.py
```

# Run tests matching a pattern
```bash
uv run python run_tests.py --pattern "create"     # Only test functions with "create" in name
uv run python run_tests.py --pattern "delete"     # Only delete tests
uv run python run_tests.py --pattern "not_found"  # Only 404 tests
```

# List all available options
```bash
uv run python run_tests.py --list
```

# Combine options
```bash
uv run python run_tests.py --suite users --verbose --html
```
