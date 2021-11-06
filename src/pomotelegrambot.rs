use std::env;
use teloxide::prelude::*;

/*
    TODO:
    * ask if they want a telegram bot
    * somehow make it easier to get the chat id
    * maybe even make a helper to get the chat_id
    * refactor so that main runs in a tokio runtime thread
*/

/*
    HOW TO SETUP THE BOT
    1. create bot with https://t.me/botfather
    2. add your TOKEN: to .env file
    ---
    // .env
    TELOXIDE_TOKEN=<TOKEN>
    CHAT_ID=<CHAT_ID>
    ---
    3. start chat and send a random msg
    4. grab the chat_id from
        https://api.telegram.org/bot<TOKEN>/getUpdates
        // "id":<ID>
    5. your env should look like
    ---
    // .env
    TELOXIDE_TOKEN=<TOKEN>
    CHAT_ID=<CHAT_ID>
    ---
*/

/*
    TODO: this is clogging heavyly
    since this creates a tokio main runtime
    each time this gets called
*/
#[tokio::main]
pub async fn send_message_telegram(message: &str) {
    run(message).await;
}
async fn run(message: &str) {
    teloxide::enable_logging!();
    let bot = Bot::from_env();
    let chat_id: String = dotenv::var("CHAT_ID").unwrap();
    bot.send_message(chat_id, message)
        .send()
        .await
        // TODO: handle the error wayyy better than this
        .expect("message telegram failed unexpectedly");
}
