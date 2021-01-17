#[macro_use]
extern crate lazy_static;

use std::{
    env,
    error::Error,
    io::{stdout, Cursor, Write}, // The Write trait is needed for stdout
    process,
    time::Duration,
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify_rust::{Notification, NotificationHandle};
use rodio::{OutputStream, OutputStreamHandle};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use constants::{DESCRIPTION, GLYPH_DEFINITIONS, PAUSE_MSG};
use session::{IntoRepresentation, Session, SessionMode};

mod constants;
mod session;

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

#[derive(Debug)]
enum PomoViews {
    Description,
    Timer,
}

#[derive(Debug)]
struct PomoState {
    current_session: Session,
    current_view: PomoViews,
    prev_sessions: i16,
}

impl Default for PomoState {
    fn default() -> PomoState {
        PomoState {
            current_session: Session::new(SessionMode::LongSession),
            current_view: PomoViews::Timer,
            prev_sessions: 0,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // how to use pomodoro, on help or when asking for it
    if env::args().len() > 2 {
        let program = env::args().next().unwrap();
        eprintln!("Usage:");
        eprintln!("  {} start", program);
        eprintln!("  {} config", program);
        eprintln!("  {} stop", program);
        process::exit(2);
    }

    // Going into raw mode
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let notify = make_notifier(&stream_handle);
    // https://notificationsounds.com/notification-sounds/done-for-you-612
    let session_over_sound = include_bytes!("../sounds/done-for-you-612.mp3").as_ref();
    // https://notificationsounds.com/notification-sounds/exquisite-557
    let break_over_sound = include_bytes!("../sounds/exquisite-557.mp3").as_ref();

    let mut state = PomoState::default();

    loop {
        // We need to check for all this before drawing for us to be able to catch a pausing of the application

        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(250))? {
            // It's guaranteed that the `read()` won't block,
            // when the `poll()` function returns `true`.
            // Matching the key
            match read()? {
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
                    code: KeyCode::Char('?'),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    state.current_view = match state.current_view {
                        PomoViews::Timer => PomoViews::Description,
                        PomoViews::Description => PomoViews::Timer,
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    modifiers: KeyModifiers::NONE,
                }) => {
                    state.current_session.toggle_pause();
                }
                _ => {}
            }
        }

        if state.current_session.is_ended() {
            match state.current_session.mode {
                SessionMode::LongSession => {
                    state.prev_sessions += 1;

                    if state.prev_sessions == 3 {
                        state.prev_sessions = 0;
                        state.current_session = Session::new(SessionMode::LongBreak);
                        notify(
                            "3 sessions are over, take a long deserved break!",
                            session_over_sound,
                        );
                    } else {
                        state.current_session = Session::new(SessionMode::ShortBreak);
                        notify("Session is over, take a short break!", session_over_sound);
                    };
                }
                SessionMode::LongBreak => {
                    notify("Long break is over!", break_over_sound);
                    state.current_session = Session::new(SessionMode::LongSession);
                }
                SessionMode::ShortBreak => {
                    notify("Short break is over!", break_over_sound);
                    state.current_session = Session::new(SessionMode::LongSession);
                }
            }
        }

        terminal.draw(|f| match state.current_view {
            PomoViews::Description => {
                // Height + 2 lines for borders
                let height = DESCRIPTION.len() as u16 + 2;
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .horizontal_margin(if f.size().width >= 60 {
                        (f.size().width - 60) / 2
                    } else {
                        0
                    })
                    .vertical_margin(if f.size().height >= height {
                        (f.size().height - height) / 2
                    } else {
                        0
                    })
                    .split(f.size());

                let paragraph = Paragraph::new(
                    DESCRIPTION
                        .iter()
                        .map(|&l| Spans::from(l))
                        .collect::<Vec<_>>(),
                )
                .block(Block::default().borders(Borders::ALL).title("Description"));

                f.render_widget(paragraph, chunks[0]);
            }
            PomoViews::Timer => {
                if state.current_session.is_paused() {
                    let width = PAUSE_MSG.chars().count() as u16;
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)])
                        .horizontal_margin(if f.size().width >= width {
                            (f.size().width - width) / 2
                        } else {
                            0
                        })
                        .vertical_margin((f.size().height - 1) / 2)
                        .split(f.size());

                    let paragraph = Paragraph::new(Span::styled(
                        PAUSE_MSG,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                    .block(Block::default())
                    .alignment(Alignment::Center);

                    f.render_widget(paragraph, chunks[0]);
                } else {
                    let app_box = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(45)].as_ref())
                        .horizontal_margin((f.size().width - 45) / 2)
                        .split(f.size());

                    let timer_areas = Layout::default()
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
                        .horizontal_margin(if f.size().width >= 45 {
                            (f.size().width - 45) / 2
                        } else {
                            0
                        })
                        .vertical_margin(if f.size().height >= 6 {
                            (f.size().height - 6) / 2
                        } else {
                            0
                        })
                        .split(f.size());

                    let time_fmt = state.current_session.remaining().into_representation();
                    let glyph_defs = time_fmt
                        .chars()
                        .map(|c| GLYPH_DEFINITIONS[&c])
                        .collect::<Vec<_>>();

                    for (ix, &glyph_def) in glyph_defs.iter().enumerate() {
                        let glyph_spans = glyph_def
                            .iter()
                            .map(|&l| Spans::from(l))
                            .collect::<Vec<_>>();
                        let paragraph = Paragraph::new(glyph_spans)
                            .block(Block::default())
                            .alignment(Alignment::Center);

                        f.render_widget(paragraph, chunks[ix]);
                    }

                    let session_text_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Max(15)].as_ref())
                        .vertical_margin((f.size().height - 10) / 2)
                        .split(app_box[0]);
                    let block = Block::default();
                    let paragraph = Paragraph::new(current_session.mode.to_string())
                        .block(block)
                        .alignment(Alignment::Center)
                        .style(Style::default().add_modifier(Modifier::ITALIC))
                        .wrap(Wrap { trim: true });
                    f.render_widget(paragraph, session_text_area[0])
                }
            }
        })?;
    }

    Ok(())
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
        let sound_cursor = Cursor::new(sound_file);
        if let Ok(sink) = stream_handle.play_once(sound_cursor) {
            sink.detach();
        };

        notification.expect("Failed to notify!")
    }
}
