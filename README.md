# Executive Decision Maker

> A modern Rust reimagining of the vintage Radio Shack Executive Decision Maker. Ask a yes/no question, press a key, and let the glowing indicators guide your choice.

## Highlights
- 🎛️ Terminal UI powered by [`ratatui`](https://github.com/ratatui-org/ratatui) with crisp layouts that resize gracefully.
- ✨ Light-show animation that shuffles through all six answers before revealing the final verdict.
- 🆘 Built-in help overlay (`Ctrl+H`) so new users can learn the controls without leaving the app.
- 🧹 Robust terminal teardown that restores your shell even after errors or interrupts.

## Controls

| Key / Combo         | Action                                        |
| ------------------- | --------------------------------------------- |
| `Enter` or `Space`  | Start the animated selection (or dismiss help) |
| `Ctrl+H`            | Toggle the in-app help overlay                |
| `q` or `Esc`        | Exit the app (Esc closes help first)          |
| `Ctrl+C`            | Emergency quit                                |

## Getting Started

### Prerequisites
- Rust 1.70+ (2021 edition) with `cargo`
- A terminal that supports ANSI escape sequences (most Unix-like shells and Windows Terminal do)

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run
```

The app launches in the terminal’s alternate screen. Think of your question and press `Enter` or `Space` to watch the answer lights dance before landing on a final choice.

### Binary (optional)
After a release build the optimized binary lives at `target/release/executive-decision-maker`.

## Development Tips
- Prefer running the app in a real TTY (e.g., `cargo run` from a shell) so keyboard events behave as expected.
- Press `Ctrl+C` if you ever need to force the app to exit; the terminal will restore automatically.
- The main logic lives in [`src/main.rs`](src/main.rs); unit tests aren’t necessary yet because the logic is event-loop driven, but integration hooks can be added in the future.

## License

Licensed under the [MIT License](LICENSE) © 2025 Frank Stallion.
