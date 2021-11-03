use serde::Deserialize;
use std::{fs::File, io::Read, path::PathBuf};

use dirs::config_dir;

use teloxide_core::Bot;

#[derive(Deserialize)]
struct Config<'a> {
    token: &'a str,
}

fn default_cfg_path() -> PathBuf {
    let mut dir = config_dir()
        .expect("Error obtaining config directory. Specify path to config file with '-c'");
    dir.push("notify-tg.toml");
    dir
}

/* goto:
// https://t.me/botfather
    create a bot
    take the token
    Create file notify-tg.toml
    in your "config" directory (on Linux: $HOME/.config/)
    put the token in notify-tg.toml
// telegrambot
// input -> TOKEN
*/
//  expected outcomes:
/*
    receive message from pomo
*/
fn send(message: &str) {
    let cfg_path: PathBuf = default_cfg_path();
    let mut config_file = File::open(&cfg_path)
        .unwrap_or_else(|err| panic!("Error opening config {:?}: {:?}", cfg_path, err));

    let mut config_raw = String::new();
    config_file
        .read_to_string(&mut config_raw)
        .unwrap_or_else(|err| panic!("Error reading config: {:?}", err));

    let Config { token } =
        toml::from_str(&config_raw).unwrap_or_else(|err| panic!("Error parsing config: {:?}", err));

    let bot = Bot::with_client(
        token,
        None => reqwest::Client::new(),
    );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Error building tokio::runtime::Runtime");

    let res = rt.block_on(async move {
        match (message) {
            (Some(message)) => {
                let message = prefix
                    .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
                    + &message;
                bot.send_message(master_chat_id, message)
                    .parse_mode(ParseMode::Html)
                    .send()
                    .await
                    .map(drop)
            }
            (None) => bot.get_me().send().await.map(|me| {
                log::info!("getMe -> {:#?}", me);
                log::info!("Config is fine. Exiting.");
                log::info!("For help use `notify-tg --help`");
            }),
        }
    });
    if let Err(err) = res {
        log::error!("{:?}", err);
    }
}
