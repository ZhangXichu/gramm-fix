# gramm-fix

German grammar checker CLI with compiler-style feedback.

## Commands

```bash
cargo build
cargo run                        # interactive REPL
cargo run -- fix "sentence"      # single-shot check
```

## Project structure

```
src/
  main.rs       # entry point — declares modules, calls cli::run()
  cli.rs        # clap CLI definition, REPL loop, display logic
  checker.rs    # grammar checking (stub → LLM backend)
```

## Output format

Errors are shown in compiler style:

```
✘ Ich habe ein Apfel gegessen
✔ Ich habe einen Apfel gegessen
           ^^^^^

Explanation:
  "Apfel" is masculine → accusative → "einen"
```

## LLM backend

- Model: `llama-3.3-70b`
- Provider: hypereal.tech
- Integration point: `checker::check()` in `src/checker.rs`
- Currently returns `None` (no errors) — API key and HTTP call pending
