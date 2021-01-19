# Pomo

## TODO:

[] tui-rs implementation of the ui
[] add sms notifcations for when the break is over
[] configuration in form of a configuration file toml
[] add clap functionality for making it a real cli tool
[] This is a way to generically define that this function might return nothing, or any Error (that extends error::Error).
However, now that this function would return errors and we don't handle them, it means that main can panic.
Maybe it is a good time to implement the process::exit wrapper trick (with match to return 0 of fine, 1 if not fine)
[] documentation of the whole shit
[] publish
[] cash in

## Configuration

for a username of alice, you would find the configuration file here

```
    // Linux:   /home/alice/.config/pomo
    // Windows: C:\Users\Alice\AppData\Roaming\pomo
    // macOS:   /Users/Alice/Library/Application Support/com.pomo
```
