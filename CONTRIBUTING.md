# Contributing to OpenChat

Thank you for your interest in contributing to OpenChat! This document provides guidelines and instructions for contributing to this open-source project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Commit Guidelines](#commit-guidelines)
- [Reporting Issues](#reporting-issues)

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. We expect all contributors to:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Accept responsibility for mistakes and learn from them

---

## Getting Started

### Prerequisites

Before contributing, ensure you have the following installed:

- **Rust 1.83+** (2024 edition)
- **Node.js 20+**
- **PostgreSQL 14+**
- **Redis 7+**
- **Docker and Docker Compose**
- **sqlx-cli** (`cargo install sqlx-cli`)

### Repository Structure

```
openchat/
├── api/          # Rust backend (Actix Web)
├── ui/           # Next.js frontend
├── windows/      # Tauri desktop application
└── docs/         # Documentation
```

---

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone git@github.com:YOUR_USERNAME/OpenChat.git
cd OpenChat
```

### 2. Set Up the Backend (API)

```bash
cd api
cp .env.example .env
# Edit .env with your database credentials

# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Run database migrations
sqlx database create
sqlx migrate run

# Run the API server
cargo run
```

### 3. Set Up the Frontend (UI)

```bash
cd ui
npm install
npm run dev
```

### 4. Verify Setup

- API: http://localhost:8080
- UI: http://localhost:3000

---

## Pull Request Process

### Requirements

**All pull requests require at least 1 approval before merging into `main`.**

### Creating a Pull Request

1. **Create a feature branch** from `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our [coding standards](#coding-standards)

3. **Test your changes** thoroughly:
   ```bash
   # Backend
   cd api
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check

   # Frontend
   cd ui
   npm run lint
   npm run build
   ```

4. **Commit your changes** following our [commit guidelines](#commit-guidelines)

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request** against `main` on the upstream repository

### PR Checklist

Before submitting your PR, ensure:

- [ ] Code compiles without errors or warnings
- [ ] All tests pass
- [ ] Linters pass (clippy, eslint)
- [ ] Documentation is updated if needed
- [ ] README.md is updated if you modified the project structure
- [ ] Version number is incremented in `Cargo.toml` (for API changes)
- [ ] No secrets or credentials are committed
- [ ] Database migrations use sqlx (new migration files for schema changes)

### PR Review Process

1. Submit your PR with a clear description of changes
2. Wait for at least 1 approval from a maintainer
3. Address any feedback or requested changes
4. Once approved, a maintainer will merge your PR

---

## Coding Standards

### Rust (API)

- Follow Rust 2024 edition idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes
- Use async/await for I/O operations to maintain performance
- Leverage Redis caching to reduce database load
- Create database indexes for frequently queried columns
- Invalidate cache on updates/deletes

**Database Guidelines:**
- Optimize queries to minimize database round-trips
- Check cache before hitting the database
- Use sqlx for migrations (never run migrations manually)
- Create indexes for performance-critical queries

### TypeScript/Next.js (UI)

- Use strict TypeScript
- Follow Next.js conventions and best practices
- Run `npm run lint` before committing
- Fix all ESLint errors and warnings properly (do not disable rules)
- Remove unused code instead of commenting it out
- Use TanStack Query for data fetching

**UI Guidelines:**
- Ensure forms include all fields with auto-save functionality
- Maintain consistent, clean UI design
- Consider UX implications of changes

### General Guidelines

- Write self-documenting code with clear variable/function names
- Avoid over-engineering - keep solutions simple and focused
- Do not add features beyond what was requested
- Performance matters - this app is designed for millions of users

---

## Testing

### Backend Tests

```bash
cd api
cargo test
```

Write tests for:
- Business logic
- API endpoints (integration tests)
- Database operations

### Frontend Tests

```bash
cd ui
npm run test        # If test suite is configured
npm run lint        # Linting
npm run build       # Build check
```

---

## Commit Guidelines

### Commit Message Format

Use clear, descriptive commit messages:

```
<type>: <short description>

[optional body with more details]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat: add message search with advanced filters
fix: resolve WebSocket reconnection issue
docs: update API documentation for notifications
refactor: optimize channel membership queries
```

### Version Numbering

When making changes to the API or UI, increment the version in `Cargo.toml` or `package.json`:

- **Major version**: Breaking changes or big rewrites
- **Minor version**: New features (backward-compatible)
- **Patch version**: Bug fixes and small tweaks

---

## Reporting Issues

### Bug Reports

When reporting bugs, include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Detailed steps to reproduce the bug
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**: OS, browser, versions
6. **Screenshots/Logs**: If applicable

### Feature Requests

For feature requests, include:

1. **Problem Statement**: What problem does this solve?
2. **Proposed Solution**: How should it work?
3. **Alternatives**: Any alternatives you've considered
4. **Additional Context**: Mockups, examples, etc.

---

## Questions?

If you have questions about contributing:

1. Check existing [GitHub Issues](https://github.com/ZerosAndOnesLLC/OpenChat/issues)
2. Open a new issue with your question
3. Join the discussion in [GitHub Discussions](https://github.com/ZerosAndOnesLLC/OpenChat/discussions)

---

## Contact

For more information or questions not covered here:

- **Email**: mackman42@outlook.com
- **GitHub**: [@mack42](https://github.com/mack42)

---

Thank you for contributing to OpenChat!
