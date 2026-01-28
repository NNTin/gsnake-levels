# gsnake-levels

Level management and validation tools for gSnake.

## Dependencies

This crate uses `gsnake-core` as a git dependency, allowing it to build standalone:

```toml
[dependencies.gsnake-core]
git = "https://github.com/nntin/gsnake"
branch = "main"
package = "gsnake-core"
```

### Local Override Detection (Root Repository Mode)

When building in the root repository, `gsnake-levels` automatically detects the context and uses the local version of `gsnake-core` instead of fetching from git. This is implemented via a `build.rs` script that:

1. **Detects root repo context** by checking:
   - `../.git` exists (indicating root repository)
   - `../gsnake-core/Cargo.toml` exists (sibling directory present)

2. **Creates .cargo/config.toml patch** with:
   ```toml
   [patch."https://github.com/nntin/gsnake"]
   gsnake-core = { path = "../gsnake-core/engine/core" }
   ```

3. **Automatically switches modes**:
   - **Root repo mode**: Uses local path for development (faster builds, no network)
   - **Standalone mode**: Uses git dependency (works anywhere)

You can verify which mode is active by checking the build output:
```bash
cargo clean && cargo build
# Root repo: "warning: Root repository detected - using local gsnake-core"
# Standalone: "warning: Standalone mode - using git dependency"
```

The override happens transparently - no manual configuration needed!

## CI/CD

This submodule has its own GitHub Actions workflow at `.github/workflows/ci.yml` to validate standalone builds.

### Testing CI Locally with nektos/act

You can test the CI workflow locally using [nektos/act](https://github.com/nektos/act):

```bash
# Install act (if not already installed)
# Ubuntu/Debian: sudo apt install act
# macOS: brew install act

# Test the build job
cd gsnake-levels
act -j build

# Test the test job
act -j test

# List all available jobs
act -l
```

**Note:** act requires Docker and uses it to simulate GitHub Actions runners. On first run, it will prompt you to select a Docker image size (recommend the medium image: `ghcr.io/catthehacker/ubuntu:act-latest`).

**Known Limitations:**
- Cache actions may not work exactly as on GitHub Actions
- Some network operations may differ from GitHub's environment
- `workflow_dispatch` trigger cannot be tested with act

For simple workflow validation without running jobs, use:
```bash
act -n  # dry run mode
```

## Build

```bash
cargo build
```

Usage:

```text
Run from this directory:
  cargo run -- <COMMAND>

Or use the built binary:
  ./target/debug/gsnake-levels <COMMAND>

Usage: gsnake-levels <COMMAND>

Commands:
  verify                Verify that a level is solvable using its playback file
  replay                Replay a level solution visually in the terminal
  verify-all            Verify all levels in all difficulty folders
  generate-levels-json  Aggregate levels into a single levels.json on stdout
  render                Render asciinema and SVG documentation
  help                  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

```bash
cargo run -- verify --playback playbacks/easy/level_001.json levels/easy/level_001.json
cargo run -- verify-all
cargo run -- replay levels/easy/level_001.json playbacks/easy/level_001.json
# Requires asciinema and svg-term (svg-term-cli) on PATH
cargo run -- render levels/easy/level_001.json playbacks/easy/level_001.json
cargo run -- generate-levels-json --filter easy,medium
# Solve a level and write a playback JSON
cargo run --bin solve_level -- levels/easy/level_001.json playbacks/easy/level_001.json 200
```

**Note:** The `replay` and `render` commands require running in the root repository context where `gsnake-core` is available as a sibling directory, as they use `cargo run` to execute the `gsnake-cli` binary. For standalone usage, install `gsnake-cli` separately and use it directly.

```text
Verify that a level is solvable using its playback file

Usage: gsnake-levels verify [OPTIONS] <LEVEL>

Arguments:
  <LEVEL>  Path to the level JSON file

Options:
      --playback <PLAYBACK>  Optional explicit playback file path
  -h, --help                 Print help
```

```text
Replay a level solution visually in the terminal

Usage: gsnake-levels replay <LEVEL> <PLAYBACK>

Arguments:
  <LEVEL>     Path to the level JSON file
  <PLAYBACK>  Path to the playback JSON file

Options:
  -h, --help  Print help
```

```text
Verify all levels in all difficulty folders

Usage: gsnake-levels verify-all

Options:
  -h, --help  Print help
```

```text
Aggregate levels into a single levels.json on stdout

Usage: gsnake-levels generate-levels-json [OPTIONS]

Options:
      --filter <FILTER>  Optional difficulty filter, e.g. "easy,medium"
      --dry-run          Dry run: do not output JSON
  -h, --help             Print help
```

```text
Render asciinema and SVG documentation

Usage: gsnake-levels render <LEVEL> <PLAYBACK>

Arguments:
  <LEVEL>     Path to the level JSON file
  <PLAYBACK>  Path to the playback JSON file

Options:
  -h, --help  Print help
```
