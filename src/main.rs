use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use std::str::FromStr;
use warp::Filter;
use solana_sdk::{
    pubkey::Pubkey, 
    signer::keypair::Keypair, 
    transaction::Transaction,
    system_instruction,
    system_program,
};
use dotenv::dotenv;
use serde_json;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Start Telegram bot
    let bot = Bot::from_env();

    tokio::spawn(phantom_callback());
    Command::repl(bot, answer).await;
}

// Define bot commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These are my commands:")]
enum Command {
    #[command(description = "Help.")]
    Help,
    #[command(description = "Authenticate with Phantom Wallet.")]
    Auth,
    #[command(description = "Sign a test transaction.")]
    Sign,
}

// Define the handler for commands
async fn answer(bot: Bot, msg: Message, cmd: Command) -> teloxide::prelude::ResponseResult<()> {
    match cmd {
        Command::Auth => {
            let user_id = msg.chat.id.to_string();
            let callback_url = format!("https://your-server.com/callback?user_id={}", user_id);

            // Generate Phantom Wallet deep link
            let app_url = "https://your-app.com";
            let app_name = "RustBot";
            let auth_link = format!(
                "https://phantom.app/ul/v1/connect?app_url={}&redirect_url={}&app_name={}",
                app_url, callback_url, app_name
            );

            bot.send_message(msg.chat.id, format!("Authenticate with Phantom: [Click here]({})", auth_link))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }

        Command::Sign => {
            let user_id = msg.chat.id.to_string();
            // Example public key (use a real one from Phantom auth callback)
            let phantom_wallet_pubkey = "your-phantom-wallet-pubkey";

            // Create a test transaction
            let keypair = Keypair::new();
            let recipient = Pubkey::new_unique();
            let lamports = 1_000_000; // 0.001 SOL
            let payer = Pubkey::from_str(phantom_wallet_pubkey).unwrap();

            let instruction = system_instruction::transfer(&payer, &recipient, lamports);
            let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer));

            // Sign transaction link
            let tx_link = format!(
                "https://phantom.app/ul/v1/transaction?app_url=https://your-app.com&app_name=RustBot&payload={}",
                urlencoding::encode(&serde_json::to_string(&transaction).unwrap())
            );

            bot.send_message(msg.chat.id, format!("Sign the transaction: [Click here]({})", tx_link))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    }
    Ok(())
}

// Define the Phantom Wallet callback handler (uses Warp for simplicity)

async fn phantom_callback() {
    let route = warp::get()
        .and(warp::path("callback"))
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(|query: std::collections::HashMap<String, String>| {
            if let Some(public_key) = query.get("public_key") {
                format!("Phantom wallet connected: {}", public_key)
            } else {
                "Failed to connect Phantom wallet.".to_string()
            }
        });

    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}