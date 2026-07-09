use clap::{Parser, Subcommand};
use taskforge_cli::format::{format_job_line, format_job_list};
use taskforge_cli::ApiClient;

#[derive(Parser)]
#[command(name = "taskforge", about = "CLI client for taskforge-api")]
struct Cli {
    #[arg(
        long,
        env = "TASKFORGE_API_URL",
        default_value = "http://localhost:8080"
    )]
    api_url: String,

    #[arg(long, env = "TASKFORGE_API_TOKEN")]
    token: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enqueue a new job.
    Enqueue {
        job_type: String,
        /// JSON payload, e.g. '{"to": "a@b.com"}'. Defaults to `{}`.
        #[arg(long)]
        payload: Option<String>,
        #[arg(long)]
        max_attempts: Option<u32>,
    },
    /// Fetch a single job by id.
    Get { id: String },
    /// List jobs, optionally filtered by type.
    List {
        #[arg(long)]
        job_type: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Cancel a pending or retrying job.
    Cancel { id: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.api_url, cli.token);

    let result = match cli.command {
        Commands::Enqueue {
            job_type,
            payload,
            max_attempts,
        } => run_enqueue(&client, &job_type, payload, max_attempts).await,
        Commands::Get { id } => run_get(&client, &id).await,
        Commands::List { job_type, limit } => run_list(&client, job_type, limit).await,
        Commands::Cancel { id } => run_cancel(&client, &id).await,
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

async fn run_enqueue(
    client: &ApiClient,
    job_type: &str,
    payload: Option<String>,
    max_attempts: Option<u32>,
) -> Result<(), String> {
    let payload_value: serde_json::Value = match payload {
        Some(raw) => {
            serde_json::from_str(&raw).map_err(|e| format!("invalid --payload JSON: {e}"))?
        }
        None => serde_json::json!({}),
    };
    let job = client
        .enqueue_job(job_type, payload_value, max_attempts)
        .await?;
    println!("{}", format_job_line(&job));
    Ok(())
}

async fn run_get(client: &ApiClient, id: &str) -> Result<(), String> {
    let job = client.get_job(id).await?;
    println!("{}", format_job_line(&job));
    Ok(())
}

async fn run_list(
    client: &ApiClient,
    job_type: Option<String>,
    limit: usize,
) -> Result<(), String> {
    let jobs = client.list_jobs(job_type.as_deref(), limit).await?;
    println!("{}", format_job_list(&jobs));
    Ok(())
}

async fn run_cancel(client: &ApiClient, id: &str) -> Result<(), String> {
    client.cancel_job(id).await?;
    println!("Cancelled job {id}.");
    Ok(())
}
