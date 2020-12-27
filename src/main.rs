use notify_rust::Notification;
use rustbox::{self, Color, Event, InitOptions, Key, RustBox};
use std::collections::HashMap;
use std::process::exit;
use std::{env, fmt, time};

mod fonts;
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

fn main() {
  // save pomos
  // how to use pomodoro, on help or when asking ofr it
  let args: Vec<String> = env::args().skip(1).collect();
  if args.len() != 1 {
    let program: String = env::args().next().unwrap();
    eprintln!("Usage:");
    eprintln!("  {} start", program);
    eprintln!("  {} config", program);
    eprintln!("  {} stop", program);
    exit(2);
  }
  //

  let mut exit_code = 0;
  if let Ok(rb) = RustBox::init(InitOptions::default()) {
    let mut current_session = long_session();
    let mut number_of_long_sessions = 0;
    let mut start = time::Instant::now();
    let mut pause_timer: Option<time::Instant> = None;
    let mut options = PomoOptions {
      show_description: false,
    };
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    //https://notificationsounds.com/notification-sounds/done-for-you-612
    let session_over_sound = include_bytes!("../sounds/done-for-you-612.mp3").as_ref();

    //https://notificationsounds.com/notification-sounds/exquisite-557
    let break_over_sound = include_bytes!("../sounds/exquisite-557.mp3").as_ref();

    loop {
      let frame_millis = time::Duration::from_millis(16);
      if let Event::KeyEvent(key) = rb.peek_event(frame_millis, false).unwrap() {
        if key == Key::Esc || key == Key::Ctrl('c') {
          exit_code = 1;
          break;
        }
        if key == Key::Char(' ') {
          if let Some(timer) = pause_timer {
            if let Some(add_timer) = start.checked_add(timer.elapsed()) {
              start = add_timer
            }
            pause_timer = None;
          } else {
            pause_timer = Some(time::Instant::now());
          }
          // see for space keypress
          println!("pressed space");
        }
        if key == Key::Char('?') {
          options.show_description = !options.show_description;
        }
      }
      if pause_timer.is_some() {
        continue;
      }

      // checking for session end
      if current_session.duration < start.elapsed() {
        match current_session.mode {
          SessionMode::LongSession => {
            notify(String::from("Pomotime is over!")).expect("could not notify");
            play_sound_file(&stream_handle, session_over_sound);

            number_of_long_sessions += number_of_long_sessions + 1;
            if number_of_long_sessions == 3 {
              current_session = long_break();
              start = time::Instant::now();
            } else {
              current_session = short_break();
              start = time::Instant::now();
            }
          }
          SessionMode::LongBreak => {
            notify(String::from("Pomotime is over!")).expect("could not notify");
            play_sound_file(&stream_handle, break_over_sound);

            current_session = long_session();
            start = time::Instant::now();
          }
          SessionMode::ShortBreak => {
            notify(String::from("Pomotime is over!")).expect("could not notify");
            play_sound_file(&stream_handle, break_over_sound);
            current_session = long_session();
            start = time::Instant::now();
          }
          _ => {
            println!("else")
          }
        }
      }
      let remain = current_session.duration - start.elapsed();
      let table = fonts::symbol_table();
      draw(
        &rb,
        remain.as_secs(),
        &table,
        &options,
        &current_session.mode,
      );
    }
  }
  exit(exit_code);
}

#[derive(Debug)]
enum SessionMode {
  LongSession,
  ShortBreak,
  LongBreak,
}

struct Session {
  duration: time::Duration,
  mode: SessionMode,
}

impl fmt::Display for SessionMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
    // or, alternatively:
    // fmt::Debug::fmt(self, f)
  }
}

fn long_session() -> Session {
  let h = 0;
  let m = 0;
  let s = 5;
  let s = Session {
    duration: time::Duration::new(3600 * h + 60 * m + s, 0),
    mode: SessionMode::LongSession,
  };
  return s;
}

fn long_break() -> Session {
  let h = 0;
  let m = 15;
  let s = 0;
  let s = Session {
    duration: time::Duration::new(3600 * h + 60 * m + s, 0),
    mode: SessionMode::LongBreak,
  };
  return s;
}

fn short_break() -> Session {
  let h = 0;
  let m = 5;
  let s = 0;
  let s = Session {
    duration: time::Duration::new(3600 * h + 60 * m + s, 0),
    mode: SessionMode::ShortBreak,
  };
  return s;
}

fn play_sound_file(stream_handle: &rodio::OutputStreamHandle, sound_file: &'static [u8]) {
  // we expect &'static because we want the bytes that we read to be available in memory for the lifetime of the program
  let sound_cursor = std::io::Cursor::new(sound_file);
  if let Ok(sink) = stream_handle.play_once(sound_cursor) {
    sink.sleep_until_end()
  };
}

struct PomoOptions {
  show_description: bool,
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
    let description = get_description();
    let mut i = 0;
    for d_text in description.iter() {
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
      i += 1;
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

fn get_description() -> [String; 4] {
  return [
    String::from("How to use the Pomodoro Timer?"),
    String::from("focus on the task for 25 minutes"),
    String::from("Take a break for 5 minutes when the alarm ring"),
    String::from("Iterate 3-5 until you finish the tasks"),
  ];
  // How to use the Pomodoro Timer?
  // Set estimate pomodoros (1 = 25min of work) for each tasks
  // Select a task to work on
  // Start timer and focus on the task for 25 minutes
  // Take a break for 5 minutes when the alarm ring
  // Iterate 3-5 until you finish the tasks",
}

fn notify(body_msg: String) -> Result<notify_rust::NotificationHandle, notify_rust::error::Error> {
  return Notification::new()
    .summary("☝️ Pomotime!")
    .body(&body_msg)
    // TODO: should use the image in abstract image
    .icon("firefox")
    .show();
}
