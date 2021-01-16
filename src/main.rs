#[macro_use]
extern crate lazy_static;

use std::{env, io, process, time};

use notify_rust::{Notification, NotificationHandle};
use rodio::{OutputStream, OutputStreamHandle};
use sessions::{Session, SessionMode};

mod fonts;
mod sessions;

use crossterm::event::{poll, read, Event};
use crossterm::{
    event::{DisableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    text::Spans,
    widgets::{Block, Paragraph, Wrap},
    Terminal,
};

use std::io::Write;

// <--- bring the trait into scope
/*
inspiration - https://github.com/zenito9970/countdown-rs/blob/master/src/main.rs

todo:
1.
cli
- start/stop: starts from beginning 25:00 -> short break -> short break
- config: sets different config for different situations
- stats: statistics about the different uses of the configs and how long you have worked on them

sound when timer is up

2.
*/

// TODO: https://github.com/clearloop/leetcode-cli/blob/master/src/cli.rs

// How to use the Pomodoro Timer?
// Set estimate pomodoros (1 = 25min of work) for each tasks
// Select a task to work on
// Start timer and focus on the task for 25 minutes
// Take a break for 5 minutes when the alarm ring
// Iterate 3-5 until you finish the tasks",
static DESCRIPTION: [&'static str; 4] = [
    "How to use the Pomodoro Timer?",
    "Focus on the task for 25 minutes",
    "Take a break for 5 minutes when the alarm ring",
    "Iterate 3-5 until you finish the tasks",
];

struct PomoOptions {
    show_description: bool,
}

fn main() -> crossterm::Result<()> {
    // how to use pomodoro, on help or when asking for it
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        let program = env::args().next().unwrap();
        eprintln!("Usage:");
        eprintln!("  {} start", program);
        eprintln!("  {} config", program);
        eprintln!("  {} stop", program);
        process::exit(2);
    }

    //going into raw mode
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_session = Session::new(SessionMode::LongSession);
    let mut number_of_long_sessions = 0;
    let mut options = PomoOptions {
        show_description: false,
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let notify = make_notifier(&stream_handle);
    // https://notificationsounds.com/notification-sounds/done-for-you-612
    let session_over_sound = include_bytes!("../sounds/done-for-you-612.mp3").as_ref();
    // https://notificationsounds.com/notification-sounds/exquisite-557
    let break_over_sound = include_bytes!("../sounds/exquisite-557.mp3").as_ref();

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let boundary_box = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(45)].as_ref())
                .horizontal_margin((f.size().width - 45) / 2)
                .split(f.size());
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Max(10),
                        Constraint::Max(10),
                        Constraint::Max(5),
                        Constraint::Max(10),
                        Constraint::Max(10),
                    ]
                    .as_ref(),
                )
                .vertical_margin((f.size().height - 6) / 2)
                .split(boundary_box[0]);

            let table = fonts::symbol_table();
            let fmt = remain_to_fmt(current_session.remaining().as_secs());
            let symbols: Vec<_> = fmt.chars().map(|c| table[&c].0).collect();

            for (ix, symbol) in symbols.iter().enumerate() {
                let block = Block::default();
                let vec: Vec<_> = symbol.iter().map(|c| Spans::from(*c)).collect();

                let paragraph = Paragraph::new(vec.clone())
                    .block(block)
                    .alignment(Alignment::Center);
                // f.render_widget(paragraph, chunks[ix + 1]);
                f.render_widget(paragraph, chunks[ix]);
            }
        })?;
        // `poll()` waits for an `Event` for a given time period
        if poll(time::Duration::from_millis(100))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            //matching the key
            match read()? {
                //i think this speaks for itself
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    current_session.toggle_pause();
                    // see for space keypress
                }
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }
        if current_session.is_paused() {
            continue;
        }
        if current_session.is_ended() {
            match current_session.mode {
                SessionMode::LongSession => {
                    number_of_long_sessions += number_of_long_sessions + 1;

                    if number_of_long_sessions == 3 {
                        number_of_long_sessions = 0;
                        current_session = Session::new(SessionMode::LongBreak);
                        notify(
                            "3 LongSessions are over, take a long deserved break!",
                            session_over_sound,
                        );
                    } else {
                        current_session = Session::new(SessionMode::ShortBreak);
                        notify("Session is over, take a short break!", session_over_sound);
                    };
                }
                SessionMode::LongBreak => {
                    notify("Break is over!", break_over_sound);
                    current_session = Session::new(SessionMode::LongSession);
                }
                SessionMode::ShortBreak => {
                    notify("Break is over!", break_over_sound);
                    current_session = Session::new(SessionMode::LongSession);
                }
            }
        }
    }
    Ok(())
}

fn remain_to_fmt(remain: u64) -> String {
    let (hours, minutes, seconds) = (remain / 3600, (remain % 3600) / 60, remain % 60);
    if hours == 0 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

fn make_notifier(
    stream_handle: &OutputStreamHandle,
) -> impl Fn(&str, &'static [u8]) -> NotificationHandle + '_ {
    move |message, sound_file| {
        let notification = Notification::new()
            .summary("☝️ Pomotime!")
            .body(message)
            // TODO: should use the image in abstract image
            .icon("firefox")
            .show();

        // we expect &'static because we want the bytes that we read to be available in memory for the lifetime of the program
        let sound_cursor = io::Cursor::new(sound_file);
        if let Ok(sink) = stream_handle.play_once(sound_cursor) {
            sink.detach();
        };

        notification.expect("Failed to notify!")
    }
}
