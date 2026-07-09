use std::sync::Arc;
use taskforge_admin_bot::format::{format_cancel_result, format_status};
use taskforge_admin_bot::ApiClient;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "TaskForge admin commands:")]
enum Command {
    #[command(description = "show recent job status")]
    Status,
    #[command(description = "cancel a job by id")]
    Cancel(String),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Reads the bot token from the TELOXIDE_TOKEN environment variable —
    // get one from @BotFather. Needs real network access and a real token
    // to actually run; that's why this file (main.rs) is separate from
    // the fully-unit-tested lib.rs modules.
    let bot = Bot::from_env();
    let api_base_url =
        std::env::var("TASKFORGE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_token = std::env::var("TASKFORGE_API_TOKEN").expect("TASKFORGE_API_TOKEN must be set");
    let client = Arc::new(ApiClient::new(api_base_url, api_token));

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(answer);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![client])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    client: Arc<ApiClient>,
) -> ResponseResult<()> {
    let text = match cmd {
        Command::Status => match client.list_jobs(10).await {
            Ok(jobs) => format_status(&jobs),
            Err(error) => format!("Error fetching status: {error}"),
        },
        Command::Cancel(id) => {
            let result = client.cancel_job(&id).await;
            format_cancel_result(&id, &result)
        }
    };
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}
