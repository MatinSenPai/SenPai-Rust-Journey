use sq_02_telegram_quiz_bot_solution::{format_question, question_bank, QuizSession};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tokio::sync::Mutex;

type Sessions = Arc<Mutex<HashMap<ChatId, QuizSession>>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Quiz bot commands:")]
enum Command {
    #[command(description = "start a new quiz")]
    Quiz,
    #[command(description = "show your current score")]
    Score,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let bot = Bot::from_env();
    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(Update::filter_message().endpoint(handle_answer));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![sessions])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    sessions: Sessions,
) -> ResponseResult<()> {
    match cmd {
        Command::Quiz => {
            let session = QuizSession::new(question_bank());
            let text = match session.current_question() {
                Some(q) => format_question(session.current_question_number(), q),
                None => "No questions available.".to_string(),
            };
            sessions.lock().await.insert(msg.chat.id, session);
            bot.send_message(msg.chat.id, text).await?;
        }
        Command::Score => {
            let sessions = sessions.lock().await;
            let text = match sessions.get(&msg.chat.id) {
                Some(session) => session.score_summary(),
                None => "No quiz in progress. Send /quiz to start one!".to_string(),
            };
            bot.send_message(msg.chat.id, text).await?;
        }
    }
    Ok(())
}

async fn handle_answer(bot: Bot, msg: Message, sessions: Sessions) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };
    let Ok(answer) = text.trim().parse::<usize>() else {
        return Ok(());
    };

    let mut sessions = sessions.lock().await;
    let Some(session) = sessions.get_mut(&msg.chat.id) else {
        return Ok(());
    };
    if session.is_finished() {
        return Ok(());
    }

    let was_correct = session.answer(answer);
    let feedback = if was_correct {
        "Correct!"
    } else {
        "Not quite."
    };

    let reply = if session.is_finished() {
        format!("{feedback} {}", session.score_summary())
    } else {
        let next = session
            .current_question()
            .expect("just checked not finished");
        format!(
            "{feedback}\n\n{}",
            format_question(session.current_question_number(), next)
        )
    };

    bot.send_message(msg.chat.id, reply).await?;
    Ok(())
}
