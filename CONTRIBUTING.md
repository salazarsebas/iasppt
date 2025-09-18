# 🤝 Contributing to DeAI Platform

Welcome to the DeAI Platform! We're excited to have you contribute to the future of decentralized AI computing. This guide will help you get started with contributing to our project.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Code Style](#code-style)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Community](#community)
- [Recognition](#recognition)

## 📜 Code of Conduct

### Our Pledge

We are committed to making participation in our project a harassment-free experience for everyone, regardless of:

- Age, body size, disability, ethnicity, gender identity and expression
- Level of experience, education, socio-economic status
- Nationality, personal appearance, race, religion
- Sexual identity and orientation

### Expected Behavior

✅ **Be respectful and inclusive**  
✅ **Use welcoming and inclusive language**  
✅ **Be collaborative and constructive**  
✅ **Focus on what is best for the community**  
✅ **Show empathy towards other community members**

### Unacceptable Behavior

❌ **Harassment, discrimination, or abuse**  
❌ **Trolling, insulting, or derogatory comments**  
❌ **Personal or political attacks**  
❌ **Publishing others' private information**  
❌ **Other unprofessional conduct**

## 🚀 Getting Started

### Ways to Contribute

| Contribution Type | Description | Skill Level | Time Commitment |
|-------------------|-------------|-------------|-----------------|
| 🐛 **Bug Reports** | Report issues and bugs | Beginner | 15-30 minutes |
| 📝 **Documentation** | Improve docs and guides | Beginner | 1-2 hours |
| 🔧 **Bug Fixes** | Fix reported issues | Intermediate | 2-4 hours |
| ✨ **Features** | Implement new features | Advanced | 1-2 weeks |
| 🧪 **Testing** | Write and improve tests | Intermediate | 2-6 hours |
| 🎨 **UI/UX** | Improve user experience | Intermediate | 1-2 weeks |
| 🏗️ **Architecture** | System design improvements | Expert | 2-4 weeks |

### Quick Start Checklist

- [ ] Read this contributing guide
- [ ] Join our [Discord community](https://discord.gg/deai)
- [ ] Check out [good first issues](https://github.com/deai-platform/deai/labels/good%20first%20issue)
- [ ] Set up your development environment
- [ ] Make your first contribution!

## 🛠️ Development Setup

### Prerequisites

Ensure you have the following installed:

```bash
# Required Tools
- Git 2.40+
- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Python 3.9+ and pip
- Docker and Docker Compose
- NEAR CLI

# Optional but Recommended
- VS Code with Rust and TypeScript extensions
- Git hooks for automated formatting
- Local PostgreSQL and Redis for testing
```

### Environment Setup

1. **Fork and Clone the Repository**

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/deai.git
cd deai

# Add upstream remote
git remote add upstream https://github.com/deai-platform/deai.git
```

2. **Install Dependencies**

```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies
cd web-dashboard && npm install
cd ../api-gateway && npm install  # If applicable
cd ..

# Install Python dependencies
pip install -r requirements.txt
pip install -r tests/requirements.txt
```

3. **Set Up Local Environment**

```bash
# Copy environment template
cp .env.example .env

# Start local services
docker-compose up -d postgres redis

# Run database migrations
cargo run --bin migrate

# Verify setup
cargo test
npm test
```

### IDE Configuration

#### VS Code (Recommended)

```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "typescript.preferences.includePackageJsonAutoImports": "on",
  "files.exclude": {
    "**/target": true,
    "**/node_modules": true
  }
}
```

#### Recommended Extensions

- `rust-lang.rust-analyzer` - Rust language support
- `bradlc.vscode-tailwindcss` - Tailwind CSS support
- `esbenp.prettier-vscode` - Code formatting
- `ms-vscode.vscode-typescript-next` - TypeScript support

## 📋 Contributing Guidelines

### Issue First Approach

Before starting work on a feature or significant change:

1. **Check existing issues** to avoid duplicate work
2. **Create an issue** describing the problem or enhancement
3. **Discuss the approach** with maintainers and community
4. **Get approval** before starting implementation

### Branch Naming Convention

Use descriptive branch names that include the issue number:

```bash
# Feature branches
git checkout -b feature/123-add-node-monitoring

# Bug fix branches  
git checkout -b fix/456-memory-leak-api-gateway

# Documentation branches
git checkout -b docs/789-update-api-reference

# Hotfix branches
git checkout -b hotfix/critical-security-patch
```

### Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```bash
# Format
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]

# Examples
feat(api): add task cancellation endpoint
fix(node): resolve memory leak in heartbeat
docs(readme): update installation instructions
test(integration): add end-to-end task flow test
refactor(auth): simplify JWT validation logic
```

### Commit Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(api): add task priority queuing` |
| `fix` | Bug fix | `fix(node): resolve connection timeout` |
| `docs` | Documentation | `docs(api): add WebSocket examples` |
| `style` | Code style changes | `style: format code with prettier` |
| `refactor` | Code refactoring | `refactor(db): optimize query performance` |
| `test` | Testing changes | `test(unit): add node manager tests` |
| `chore` | Maintenance | `chore(deps): update dependencies` |

## 🔄 Pull Request Process

### Before Submitting

✅ **Ensure all tests pass locally**  
✅ **Update documentation if needed**  
✅ **Add tests for new functionality**  
✅ **Follow code style guidelines**  
✅ **Update CHANGELOG.md if applicable**

### PR Template

When creating a pull request, use this template:

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Performance impact assessed

## Screenshots (if applicable)
Add screenshots for UI changes.

## Checklist
- [ ] My code follows the style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
```

### Review Process

1. **Automated Checks**: CI/CD pipeline runs tests and checks
2. **Code Review**: At least one maintainer reviews the code
3. **Community Review**: Community members may provide feedback
4. **Integration Testing**: Changes tested in staging environment
5. **Approval**: Maintainer approves and merges the PR

### Review Criteria

| Aspect | Requirements |
|--------|--------------|
| **Functionality** | Code works as intended and solves the problem |
| **Code Quality** | Clean, readable, and maintainable code |
| **Testing** | Adequate test coverage and all tests pass |
| **Documentation** | Updated docs and clear code comments |
| **Performance** | No significant performance regressions |
| **Security** | No security vulnerabilities introduced |

## 🐛 Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected Behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
- OS: [e.g. Ubuntu 20.04]
- Browser: [e.g. Chrome 91.0]
- Node Version: [e.g. 18.16.0]
- Rust Version: [e.g. 1.70.0]

**Additional Context**
Add any other context about the problem here.
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear and concise description of what the problem is.

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.

**Implementation Notes**
If you have ideas about how this could be implemented, please share them.
```

## 🎨 Code Style

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// Good: Clear naming and documentation
/// Processes a task and returns the result.
/// 
/// # Arguments
/// * `task` - The task to process
/// * `timeout` - Maximum processing time in seconds
/// 
/// # Returns
/// Result containing the processed output or an error
pub async fn process_task(
    task: &Task,
    timeout: Duration,
) -> Result<TaskResult, ProcessingError> {
    // Implementation
}

// Use meaningful variable names
let task_processing_time = start_time.elapsed();

// Prefer explicit error handling
match result {
    Ok(value) => value,
    Err(e) => return Err(ProcessingError::from(e)),
}
```

### TypeScript Style Guide

Follow [Airbnb TypeScript Style Guide](https://github.com/airbnb/javascript/tree/master/packages/eslint-config-airbnb-typescript):

```typescript
// Good: Interface definitions
interface TaskSubmissionRequest {
  taskType: string;
  modelName: string;
  inputData: string;
  maxCost: string;
  priority: number;
}

// Use async/await for promises
const submitTask = async (request: TaskSubmissionRequest): Promise<Task> => {
  try {
    const response = await fetch('/api/v1/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error('Failed to submit task:', error);
    throw error;
  }
};
```

### Python Style Guide

Follow [PEP 8](https://peps.python.org/pep-0008/) and use [Black](https://black.readthedocs.io/) for formatting:

```python
# Good: Type hints and docstrings
from typing import Optional, Dict, Any
import asyncio

async def process_ai_task(
    task_data: Dict[str, Any],
    model_path: str,
    timeout: Optional[float] = None
) -> Dict[str, Any]:
    """
    Process an AI task using the specified model.
    
    Args:
        task_data: Input data for the task
        model_path: Path to the AI model
        timeout: Optional timeout in seconds
        
    Returns:
        Dictionary containing the processing results
        
    Raises:
        ProcessingError: If task processing fails
    """
    try:
        # Implementation here
        result = await run_inference(task_data, model_path)
        return {"status": "completed", "result": result}
    except Exception as e:
        raise ProcessingError(f"Task processing failed: {e}")
```

### Formatting Tools

Run these commands before committing:

```bash
# Rust formatting
cargo fmt
cargo clippy --all-targets --all-features

# TypeScript formatting
npm run lint
npm run format

# Python formatting
black .
flake8 .
mypy .
```

## 🧪 Testing Requirements

### Test Categories

| Test Type | Coverage | Command | Required For |
|-----------|----------|---------|--------------|
| **Unit Tests** | >90% | `cargo test` | All PRs |
| **Integration Tests** | >80% | `npm run test:integration` | Feature PRs |
| **End-to-End Tests** | Core flows | `npm run test:e2e` | Major features |
| **Performance Tests** | Benchmarks | `python tests/performance_test.py` | Performance PRs |
| **Security Tests** | Vulnerabilities | `python tests/security_test.py` | Security PRs |

### Writing Tests

#### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_task_processing() {
        // Arrange
        let task = Task::new("test_task", "gpt2");
        let processor = TaskProcessor::new();
        
        // Act
        let result = processor.process(task).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TaskStatus::Completed);
    }
}
```

#### TypeScript Tests

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { TaskSubmissionForm } from './TaskSubmissionForm';

describe('TaskSubmissionForm', () => {
  it('should submit task when form is valid', async () => {
    // Arrange
    const mockSubmit = jest.fn();
    render(<TaskSubmissionForm onSubmit={mockSubmit} />);
    
    // Act
    fireEvent.change(screen.getByLabelText(/task type/i), {
      target: { value: 'text_generation' }
    });
    fireEvent.click(screen.getByRole('button', { name: /submit/i }));
    
    // Assert
    await waitFor(() => {
      expect(mockSubmit).toHaveBeenCalledWith({
        taskType: 'text_generation'
      });
    });
  });
});
```

## 📚 Documentation

### Documentation Types

| Type | Location | Purpose |
|------|----------|---------|
| **API Docs** | `docs/api/` | API reference and examples |
| **Architecture** | `docs/ARCHITECTURE.md` | System design and structure |
| **User Guides** | `docs/guides/` | Step-by-step tutorials |
| **Code Comments** | Inline | Code explanation and context |
| **README** | `README.md` | Project overview and setup |

### Documentation Standards

✅ **Write clear, concise explanations**  
✅ **Include code examples**  
✅ **Add diagrams where helpful**  
✅ **Keep documentation up-to-date**  
✅ **Use consistent formatting**

### Adding Documentation

```bash
# For new features, update:
1. API documentation (if applicable)
2. User guides (if user-facing)
3. Code comments (for complex logic)
4. CHANGELOG.md (for notable changes)
```

## 👥 Community

### Communication Channels

| Channel | Purpose | Link |
|---------|---------|------|
| **Discord** | Real-time chat and support | [Join Discord](https://discord.gg/deai) |
| **GitHub Issues** | Bug reports and feature requests | [View Issues](https://github.com/deai-platform/deai/issues) |
| **GitHub Discussions** | Community discussions | [Join Discussions](https://github.com/deai-platform/deai/discussions) |
| **Twitter** | Announcements and updates | [@deai_network](https://twitter.com/deai_network) |

### Community Events

📅 **Weekly Developer Calls**: Thursdays 3PM UTC  
🎓 **Monthly Workshops**: First Saturday of each month  
🏆 **Quarterly Hackathons**: Seasonal competitions  
📝 **Documentation Sprints**: As needed

### Getting Help

1. **Check existing documentation** first
2. **Search GitHub issues** for similar problems
3. **Ask in Discord** for quick questions
4. **Create a GitHub issue** for bugs or feature requests
5. **Join developer calls** for complex discussions

## 🏆 Recognition

### Contributor Levels

| Level | Requirements | Benefits |
|-------|--------------|----------|
| **Contributor** | First merged PR | Listed in contributors |
| **Regular Contributor** | 5+ merged PRs | Special Discord role |
| **Core Contributor** | 20+ PRs, regular participation | Vote on technical decisions |
| **Maintainer** | Significant contributions, community trust | Write access, release management |

### Contributor Rewards

🎁 **Swag**: T-shirts, stickers, and merchandise  
🪙 **Tokens**: DEAI tokens for significant contributions  
📜 **Certificates**: Digital contribution certificates  
🌟 **Recognition**: Featured in newsletters and social media

### Hall of Fame

We recognize outstanding contributors in our [CONTRIBUTORS.md](CONTRIBUTORS.md) file and during community events.

## 🚀 Getting Your First PR Merged

### Good First Issues

Look for issues labeled:
- `good first issue` - Perfect for newcomers
- `help wanted` - Community help needed
- `documentation` - Documentation improvements
- `bug` - Bug fixes (often straightforward)

### Step-by-Step Guide

1. **Find an Issue**
   ```bash
   # Browse good first issues
   https://github.com/deai-platform/deai/labels/good%20first%20issue
   ```

2. **Claim the Issue**
   ```bash
   # Comment on the issue
   "I'd like to work on this issue!"
   ```

3. **Create Your Branch**
   ```bash
   git checkout -b fix/123-your-issue-description
   ```

4. **Make Your Changes**
   ```bash
   # Make changes, test locally
   cargo test
   npm test
   ```

5. **Submit Your PR**
   ```bash
   git push origin fix/123-your-issue-description
   # Create PR on GitHub
   ```

6. **Respond to Feedback**
   - Address reviewer comments
   - Update your branch as needed
   - Be patient and collaborative

### Success Tips

💡 **Start small** - Fix typos, improve docs, or tackle simple bugs  
💡 **Ask questions** - Don't hesitate to ask for help  
💡 **Be patient** - Reviews take time, especially for new contributors  
💡 **Learn from feedback** - Each review makes you a better developer  
💡 **Stay engaged** - Participate in community discussions

## 📞 Contact

### Maintainers

- **Core Team**: core@deai.network
- **Security Issues**: security@deai.network
- **General Questions**: hello@deai.network

### Quick Links

- 🌐 **Website**: [deai.network](https://deai.network)
- 📖 **Documentation**: [docs.deai.network](https://docs.deai.network)
- 💬 **Discord**: [discord.gg/deai](https://discord.gg/deai)
- 🐦 **Twitter**: [@deai_network](https://twitter.com/deai_network)

---

Thank you for contributing to DeAI Platform! Together, we're building the future of decentralized AI computing. 🚀