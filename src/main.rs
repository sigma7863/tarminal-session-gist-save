use std::{collections::HashMap, fs, io::Read, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(
    name = "tarminal-session-gist-save",
    version,
    about = "Save terminal session logs to GitHub Gist"
)]
struct Cli {
    /// Path to a log file. If omitted, stdin is used.
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Gist filename shown on GitHub.
    #[arg(short = 'n', long, default_value = "terminal-session.log")]
    name: String,

    /// Optional gist description.
    #[arg(short, long, default_value = "Saved by tarminal-session-gist-save")]
    description: String,

    /// Publish gist publicly. Omit for secret gist.
    #[arg(long, default_value_t = false)]
    public: bool,

    /// GitHub API token. Defaults to $GITHUB_TOKEN.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct GistRequest {
    description: String,
    public: bool,
    files: HashMap<String, GistFileContent>,
}

#[derive(Debug, Serialize)]
struct GistFileContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GistResponse {
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let token = cli
        .token
        .context("GitHub token is required. Set --token or environment variable GITHUB_TOKEN.")?;

    let content = read_input(cli.file.as_ref())?;
    if content.trim().is_empty() {
        bail!("Input is empty. Provide a file via --file or pipe session logs via stdin.");
    }

    let gist_url = create_gist(&token, &cli.name, &cli.description, cli.public, &content).await?;

    println!("{}", gist_url);
    Ok(())
}

fn read_input(file: Option<&PathBuf>) -> Result<String> {
    match file {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("Failed to read log file: {}", path.display())),
        None => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .context("Failed to read from stdin")?;
            Ok(input)
        }
    }
}

async fn create_gist(
    token: &str,
    filename: &str,
    description: &str,
    is_public: bool,
    content: &str,
) -> Result<String> {
    let mut files = HashMap::new();
    files.insert(
        filename.to_string(),
        GistFileContent {
            content: content.to_string(),
        },
    );

    let payload = GistRequest {
        description: description.to_string(),
        public: is_public,
        files,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("tarminal-session-gist-save"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).context("Invalid token header value")?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .post("https://api.github.com/gists")
        .json(&payload)
        .send()
        .await
        .context("Failed to call GitHub Gist API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read error body>".to_string());
        bail!("GitHub API error ({status}): {body}");
    }

    let gist: GistResponse = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    Ok(gist.html_url)
}
