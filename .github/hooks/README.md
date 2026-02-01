# Git Hooks for gsnake-levels

This directory contains shared git hooks for the gsnake-levels repository.

## Enabling Hooks

To enable these hooks, run from the gsnake-levels directory:

```bash
git config core.hooksPath .github/hooks
```

## Verification

Verify that hooks are enabled:

```bash
git config core.hooksPath
```

This should output: `.github/hooks`

## Disabling Hooks

To disable the hooks and revert to default behavior:

```bash
git config --unset core.hooksPath
```

## Available Hooks

### pre-commit

The pre-commit hook runs the following checks:

1. **Format Check**: `cargo fmt --all -- --check`
2. **Linting**: `cargo clippy --all-targets -- -D warnings`
3. **Type Check**: `cargo check`
4. **Build**: `cargo build`

**Note: Tests are NOT run in pre-commit hooks for speed.** Run tests manually with `cargo test` before pushing.
