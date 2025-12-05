# Chapter 11: Continuous Integration

Continuous Integration (CI) automatically builds and tests your code whenever you push changes. This chapter covers setting up GitHub Actions to ensure LockSmith stays healthy.

## What Is CI?

**Continuous Integration** is the practice of frequently merging code changes and automatically verifying them. A CI system:

1. **Triggers** on events (push, pull request)
2. **Runs** automated checks (build, test, lint)
3. **Reports** success or failure
4. **Blocks** merging if checks fail

## Why Use CI?

CI provides:

1. **Automated testing** — Tests run on every change, not just when you remember
2. **Consistency** — Same environment for all builds
3. **Early detection** — Catch bugs before they reach main branch
4. **Documentation** — CI config shows how to build/test the project
5. **Confidence** — Green checkmarks mean the code works

## GitHub Actions Basics

GitHub Actions is GitHub's built-in CI/CD platform. Key concepts:

| Term | Meaning |
|------|---------|
| **Workflow** | Automated process defined in YAML |
| **Job** | A set of steps that run on the same runner |
| **Step** | Individual task within a job |
| **Runner** | Virtual machine that executes jobs |
| **Trigger** | Event that starts a workflow |

### Workflow File Location

Workflows live in `.github/workflows/`:

```
LockSmith/
└── .github/
    └── workflows/
        └── ci.yml
```

## LockSmith's CI Workflow

Here's the complete workflow file:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    name: Rustfmt
    runs-on: windows-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        working-directory: ./src
        run: cargo fmt -- --check

  clippy:
    name: Clippy
    runs-on: windows-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src

      - name: Run clippy
        working-directory: ./src
        run: cargo clippy -- -D warnings

  build:
    name: Build
    runs-on: windows-latest
    needs: [fmt, clippy]
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src

      - name: Build debug binary
        working-directory: ./src
        run: cargo build

      - name: Upload debug binary
        uses: actions/upload-artifact@v4
        with:
          name: locksmith-debug
          path: src/target/debug/locksmith.exe
          retention-days: 7

  build-release:
    name: Build Release
    runs-on: windows-latest
    needs: [fmt, clippy]
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src

      - name: Build release
        working-directory: ./src
        run: cargo build --release

      - name: Upload release binary
        uses: actions/upload-artifact@v4
        with:
          name: locksmith-release
          path: src/target/release/locksmith.exe
          retention-days: 30
```

## Understanding the Workflow

### Workflow Name

```yaml
name: CI
```

This appears in the GitHub Actions tab and in status checks.

### Triggers

```yaml
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
```

The workflow runs when:
- Pushing to `main` or `develop` branches
- Opening/updating a PR targeting these branches

Other trigger options:
```yaml
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
  workflow_dispatch:      # Manual trigger
  release:
    types: [published]    # On release publication
```

### Environment Variables

```yaml
env:
  CARGO_TERM_COLOR: always
```

Global environment variables available to all jobs. `CARGO_TERM_COLOR: always` forces colored output even in non-terminal environments.

## Job: Rustfmt

Checks that code is properly formatted:

```yaml
fmt:
  name: Rustfmt
  runs-on: windows-latest
  steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt

    - name: Check formatting
      working-directory: ./src
      run: cargo fmt -- --check
```

### Understanding Steps

#### Checkout

```yaml
- name: Checkout code
  uses: actions/checkout@v4
```

`uses` runs a pre-built action. `actions/checkout@v4` clones your repository. Without this, the runner has no code!

#### Setup Rust

```yaml
- name: Setup Rust
  uses: dtolnay/rust-toolchain@stable
  with:
    components: rustfmt
```

`dtolnay/rust-toolchain` is a popular action for setting up Rust:
- `stable` — Use stable toolchain
- `components: rustfmt` — Also install rustfmt

#### Check Formatting

```yaml
- name: Check formatting
  working-directory: ./src
  run: cargo fmt -- --check
```

- `working-directory` — Run command from this directory
- `run` — Execute a shell command
- `cargo fmt -- --check` — Check format without modifying files (fails if changes needed)

## Job: Clippy

Runs the Rust linter:

```yaml
clippy:
  name: Clippy
  runs-on: windows-latest
  steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy

    - name: Cache cargo
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: src

    - name: Run clippy
      working-directory: ./src
      run: cargo clippy -- -D warnings
```

### Caching Dependencies

```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: src
```

`Swatinem/rust-cache` caches:
- `~/.cargo/registry` — Downloaded crates
- `target/` — Compiled dependencies

This dramatically speeds up subsequent builds (from minutes to seconds).

### Clippy Options

```yaml
run: cargo clippy -- -D warnings
```

The `-- -D warnings` passes flags to clippy:
- `-D warnings` — Treat warnings as errors (fail the build)

This ensures all warnings are addressed before merging.

## Job: Build

Builds the debug binary:

```yaml
build:
  name: Build
  runs-on: windows-latest
  needs: [fmt, clippy]
  steps:
    # ... checkout, setup, cache ...

    - name: Build debug binary
      working-directory: ./src
      run: cargo build

    - name: Upload debug binary
      uses: actions/upload-artifact@v4
      with:
        name: locksmith-debug
        path: src/target/debug/locksmith.exe
        retention-days: 7
```

### Job Dependencies

```yaml
needs: [fmt, clippy]
```

This job waits for `fmt` and `clippy` to complete successfully. If either fails, `build` doesn't run.

This creates a pipeline:
```
┌─────┐     ┌─────────┐
│ fmt │────►│         │
└─────┘     │  build  │
┌─────────┐ │         │
│ clippy  │►│         │
└─────────┘ └─────────┘
```

### Uploading Artifacts

```yaml
- name: Upload debug binary
  uses: actions/upload-artifact@v4
  with:
    name: locksmith-debug
    path: src/target/debug/locksmith.exe
    retention-days: 7
