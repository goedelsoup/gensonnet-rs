# GitHub Actions Workflows

This directory contains the GitHub Actions workflows for the JsonnetGen project.

## Workflows

### 1. CI (`ci.yml`)
**Triggers**: Push to `main`/`develop`, Pull Requests to `main`

Runs comprehensive CI checks:
- **Test Matrix**: Tests against Rust 1.70, stable, and beta
- **Security Audit**: Runs `cargo audit` for vulnerability scanning
- **Dependency Check**: Checks for outdated dependencies
- **Build Matrix**: Builds for Ubuntu, Windows, and macOS

### 2. Release (`release.yml`)
**Triggers**: Push of version tags (`v*`)

Automates the release process:
- **Multi-platform Builds**: Creates binaries for Linux, Windows, and macOS (x86_64 and ARM64)
- **GitHub Release**: Automatically creates GitHub releases with binaries
- **Crates.io Publishing**: Publishes to crates.io (requires `CARGO_REGISTRY_TOKEN` secret)

### 3. Dependency Review (`dependency-review.yml`)
**Triggers**: Pull Requests to `main`

Security-focused dependency scanning:
- Reviews all dependency changes in PRs
- Fails on moderate or higher severity issues
- Provides detailed dependency analysis

### 4. CodeQL (`codeql.yml`)
**Triggers**: Push to `main`, Pull Requests to `main`, Weekly schedule

Advanced security analysis:
- Static code analysis for security vulnerabilities
- Runs on Rust code
- Scheduled weekly scans

### 5. Test Matrix (`test-matrix.yml`)
**Triggers**: Push to `main`/`develop`, Pull Requests to `main`

Comprehensive testing:
- **Configuration Testing**: Tests with default, Istio, and Prometheus configs
- **Feature Testing**: Tests with different feature flags
- **Integration Testing**: Runs integration tests
- **Error Handling**: Tests error scenarios

### 6. Documentation (`docs.yml`)
**Triggers**: Push to `main`, Pull Requests to `main`

Documentation management:
- **Generate Docs**: Creates Rust documentation
- **Deploy Docs**: Deploys to GitHub Pages (main branch only)

## Required Secrets

For full functionality, set up these repository secrets:

### `CARGO_REGISTRY_TOKEN`
- **Purpose**: Publishing to crates.io
- **How to get**: Generate from https://crates.io/settings/tokens
- **Required for**: Release workflow

### `GITHUB_TOKEN`
- **Purpose**: GitHub API access
- **Default**: Automatically provided by GitHub
- **Required for**: All workflows

## Local Development

Use the `justfile` for local development tasks that mirror the CI workflows:

```bash
# Run all checks (equivalent to CI)
just check-all

# Run tests
just test-all

# Build release
just build-release

# Check formatting
just fmt

# Run clippy
just clippy
```

## Workflow Status Badges

Add these badges to your README.md:

```markdown
[![CI](https://github.com/goedelsoup/gensonnet-rs/workflows/CI/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ACI)
[![Release](https://github.com/goedelsoup/gensonnet-rs/workflows/Release/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ARelease)
[![Test Matrix](https://github.com/goedelsoup/gensonnet-rs/workflows/Test%20Matrix/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3A%22Test+Matrix%22)
[![Documentation](https://github.com/goedelsoup/gensonnet-rs/workflows/Documentation/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ADocumentation)
```

## Troubleshooting

### Common Issues

1. **Build failures on Windows/macOS**
   - Ensure all dependencies are properly declared in `Cargo.toml`
   - Check that platform-specific code is properly conditional

2. **Release workflow failures**
   - Verify `CARGO_REGISTRY_TOKEN` is set
   - Check that version tags follow the `v*` pattern
   - Ensure the crate name is available on crates.io

3. **Security scan failures**
   - Update dependencies to fix vulnerabilities
   - Review dependency changes in PRs
   - Address CodeQL findings

### Debugging

- Check workflow logs in the Actions tab
- Use `just test-verbose` for detailed test output locally
- Run `just status` to check your local environment
