use std::fs;
use std::path::Path;

use anyhow::Context;
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};

const TOKEN_FILE: &str = ".blog_token";

#[derive(Parser)]
#[command(name = "blog-cli")]
struct Cli {
    /// Использовать gRPC вместо HTTP.
    #[arg(long)]
    grpc: bool,
    /// Базовый URL HTTP (например http://127.0.0.1:8080) или gRPC endpoint (http://127.0.0.1:50051).
    #[arg(long)]
    server: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    Get {
        #[arg(long)]
        id: i64,
    },
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: i64,
    },
    List {
        #[arg(long, default_value_t = 10)]
        limit: i64,
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },
}

fn default_server(grpc: bool) -> String {
    if grpc {
        "http://127.0.0.1:50051".to_string()
    } else {
        "http://127.0.0.1:8080".to_string()
    }
}

fn load_saved_token() -> Option<String> {
    let path = Path::new(TOKEN_FILE);
    if !path.exists() {
        return None;
    }
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn save_token(token: &str) -> anyhow::Result<()> {
    fs::write(TOKEN_FILE, token).with_context(|| format!("write {TOKEN_FILE}"))
}

fn ensure_token(client: &BlogClient) -> anyhow::Result<()> {
    if client.get_token().is_none() {
        anyhow::bail!("нет JWT: выполните login/register или положите токен в {TOKEN_FILE}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let endpoint = cli.server.unwrap_or_else(|| default_server(cli.grpc));
    let transport = if cli.grpc {
        Transport::Grpc(endpoint)
    } else {
        Transport::Http(endpoint)
    };
    let mut client = BlogClient::new(transport);
    if let Some(tok) = load_saved_token() {
        client.set_token(tok);
    }

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let auth = client.register(&username, &email, &password).await?;
            save_token(&auth.token)?;
            println!("{}", serde_json::to_string_pretty(&auth)?);
        }
        Commands::Login { username, password } => {
            let auth = client.login(&username, &password).await?;
            save_token(&auth.token)?;
            println!("{}", serde_json::to_string_pretty(&auth)?);
        }
        Commands::Create { title, content } => {
            ensure_token(&client)?;
            let post = client.create_post(&title, &content).await?;
            println!("{}", serde_json::to_string_pretty(&post)?);
        }
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("{}", serde_json::to_string_pretty(&post)?);
        }
        Commands::Update { id, title, content } => {
            ensure_token(&client)?;
            let post = client.update_post(id, &title, content.as_deref()).await?;
            println!("{}", serde_json::to_string_pretty(&post)?);
        }
        Commands::Delete { id } => {
            ensure_token(&client)?;
            client.delete_post(id).await?;
            println!("deleted post {id}");
        }
        Commands::List { limit, offset } => {
            let list = client.list_posts(limit, offset).await?;
            println!("{}", serde_json::to_string_pretty(&list)?);
        }
    }

    Ok(())
}
