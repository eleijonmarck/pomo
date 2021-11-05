// use serde::{Deserialize, Serialize};
// use std::fmt;
// use teloxide::prelude::*;

// #[derive(Default, Debug, Serialize, Deserialize)]
// struct MyConfig {
//     token: String,
// }

// impl fmt::Display for MyConfig {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.token)
//     }
// }

// /* goto:
// // https://t.me/botfather
//     create a bot
//     take the token
//     Create file notify-tg.toml
//     in your "config" directory (on Linux: $HOME/.config/)
//     put the token in notify-tg.toml
// // telegrambot
// // input -> TOKEN
// */
// //  expected outcomes:
// /*
//     receive message from pomo
// */
// // TODO: remove me if send_message_telegram works
// // pub async fn easy_send() {
// //     let bot = Bot::new("TOKEN");
// //     // Note: it's recommended to `Requester` instead of creating requests directly
// //     let method = GetMe::new();
// //     let request = JsonRequest::new(bot, method);
// //     let _: _ = request.send().await.unwrap();
// // }

// // pub fn send_message_telegram(message: Option<&str>) {
// pub async fn send_message_telegram() {
//     let bot = Bot::from_env();
//     let rt = tokio::runtime::Builder::new_current_thread()
//         .enable_io()
//         .enable_time()
//         .build()
//         .expect("Error building tokio::runtime::Runtime");

//     assert!(res.is_ok() && !res.is_err());
//     //         let message = prefix
//     //             .map_or_else(|| String::with_capacity(message.len()), str::to_owned)
//     //             + &message;
//     //         bot.send_message(message)
//     //             // .parse_mode(ParseMode::Html)
//     //             .send()
//     //             .await
//     //             .map(drop)
//     //     }

//     // }
// }
use futures::future::Future;
use std::env;

use telegrambot::api::BotApi;
use telegrambot::api::SendMessage;
use telegrambot::config::Config;

pub fn config() -> Config {
    let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
    Config::builder(token)
        .proxy("http://127.0.0.1:1081")
        .build()
        .unwrap()
}
use std::io;
use tokio::net::TcpListener;

pub async fn send_message_telegram() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (_, _) = listener.accept().await?;
        let api = BotApi::new(config());

        tokio::spawn(async move {
            // Process each socket concurrently.
            api.send_message(&SendMessage::new(1556709938, format!("Hi")))
                .map(|_| {})
                .map_err(|_| {})
        });
    }
    /*
       tokio::run(futures::lazy(move || {
           tokio::spawn(
               api.get_me()
                   .map(|user| println!("{:?}", user))
                   .map_err(|_| {}),
           )
        }))
    */
}