```

`actions/upload-artifact` saves build outputs:
- `name` — Artifact name (appears in GitHub UI)
- `path` — File or directory to upload
- `retention-days` — How long to keep (default: 90)

Artifacts can be downloaded from the Actions run page.

## Job: Build Release

Only runs on pushes to main:

```yaml
build-release:
  name: Build Release
  runs-on: windows-latest
  needs: [fmt, clippy]
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  steps:
    # ... setup ...

    - name: Build release
      working-directory: ./src
      run: cargo build --release

    - name: Upload release binary
      uses: actions/upload-artifact@v4
      with:
        name: locksmith-release
        path: src/target/release/locksmith.exe
        retention-days: 30
```

### Conditional Execution

```yaml
if: github.event_name == 'push' && github.ref == 'refs/heads/main'
```

This job only runs when:
- Event is a push (not PR)
- Branch is main

This saves CI time—release builds are expensive and only needed for the main branch.

### Context Variables

| Variable | Meaning |
|----------|---------|
| `github.event_name` | Trigger type (push, pull_request, etc.) |
| `github.ref` | Full ref (refs/heads/main, refs/pull/123/merge) |
| `github.sha` | Commit SHA |
| `github.actor` | User who triggered the workflow |
| `github.repository` | Owner/repo name |

## Adding Tests

To run tests in CI, add a job:

```yaml
test:
  name: Test
  runs-on: windows-latest
  needs: [fmt, clippy]
  steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache cargo
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: src

    - name: Run tests
      working-directory: ./src
      run: cargo test --all-features
```

### Test Options

```yaml
run: cargo test --all-features
```

Common test flags:
- `--all-features` — Enable all feature flags
- `--no-fail-fast` — Continue after first failure
- `-- --nocapture` — Show println! output
- `-- --test-threads=1` — Run tests sequentially

## CI Best Practices

### 1. Fail Fast

Run quick checks first:

```yaml
build:
  needs: [fmt, clippy]  # Don't build if format/lint fails
```

### 2. Cache Dependencies

Always cache Cargo's registry and target:

```yaml
- uses: Swatinem/rust-cache@v2
```

### 3. Use Specific Action Versions

Pin action versions with `@v4`, not `@main`:

```yaml
uses: actions/checkout@v4  # Good
uses: actions/checkout@main  # Bad - can break
```

### 4. Minimize Secrets Exposure

For private repos, be careful with secrets in PRs from forks.

### 5. Keep Workflows DRY

For complex workflows, use:
- Reusable workflows
- Composite actions
- Matrix builds

## Matrix Builds

Test on multiple configurations:

```yaml
test:
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [windows-latest, ubuntu-latest, macos-latest]
      rust: [stable, beta]
  steps:
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
```

This creates 6 jobs (3 OS × 2 Rust versions).

Note: LockSmith is Windows-only, so we don't use matrix builds.

## Viewing CI Results

### On GitHub

1. Go to repository
2. Click "Actions" tab
3. See workflow runs
4. Click a run for details
5. Click a job for step-by-step logs

### Status Badges

Add to README.md:

```markdown
![CI](https://github.com/username/LockSmith/actions/workflows/ci.yml/badge.svg)
```

This shows a live status badge.

### Pull Request Checks

PRs show check status:
- ✅ All checks passed
- ❌ Checks failed
- 🟡 Checks in progress

You can require checks to pass before merging (repository settings).

## Troubleshooting CI

### Build Succeeds Locally But Fails in CI

Common causes:
- Different Rust version
- Missing dependencies
- Case-sensitivity (Linux CI)
- Cached artifacts from old runs

Solutions:
- Specify Rust version exactly
- Clear CI cache
- Test in clean environment

### Flaky Tests

Tests that sometimes pass, sometimes fail:
- Race conditions
- Time-sensitive assertions
- External dependencies

Solutions:
- Use proper synchronization
- Mock external services
- Increase timeouts

### Slow CI

If CI takes too long:
- Cache aggressively
- Run jobs in parallel
- Skip expensive checks on drafts
- Use faster runners (paid)

## Advanced: Creating Releases

Automate release creation:

```yaml
release:
  name: Release
  runs-on: windows-latest
  if: startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: actions/checkout@v4

    - uses: dtolnay/rust-toolchain@stable

    - name: Build release
      run: cargo build --release
      working-directory: ./src

    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: src/target/release/locksmith.exe
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

When you push a tag like `v1.0.0`, this:
1. Builds the release binary
2. Creates a GitHub Release
3. Attaches the binary

## What We've Accomplished

You now understand:

✅ CI concepts — Workflows, jobs, steps, triggers  
✅ GitHub Actions syntax — YAML configuration  
✅ Key actions — checkout, rust-toolchain, cache, upload-artifact  
✅ Job dependencies — needs, if conditions  
✅ Best practices — Caching, fail fast, version pinning  

## Key Takeaways

1. **CI catches issues early** — Before they reach production
2. **Cache everything** — Rust builds are slow without caching
3. **Fail fast** — Run quick checks before expensive ones
4. **Use job dependencies** — Create efficient pipelines
5. **Conditional jobs** — Skip unnecessary work (like release builds on PRs)

## Looking Ahead

In the next chapter, we'll put everything together—the complete application with all components working in harmony.

---

**Previous Chapter:** [← Build Configuration](10-build-configuration.md)  
**Next Chapter:** [Final Assembly →](12-final-assembly.md)