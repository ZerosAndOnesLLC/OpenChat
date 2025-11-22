# Contributing to OpenChat

Thank you for your interest in contributing to OpenChat! We're building the best open-source, self-hosted team collaboration platform, and we're excited to have you join us.

Whether you're fixing a bug, adding a feature, improving documentation, or helping with design, every contribution makes OpenChat better for everyone.

## Why Contribute?

- **Make an Impact**: OpenChat is used by real teams for their daily communication
- **Learn Modern Tech**: Work with Rust 2024, Next.js 16, PostgreSQL, and more
- **Build Your Portfolio**: Contribute to a production-ready open-source project
- **Join a Community**: Connect with developers who care about privacy and self-hosting
- **Shape the Future**: Help define the roadmap and architecture of OpenChat

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Guidelines](#coding-guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

---

## Code of Conduct

We are committed to providing a welcoming, inclusive, and harassment-free experience for everyone. By participating in this project, you agree to abide by our Code of Conduct.

**In short:**
- Be respectful and kind
- Welcome newcomers and help them learn
- Focus on what's best for the community
- Show empathy towards others

---

## Getting Started

### What Can I Contribute?

We welcome all kinds of contributions:

- **🐛 Bug Fixes**: Found a bug? Fix it and submit a PR!
- **✨ New Features**: Check our [roadmap](README.md#roadmap) or propose your own ideas
- **📚 Documentation**: Improve README, add code comments, write guides
- **🎨 UI/UX**: Enhance the interface, improve accessibility, design new features
- **🧪 Testing**: Write tests, improve coverage, test edge cases
- **🚀 Performance**: Optimize queries, reduce bundle size, improve latency
- **🔒 Security**: Find vulnerabilities, improve authentication, harden infrastructure

### First-Time Contributors

New to open source? No problem! Here's how to get started:

1. **Browse Issues**: Look for issues labeled `good first issue` or `help wanted`
2. **Ask Questions**: Don't hesitate to ask in the issue comments or discussions
3. **Start Small**: Begin with documentation, typo fixes, or small bug fixes
4. **Learn as You Go**: We'll help you through the PR process

### Finding an Issue

- **Bug Reports**: Check [Issues](https://github.com/yourusername/openchat/issues) for bugs that need fixing
- **Feature Requests**: See what features the community is asking for
- **Roadmap**: Review our [roadmap](README.md#roadmap) for planned features
- **Your Own Ideas**: Have an idea? Open an issue to discuss it first!

---

## Development Setup

### Prerequisites

Make sure you have the following installed:

- **Docker** and **Docker Compose** (for PostgreSQL and Redis)
- **Rust** 1.83+ (2024 edition) - [Install Rust](https://rustup.rs/)
- **Node.js** 20+ and **npm** - [Install Node.js](https://nodejs.org/)
- **Git** - For version control

### Initial Setup

1. **Fork the Repository**

   Click the "Fork" button in the top-right corner of the [OpenChat repository](https://github.com/ZerosAndOnesLLC/OpenChat).

2. **Clone Your Fork**

   ```bash
   # HTTPS
   git clone https://github.com/YOUR_USERNAME/OpenChat.git
   cd OpenChat

   # Or SSH (recommended)
   git clone git@github.com:YOUR_USERNAME/OpenChat.git
   cd OpenChat
   ```

3. **Add Upstream Remote**

   ```bash
   git remote add upstream https://github.com/ZerosAndOnesLLC/OpenChat.git
   ```

### Backend Setup (Rust API)

1. **Start Database Services**

   ```bash
   docker-compose up -d postgres redis
   ```

2. **Configure Environment**

   ```bash
   cd api
   cp .env.example .env
   ```

   Edit `.env` with your database credentials and configuration.

3. **Install SQLx CLI**

   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

4. **Run Database Migrations**

   ```bash
   sqlx database create
   sqlx migrate run
   ```

5. **Start the API Server**

   ```bash
   cargo run
   ```

   The API will be available at `http://localhost:8080`

### Frontend Setup (Next.js UI)

1. **Install Dependencies**

   ```bash
   cd ui
   npm install
   ```

2. **Start the Development Server**

   ```bash
   npm run dev
   ```

   The UI will be available at `http://localhost:3000`

### Desktop App Setup (Optional)

If you're working on the desktop application:

```bash
cd windows
npm install
npm run tauri dev
```

See [windows/README.md](windows/README.md) for more details.

---

## Development Workflow

### Creating a Feature Branch

Always create a new branch for your changes:

```bash
# Make sure you're on main and up to date
git checkout main
git pull upstream main

# Create a new feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

### Making Changes

1. **Write Code**: Make your changes in your feature branch
2. **Follow Guidelines**: See [Coding Guidelines](#coding-guidelines) below
3. **Test Locally**: Ensure everything works as expected
4. **Commit Often**: Make small, logical commits with clear messages

### Commit Message Guidelines

Write clear, descriptive commit messages:

```bash
# Good commit messages
git commit -m "feat: add message search with PostgreSQL full-text search"
git commit -m "fix: resolve WebSocket reconnection issue on network change"
git commit -m "docs: update API documentation for audit logs endpoint"
git commit -m "refactor: simplify channel membership validation logic"

# Commit message format
<type>: <description>

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes (formatting, no logic change)
- refactor: Code refactoring
- test: Adding or updating tests
- chore: Maintenance tasks
```

### Keeping Your Branch Updated

Regularly sync with upstream to avoid conflicts:

```bash
git fetch upstream
git rebase upstream/main
```

---

## Coding Guidelines

### Rust Guidelines (Backend)

**Style:**
- Follow Rust 2024 edition idioms
- Run `cargo fmt` before committing (enforces consistent formatting)
- Run `cargo clippy` and address all warnings
- Use meaningful variable and function names

**Best Practices:**
- Use `Result<T, E>` for error handling
- Prefer `async/await` over manual futures
- Use `sqlx::query!` and `query_as!` macros for compile-time SQL verification
- Add `#[tracing::instrument]` to important functions for logging
- Document public APIs with doc comments (`///`)

**Example:**

```rust
/// Creates a new channel with the given name and description
#[tracing::instrument(skip(pool))]
pub async fn create_channel(
    pool: &PgPool,
    org_id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<Channel, ApiError> {
    // Implementation
}
```

**Database:**
- Always use parameterized queries (never string concatenation)
- Create migrations for schema changes: `sqlx migrate add <name>`
- Add indexes for frequently queried columns
- Use PostgreSQL RLS for security

### TypeScript/Next.js Guidelines (Frontend)

**Style:**
- Use TypeScript strict mode (no `any` types)
- Run `npm run lint` before committing
- Use functional components with hooks
- Follow Next.js App Router conventions

**Best Practices:**
- Use TanStack Query for data fetching
- Keep components small and focused
- Use Tailwind CSS for styling (no custom CSS unless necessary)
- Add proper TypeScript types for all props and state
- Use React 19 concurrent features when appropriate

**Example:**

```typescript
interface MessageListProps {
  channelId: string;
  userId: string;
}

export function MessageList({ channelId, userId }: MessageListProps) {
  const { data: messages, isLoading } = useQuery({
    queryKey: ['messages', channelId],
    queryFn: () => fetchMessages(channelId),
  });

  // Component implementation
}
```

**File Organization:**
- Components: `ui/src/components/`
- Pages: `ui/src/app/`
- API calls: `ui/src/lib/api/`
- Types: `ui/src/types/`
- State management: `ui/src/store/`

### General Guidelines

- **No Secrets**: Never commit API keys, passwords, or sensitive data
- **Security First**: Follow OWASP guidelines, validate all inputs
- **Accessibility**: Ensure UI is keyboard-navigable and screen-reader friendly
- **Performance**: Profile changes, avoid unnecessary re-renders or queries
- **Documentation**: Update README and inline docs when adding features

---

## Testing

### Running Tests

**Backend Tests:**
```bash
cd api
cargo test
```

**Frontend Tests:**
```bash
cd ui
npm test
```

**Linting:**
```bash
# Rust
cargo clippy

# TypeScript
cd ui
npm run lint
```

### Writing Tests

**Rust Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_channel() {
        // Test implementation
    }
}
```

**TypeScript Tests:**
```typescript
import { render, screen } from '@testing-library/react';
import { MessageList } from './MessageList';

describe('MessageList', () => {
  it('renders messages correctly', () => {
    // Test implementation
  });
});
```

### Test Coverage

- Aim for 70%+ code coverage for critical paths
- Write integration tests for API endpoints
- Test error handling and edge cases
- Add E2E tests for critical user flows (login, send message, etc.)

---

## Submitting Changes

### Before Submitting

Make sure you've done the following:

- [ ] Code follows our style guidelines
- [ ] All tests pass locally
- [ ] `cargo clippy` and `npm run lint` pass without warnings
- [ ] You've added tests for new functionality
- [ ] Documentation is updated (README, inline comments, etc.)
- [ ] Commit messages follow our format
- [ ] Branch is rebased on latest `main`

### Creating a Pull Request

1. **Push Your Branch**

   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a Pull Request**

   Go to the [OpenChat repository](https://github.com/ZerosAndOnesLLC/OpenChat) and click "New Pull Request"

3. **Fill Out the PR Template**

   Provide a clear description:
   - **What**: What does this PR do?
   - **Why**: Why is this change needed?
   - **How**: How does it work?
   - **Testing**: How did you test it?
   - **Screenshots**: Include screenshots for UI changes

4. **Example PR Description**

   ```markdown
   ## Summary
   Adds full-text message search powered by PostgreSQL GIN indexes

   ## Changes
   - Added `search_messages` API endpoint
   - Created PostgreSQL full-text search index on messages table
   - Implemented search UI with filters (user, channel, date)
   - Added Redis caching for search results (1 minute TTL)

   ## Testing
   - Added unit tests for search query parsing
   - Tested with 100k+ messages in database
   - Verified performance: <100ms for typical searches

   ## Screenshots
   ![Search UI](screenshot.png)

   Closes #123
   ```

### PR Review Process

1. **Automated Checks**: CI/CD will run tests and linting
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, your PR will be merged!

### After Your PR is Merged

- Delete your feature branch: `git branch -d feature/your-feature-name`
- Update your fork: `git pull upstream main`
- Celebrate! You've contributed to OpenChat! 🎉

---

## Community

### Getting Help

- **GitHub Discussions**: Ask questions, share ideas, get help
- **GitHub Issues**: Report bugs, request features
- **Code Review**: Ask for feedback on your approach before implementing

### Staying Updated

- **Watch the Repository**: Get notifications for new issues and PRs
- **Read the Changelog**: Check release notes for new features
- **Join Discussions**: Participate in architecture and design discussions

### Recognition

We value all contributions! Contributors are:
- Listed in our GitHub contributors page
- Mentioned in release notes for significant contributions
- Invited to help shape the project's direction

---

## Questions?

If you have questions about contributing, feel free to:
- Open a [GitHub Discussion](https://github.com/ZerosAndOnesLLC/OpenChat/discussions)
- Comment on an existing issue
- Reach out to the maintainers

---

## Thank You!

Your contributions help make OpenChat better for everyone. Whether you're fixing a typo or building a major feature, we appreciate your time and effort.

**Happy coding!** 🚀

---

*Built with ❤️ by the OpenChat community*
