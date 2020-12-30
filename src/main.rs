#[macro_use]
extern crate lazy_static;

use std::{collections::HashMap, env, io, process, time};

use notify_rust::{Notification, NotificationHandle};
use rodio::{OutputStream, OutputStreamHandle};
use rustbox::{self, Color, Event, InitOptions, Key, RustBox};

use sessions::{Session, SessionMode};

mod fonts;
mod sessions;

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

fn main() {
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

    let mut exit_code = 0;
    if let Ok(rb) = RustBox::init(InitOptions::default()) {
        let mut current_session = Session::init(SessionMode::LongSession);
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

        loop {
            let frame_millis = time::Duration::from_millis(16);
            if let Ok(Event::KeyEvent(key)) = rb.peek_event(frame_millis, false) {
                match key {
                    Key::Esc | Key::Ctrl('c') => {
                        exit_code = 1;
                        break;
                    }
                    Key::Char('?') => {
                        options.show_description = !options.show_description;
                    }
                    Key::Char(' ') => {
                        current_session.toggle_pause();
                        // see for space keypress
                        println!("pressed space");
                    }
                    _ => {}
                }
            }

            if current_session.is_paused() {
                continue;
            }

            if current_session.is_ended() {
                match current_session.mode {
                    SessionMode::LongSession => {
                        notify("Pomotime is over!", session_over_sound);
                        number_of_long_sessions += number_of_long_sessions + 1;

                        if number_of_long_sessions == 3 {
                            number_of_long_sessions = 0;
                            current_session = Session::init(SessionMode::LongBreak);
                        } else {
                            current_session = Session::init(SessionMode::ShortBreak);
                        };
                    }
                    SessionMode::LongBreak => {
                        notify("Pomotime is over!", break_over_sound);
                        current_session = Session::init(SessionMode::LongSession);
                    }
                    SessionMode::ShortBreak => {
                        notify("Pomotime is over!", break_over_sound);
                        current_session = Session::init(SessionMode::LongSession);
                    }
                }
            }

            let table = fonts::symbol_table();
            draw(
                &rb,
                current_session.remaining().as_secs(),
                &table,
                &options,
                &current_session.mode,
            );
        }
    }

    process::exit(exit_code);
}

fn draw(
    rb: &RustBox,
    remain: u64,
    table: &HashMap<char, ([&str; 6], usize)>,
    options: &PomoOptions,
    mode: &SessionMode,
) {
    let fmt = remain_to_fmt(remain);
    let symbols = fmt.chars().map(|c| table[&c]);

    let w_sum = symbols.clone().map(|(_, w)| w).fold(0, |sum, w| sum + w);
    let start_x = rb.width() / 2 - w_sum / 2;
    let start_y = rb.height() / 2 - 3;

    rb.clear();

    let mut x = start_x;
    for (symbol, w) in symbols {
        echo(rb, &symbol, x, start_y);
        x += w;
    }

    // show current mode
    let modetext = mode.to_string();
    let start_x = rb.width() / 2 - modetext.len() / 2;
    let start_y = rb.height() / 1 - 5;
    rb.print(
        start_x,
        start_y,
        rustbox::RB_NORMAL,
        Color::Default,
        Color::Default,
        &modetext,
    );

    if options.show_description {
        for (i, d_text) in DESCRIPTION.iter().enumerate() {
            let start_x = rb.width() / 2 - d_text.len() / 2;
            let start_y = rb.height() / 4 - 3;
            rb.print(
                start_x,
                start_y + i,
                rustbox::RB_NORMAL,
                Color::Default,
                Color::Default,
                d_text,
            );
        }
    }

    rb.present();
}

fn echo(rb: &RustBox, symbol: &[&str], start_x: usize, start_y: usize) {
    let mut y = start_y;
    for line in symbol {
        rb.print(
            start_x,
            y,
            rustbox::RB_NORMAL,
            Color::Default,
            Color::Default,
            line,
        );
        y += 1;
    }
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
        if let Ok(sink) = &stream_handle.play_once(sound_cursor) {
            sink.sleep_until_end();
        };

        notification.expect("Failed to notify!")
    }
}
