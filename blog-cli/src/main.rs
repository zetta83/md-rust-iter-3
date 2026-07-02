use anyhow::{Context, Result};
use blog_client::{BlogApi, BlogClient, BlogClientError};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "blog-cli", about = "Утилита клиент для работы с blog-server", version=env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Использовать gRPC клиент
    #[arg(long, default_value = "false")]
    grpc: bool,
    /// Адрес сервера
    #[arg(long)]
    server: Option<String>,
}

fn token_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".blog_token")
}

fn get_token() -> String {
    fs::read_to_string(token_path()).unwrap_or_default()
}

fn set_token(token: &str) -> Result<()> {
    fs::write(token_path(), token).context("Не удалось сохранить токен")
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Регистрация нового пользователя
    ///
    /// Пример: blog-cli register --username "ivan" --email "ivan@example.com" --password "secret123"
    Register {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
    },
    /// Авторизовать пользователя по username
    ///
    /// Пример: blog-cli login --username "ivan" --password "secret123"
    Login {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    /// Создать пост
    ///
    /// Пример: blog-cli create --title "Мой первый пост" --content "Содержание"
    Create {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        content: String,
    },
    /// Получить пост по id
    ///
    /// Пример: blog-cli get --id 1
    Get {
        #[arg(short, long)]
        id: i64,
    },
    /// Обновить пост
    ///
    /// Пример: blog-cli update --id 1 --title "Новый заголовок" --content "Новое содержание"
    Update {
        #[arg(short, long)]
        id: i64,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        content: Option<String>,
    },
    /// Удалить пост
    ///
    /// Пример: blog-cli delete --id 1
    Delete {
        #[arg(short, long)]
        id: i64,
    },
    /// Показать список постов
    ///
    /// Пример: blog-cli list --limit 20 --offset 0
    List {
        #[arg(short, long, default_value = "10")]
        limit: u32,
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Ошибка: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let server_addr = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            "http://localhost:50051".into()
        } else {
            "http://localhost:8080/api".into()
        }
    });

    let mut client = if cli.grpc {
        BlogClient::grpc(server_addr)
            .await
            .context("Не удалось подключиться к gRPC серверу")?
    } else {
        BlogClient::http(server_addr)
    };

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let user = client
                .register(&username, &email, &password)
                .await
                .context("Ошибка регистрации")?;
            set_token(&client.get_token())?;
            println!("Зарегистрирован: {} (id={})", user.username, user.id);
        }
        Commands::Login { username, password } => {
            client
                .login(&username, &password)
                .await
                .context("Ошибка авторизации")?;
            set_token(&client.get_token())?;
            println!("Вход выполнен: {username}");
        }
        Commands::Create { title, content } => {
            client.set_token(&get_token());
            let post = client
                .create_post(&title, &content)
                .await
                .context("Ошибка создания поста")?;
            println!("Создан пост #{}: {}", post.id, post.title);
        }
        Commands::Get { id } => {
            match client
                .get_post(id)
                .await
                .context("Ошибка получения поста")?
            {
                Some(post) => {
                    println!("#{}: {}", post.id, post.title);
                    println!("{}", post.content);
                }
                None => println!("Пост #{id} не найден"),
            }
        }
        Commands::Update { id, title, content } => {
            client.set_token(&get_token());
            let current = client
                .get_post(id)
                .await
                .context("Ошибка получения поста")?
                .ok_or_else(|| anyhow::anyhow!("Пост #{id} не найден"))?;

            let new_title = title.as_deref().unwrap_or(&current.title);
            let new_content = content.as_deref().unwrap_or(&current.content);

            let post = client
                .update_post(id, new_title, new_content)
                .await
                .context("Ошибка обновления поста")?;
            println!("Обновлён пост #{}: {}", post.id, post.title);
        }
        Commands::Delete { id } => {
            client.set_token(&get_token());
            match client.delete_post(id).await {
                Ok(()) => println!("Пост #{id} удалён"),
                Err(BlogClientError::NotFound(_)) => {
                    anyhow::bail!("Пост #{id} не найден или не принадлежит вам")
                }
                Err(e) => return Err(e).context("Ошибка удаления поста"),
            }
        }
        Commands::List { limit, offset } => {
            let result = client
                .list_posts(Some(limit), Some(offset))
                .await
                .context("Ошибка получения списка постов")?;
            println!(
                "Постов: {} (показано: {})",
                result.total,
                result.posts.len()
            );
            for post in &result.posts {
                println!("  #{}: {}", post.id, post.title);
            }
        }
    }

    Ok(())
}
