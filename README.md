![sc1.png](docs/sc1.png)

Guess the correct number (from binary to decimal) before time runs out!
![sc3.png](docs/sc3.png)

Or lose a life trying.
![sc4.png](docs/sc4.png)

Includes 16-bit mode as well, when you feel a little bit insane.
![sc6.png](docs/sc6.png)

Includes multiple 4-bit modes, to train individual nibbles.
![sc5.png](docs/sc5.png)

## Colorblind friendly
I discovered usability issues early on while testing on a monochromatic terminal emulator,
and took this as a challenge to make it work well regardless of color perception.
![sc7.png](docs/sc7.png)

## Can you crack the high score?
The longer your streak, the more points you get, but the faster the timer runs out!

High scores are tracked for each game-mode separately, and saved in a text file relative to the executable.

## Play
Download the release for your platform, see [Releases](https://github.com/epic-64/binbreak/releases).  
There is one file for linux and one for windows (.exe).

## Linux
- download the file `binbreak-linux`
- open a terminal and navigate to the folder where you downloaded it, e.g. `cd ~/Downloads`
- make it executable: `chmod +x binbreak-linux`
- run the game: `./binbreak-linux`

## Controls
- use the arrow keys for navigation
- press Enter to confirm choices
- press Esc to exit a game mode or the game. CTRL+C also works to exit the game.

## Retro aesthetics
For the best experience, I recommend using Cool Retro Terminal on Linux, or Windows Terminal in Retro mode.  
But the game should run fine in any terminal.

## Build/Run from source
You be inclined to not run binaries from the internet, and want to build from source instead.

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
