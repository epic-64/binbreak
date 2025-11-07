## Play
Download the release for your platform, see [Releases](https://github.com/epic-64/binbreak/releases).  
There is one file for linux and one for windows (.exe).

## Linux
- download the file `binbreak-linux`
- open a terminal and navigate to the folder where you downloaded it, e.g. `cd ~/Downloads`
- make it executable: `chmod +x binbreak-linux`
- run the game: `./binbreak-linux`


Instead of relying on prebuilt binaries, you can also build from source.

## Build/Run from source
- download the source code
- make sure you have Rust and Cargo installed, see [rustup.rs](https://rustup.rs/)
- open a terminal and navigate to the folder where you downloaded the source code, e.g. `cd ~/Downloads/binbreak`
- build the project: `cargo build --release`

## Run
```bash
cargo run --release
```

## Test
```bash
cargo test
```

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)