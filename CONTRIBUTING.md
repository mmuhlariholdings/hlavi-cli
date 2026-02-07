# Contributing Guidelines

To keep our workflow consistent, please follow these conventions when creating branches, commits, and pull requests.

## Branch Naming

Branches must follow this pattern:

```bash
<type>/<ticket-id>-<short-summary>
```

or if no ticket exists:

```bash
<type>/<short-summary>
```

### Types

- **feature** → new functionality
- **fix** → bug fix
- **chore** → maintenance or tooling
- **docs** → documentation only
- **refactor** → code restructuring without changing behavior
- **test** → adding or updating tests

### Examples

- `feature/HLAVI-50-Create-Docker-File`
- `fix/HLAVI-72-Handle-Null-Values`
- `docs/Update-Setup-Instructions`
- `feature/Add-Board-Visualization`

### Notes:

- Include the ticket ID if available (future: HLAVI-XXX, currently optional)
- Use Pascal-Case or hyphenated words for the summary
- Keep summaries concise but descriptive
- All feature branches merge into `main` via Pull Request

## Commit Message Convention

Commit messages must follow this style:

```bash
<type>: <short summary>
```

or with ticket ID:

```bash
<type>(TICKET-ID): <short summary>
```

### Types

- **feature** → new feature
- **fix** → bug fix
- **chore** → maintenance or tooling
- **docs** → documentation
- **refactor** → code restructuring
- **test** → adding or updating tests
- **ci** → CI/CD changes

### Examples

- `feature: add board visualization command`
- `fix(HLAVI-72): handle null values in login flow`
- `docs: update Docker Compose instructions`
- `refactor: split file operations into separate modules`
- `ci: add release workflow for multi-platform builds`

### Notes:

- Use imperative mood in the summary (add, fix, update)
- Keep summaries concise (< 72 characters)
- Ticket ID optional until tracking system is finalized

## Pull Request Titles

PR titles must follow this pattern:

```bash
[XX] <ticket-id>: <short summary>
```

or without ticket:

```bash
[XX] <short summary>
```

### Where:

- **XX** → Your initials (first 2 characters of first name + surname)
- **ticket-id** → Optional ticket identifier (e.g., HLAVI-50)
- **short summary** → Description of the completed work

### Examples

- `[MaMu] HLAVI-50: Created Docker File`
- `[MaMu] Fix null value handling in login`
- `[JoDo] Add board visualization feature`

### Notes:

- Use past tense for the summary (describes what was done)
- Keep summaries under 70 characters
- Link to ticket in PR description when available

## Dependency Management

This project uses **Git dependencies** with tags/branches for version control. Dependencies like `hlavi-core` and `hlavi-agent` are fetched from GitHub repositories rather than local paths.

### Current Dependency Configuration

```toml
[dependencies]
hlavi-core = { git = "https://github.com/mmuhlariholdings/hlavi-core", branch = "main" }
```

### Working with Dependencies

**When making changes to dependencies:**

1. **Make changes in the dependency repository** (e.g., hlavi-core):
   ```bash
   cd ../hlavi-core
   # Make changes
   git add .
   git commit -m "feature: add new functionality"
   git push origin main
   ```

2. **Wait for CI to validate** the dependency changes

3. **Update Cargo.lock in dependent project**:
   ```bash
   cd ../hlavi-cli
   cargo update -p hlavi-core
   ```

4. **Make changes that use the new functionality**:
   ```bash
   git add Cargo.lock
   git commit -m "feature: use new core functionality"
   git push origin main
   ```

### Using Specific Versions

Once dependencies stabilize, update to specific tags:

```toml
[dependencies]
hlavi-core = { git = "https://github.com/mmuhlariholdings/hlavi-core", tag = "v0.1.0" }
```

### Benefits of This Approach

- ✅ Explicit version control - always know what version you depend on
- ✅ Prevents version drift between repositories
- ✅ CI automatically fetches correct dependencies
- ✅ Forces proper versioning discipline
- ✅ No need to manually checkout dependencies locally

### Important Notes

- Always commit and push dependency changes **before** updating dependent projects
- CI will fail if dependencies reference uncommitted changes
- Use `cargo update -p <dependency-name>` to fetch latest changes from git
- Cargo caches git dependencies in `~/.cargo/git/checkouts/`

## Workflow

### 1. Create Feature Branch

```bash
# Start from main
git checkout main
git pull origin main

# Create feature branch
git checkout -b feature/HLAVI-50-Add-Board-View
```

### 2. Make Changes and Commit

```bash
git add .
git commit -m "feature(HLAVI-50): add board visualization with kanban columns"
```

### 3. Push and Create PR

```bash
git push origin feature/HLAVI-50-Add-Board-View
```

Then open a Pull Request on GitHub with title:
```
[MaMu] HLAVI-50: Added Board Visualization
```

### 4. Code Review

- Request reviews from maintainers
- Address feedback and push updates
- Ensure CI passes (tests, clippy, formatting)

### 5. Merge

- Maintainer merges via GitHub after approval
- Feature branch is automatically deleted
- Changes flow: `feature/* → main`

## Protected Branch Rules

The `main` branch is protected:

- ✅ Requires pull request reviews before merging
- ✅ Requires status checks to pass (CI)
- ✅ Requires branches to be up to date before merging
- ✅ Requires conversation resolution before merging
- ❌ Direct pushes not allowed
- ❌ Force pushes not allowed

## Environment Promotion

Deployments follow this flow:

```
main (dev) → staging → production
```

- **dev**: Automatic deployment on merge to main
- **staging**: Manual promotion with approval
- **production**: Manual promotion with stricter approval

## Example End-to-End Flow

**Ticket**: HLAVI-50: Add board visualization

1. **Create branch**:
   ```bash
   git checkout -b feature/HLAVI-50-Add-Board-View
   ```

2. **Make changes and commit**:
   ```bash
   git commit -m "feature(HLAVI-50): add kanban board view with drag-drop"
   ```

3. **Push and open PR**:
   ```bash
   git push origin feature/HLAVI-50-Add-Board-View
   ```
   PR Title: `[MaMu] HLAVI-50: Added Board Visualization`

4. **Get approval and merge** → Auto-deploys to dev

5. **Promote to staging** → Manual approval

6. **Promote to production** → Manual approval

This ensures full traceability: `branch → commit → PR → deployment`

## Code Quality Standards

Before submitting a PR, ensure:

- [ ] Code passes `cargo fmt`
- [ ] Code passes `cargo clippy`
- [ ] All tests pass `cargo test`
- [ ] New features have tests
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for significant changes)

## Questions?

If you have questions about contributing, please:
- Check existing issues and PRs
- Ask in GitHub Discussions
- Contact maintainers

Thank you for contributing to Hlavi! 🚀
