Build:

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

Examples:

```bash
cargo run -- verify --playback playbacks/easy/level_001.json levels/easy/level_001.json
cargo run -- verify-all
cargo run -- replay levels/easy/level_001.json playbacks/easy/level_001.json
# Requires asciinema and svg-term (svg-term-cli) on PATH
cargo run -- render levels/easy/level_001.json playbacks/easy/level_001.json
cargo run -- generate-levels-json --filter easy,medium
```

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
