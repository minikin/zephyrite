# Conventional Commits Setup

- [Conventional Commits Setup](#conventional-commits-setup)
  - [🚀 Quick Setup (Essential)](#-quick-setup-essential)
    - [1. Set Git Commit Template](#1-set-git-commit-template)
    - [2. Start Using Conventional Commits](#2-start-using-conventional-commits)
  - [📝 **Commit Format**](#-commit-format)
    - [**Types:**](#types)
    - [**Scopes for Zephyrite:**](#scopes-for-zephyrite)
    - [**Examples:**](#examples)
  - [💡 **Usage Examples**](#-usage-examples)
    - [**Making Your First Commit:**](#making-your-first-commit)
    - [**Quick Command Reference:**](#quick-command-reference)
  - [🔧 **Optional: Automated Tools**](#-optional-automated-tools)
    - [1. Install Node.js Tools](#1-install-nodejs-tools)
    - [2. Interactive Commits](#2-interactive-commits)
  - [✅ **Benefits**](#-benefits)
  - [🎯 **Commit Examples**](#-commit-examples)
  - [📚 **Resources**](#-resources)

## 🚀 Quick Setup (Essential)

### 1. Set Git Commit Template

```bash
just setup-git
```

This configures your git to show a helpful template when you commit.

### 2. Start Using Conventional Commits

```bash
# Stage your changes
git add .

# Commit with template
git commit
```

Git will open your editor with the commit template showing examples.

## 📝 **Commit Format**

```bash
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

### **Types:**

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `test` - Adding/updating tests
- `refactor` - Code refactoring
- `style` - Code style changes (formatting)
- `chore` - Maintenance tasks

### **Scopes for Zephyrite:**

- `storage` - Storage engine, memory operations
- `server` - HTTP server, endpoints
- `config` - Configuration management
- `api` - API design and endpoints
- `cli` - Command-line interface
- `docs` - Documentation

### **Examples:**

```bash
feat(server): add HTTP GET endpoint
fix(storage): handle empty key validation
docs(readme): update installation guide
test(storage): add memory storage tests
chore(deps): update tokio dependency
```

## 💡 **Usage Examples**

### **Making Your First Commit:**

```bash
# 1. Make changes to code
# 2. Stage changes
git add .

# 3. Commit with template
git commit

# 4. Fill in the template:
feat(storage): add basic in-memory storage

Implement HashMap-based storage for key-value pairs.
Includes get, put, delete, and list operations.
```

### **Quick Command Reference:**

```bash
just setup-git         # One-time setup
just commit-examples    # Show commit examples
just commit            # Interactive commit (if tools installed)
git commit             # Use template
```

## 🔧 **Optional: Automated Tools**

If you want automated validation and interactive commits:

### 1. Install Node.js Tools

```bash
npm install
```

### 2. Interactive Commits

```bash
just commit
# or
npx git-cz
```

This gives you guided prompts for type, scope, and description.

## ✅ **Benefits**

- 📝 **Consistent commit messages** across the project
- 🔍 **Searchable history** by type or scope
- 📊 **Clear project progress** tracking
- 🤖 **Future automation** (changelog, releases)

## 🎯 **Commit Examples**

Here are typical commits you'll make:

```bash
feat(storage): implement memory storage trait
feat(storage): add key validation
feat(server): add HTTP server foundation
feat(api): implement GET /keys/:key endpoint
feat(api): implement PUT /keys/:key endpoint
test(storage): add memory storage tests
test(server): add HTTP endpoint tests
docs(readme): update API documentation
fix(server): handle malformed JSON requests
chore(deps): add axum dependency
```

## 📚 **Resources**

- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- Zephyrite Commit Examples: `just commit-examples`

**Ready to start making conventional commits!** 🚀

Use `just setup-git` to get started, then `git commit` or `just commit` for your first conventional commit.
