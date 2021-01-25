# Pomo

The first ever built and most performant pomodoro timer for your terminal.

<img src="./images/pomo-logo-transparent.png" alt="drawing" width="250"/>

## Installation

```bash
$ git clone <repo>
$ cargo run start
```

## Motivation

I love to get productive with my pomodoro timers. However once I made the move to linux, I used Pomotroid which took up 25% to 55% of the CPU (XPS-15 2020) and could not find a alternative.

<img src="./images/pomotroid-cpu-usage.png" alt="drawing" width="550"/>

This is the take to make a pomodoro timer the most performant possible and easy to use through your terminal.

- here we can bind the binary data from the audio directly into the published binary
- all values that can be made static and make use of the lifetime features of Rust has been (or please try to find and we will correct it together. :heart:)

## Improvements

If you find that this could be improved please send in a PR and make benchmarks.

## TODO:

- [x] tui-rs implementation of the ui
- [x] add sms notifcations for when the break is over
- [ ] configuration in form of a configuration file toml
- [ ] add clap functionality for making it a real cli tool
- [ ] This is a way to generically define that this function might return nothing, or any Error (that extends error::Error).
  - However, now that this function would return errors and we don't handle them, it means that main can panic.
  - Maybe it is a good time to implement the process::exit wrapper trick (with match to return 0 of fine, 1 if not fine)
- [ ] documentation of the whole shit
- [ ] publish
- [ ] cash in

## Configuration

for a username of alice, you would find the configuration file here

```
    // Linux:   /home/alice/.config/pomo
    // Windows: C:\Users\Alice\AppData\Roaming\pomo
    // macOS:   /Users/Alice/Library/Application Support/com.pomo
```
