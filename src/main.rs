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
use num::Integer;
use rodio::{OutputStream, OutputStreamHandle};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use constants::{IntoSpans, DESCRIPTION, GLYPH_DEFINITIONS, PAUSE_MSG};
use session::{IntoRepresentation, Session, SessionMode};

mod constants;
mod session;

/*
inspiration - https://github.com/zenito9970/countdown-rs/blob/master/src/main.rs
*/

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
        // We need to check for events and keystrokes before drawing
        // - for us to be able to catch a pausing of the application or similiar

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
                            "3 long sessions are over, take a deserved long break!",
                            session_over_sound,
                        );
                    } else {
                        state.current_session = Session::new(SessionMode::ShortBreak);
                        notify(
                            "Long session is over, take a short break!",
                            session_over_sound,
                        );
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

        terminal.draw(|f| {
            match state.current_view {
                PomoViews::Description => {
                    // Height + 2 lines for borders
                    let height = DESCRIPTION.len() as u16 + 2;
                    let description_dialog = make_centered_box(f.size(), 60, height);
                    let description_dialog_widget = Paragraph::new(DESCRIPTION.into_spans())
                        .block(Block::default().borders(Borders::ALL).title("Description"));

                    f.render_widget(description_dialog_widget, description_dialog);
                }
                PomoViews::Timer => {
                    let app_box = make_centered_box(f.size(), 45, f.size().height);

                    if state.current_session.is_paused() {
                        let paused_dialog = make_centered_box(app_box, app_box.width, 4);
                        let paused_dialog_widget = Paragraph::new(PAUSE_MSG.into_spans())
                            .block(Block::default().borders(Borders::ALL))
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

                        f.render_widget(paused_dialog_widget, paused_dialog);
                    } else {
                        let session_text_area = make_centered_box(app_box, 15, 10);
                        let session_text_widget =
                            Paragraph::new(state.current_session.mode.to_string())
                                .block(Block::default())
                                .alignment(Alignment::Center)
                                .style(Style::default().add_modifier(Modifier::ITALIC));

                        f.render_widget(session_text_widget, session_text_area);

                        let timer_box = make_centered_box(app_box, 45, 6);
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
                            .split(timer_box);

                        let time_fmt = state.current_session.remaining().into_representation();
                        for (ix, c) in time_fmt.chars().enumerate() {
                            let glyph_widget = Paragraph::new(GLYPH_DEFINITIONS[&c].into_spans())
                                .block(Block::default())
                                .alignment(Alignment::Center);

                            f.render_widget(glyph_widget, timer_areas[ix]);
                        }
                    }
                }
            }
        })?;
    }

    Ok(())
}

fn make_centered_box(rect: Rect, width: u16, height: u16) -> Rect {
    let Rect {
        height: frame_height,
        width: frame_width,
        ..
    } = rect;
    let box_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(width)].as_ref())
        .horizontal_margin(if frame_width >= width {
            (frame_width - width).div_floor(&2)
        } else {
            0
        })
        .vertical_margin(if frame_height >= height {
            (frame_height - height).div_floor(&2)
        } else {
            0
        })
        .split(rect);

    box_areas[0]
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
