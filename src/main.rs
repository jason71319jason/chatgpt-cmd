use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::ArgAction;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use clap::{Arg, Command};

static CHATGPT_DIR: &str = ".chatgpt";
static CONFIG_FILE: &str = "config.json";
static HISTORY_FILE: &str = "history.json";

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    index: i32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    id: String,
    object: String,
    created: i64,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct History {
    hint: Message,
    history: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    url: String,
    model: String,
    key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::from("https://api.openai.com/v1/chat/completions"),
            model: String::from("gpt-3.5-turbo"),
            key: String::default(),
        }
    }
}

fn get_chatgpt_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut chatgpt_dir = PathBuf::from(env::var("HOME")?);
    chatgpt_dir.push(CHATGPT_DIR);
    Ok(chatgpt_dir)
}

fn init_chatgpt() -> Result<(), Box<dyn std::error::Error>> {
    let chatgpt_dir = get_chatgpt_dir()?;

    if !chatgpt_dir.exists() {
        fs::create_dir(&chatgpt_dir)?;
    }

    let config_path = chatgpt_dir.join(CONFIG_FILE);
    init_json::<Config>(&config_path)?;

    let history_path = chatgpt_dir.join(HISTORY_FILE);
    init_json::<History>(&history_path)?;

    Ok(())
}

fn init_json<T>(json_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>>
where
    T: Default + Serialize,
{
    if !json_path.exists() {
        let mut file = File::create(json_path)?;
        file.write_all(serde_json::to_string(&T::default())?.as_bytes())?;
    }

    Ok(())
}

fn load_json<T>(json_path: &PathBuf) -> Result<T, Box<dyn std::error::Error>> 
where
    for<'de> T: Deserialize<'de>,
{
    let file = File::open(json_path)?;
    Ok(serde_json::from_reader(file)?) 
}

fn update_json<T>(path: &PathBuf, json: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize
{
    let json = serde_json::to_string_pretty(&json).unwrap();
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

async fn chat(config: &Config, messages: &Vec<Message>) -> Result<Message, Box<dyn std::error::Error>> {
    //"Bearer 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"
    // set up request payload
    let request = Request {
        model: config.model.clone(),
        messages: messages.clone(),
    };

    // set the request headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
            format!("Bearer {}", config.key)
            .parse()
            .unwrap(),
    );

    // send the POST request to the API endpoint
    let client = reqwest::Client::new();
    let mut response: Response = client
        .post(config.url.clone())
        .headers(headers)
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    // print the response data
    let message = response.choices.remove(0).message;
    println!("{}", message.content);

    Ok(message)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("chat")
                    .arg(Arg::new("clean")
                        .short('c')
                        .long("clean")
                        .help("Clean history")
                        .action(ArgAction::SetTrue))
                    .arg(Arg::new("prompt")
                        .action(ArgAction::Set))
                    .arg(Arg::new("hint")
                        .short('H')
                        .long("hint")
                        .help("Hint ChatGPT")
                        .action(ArgAction::Set))
                    .get_matches();

    init_chatgpt()?;

    if matches.get_flag("clean") {
        let history_path = get_chatgpt_dir()?.join(HISTORY_FILE);
        fs::remove_file(&history_path)?;
        init_json::<History>(&history_path)?;
        return Ok(());
    }

    match matches.get_one::<String>("hint") {
        Some(prompt) => {
            let history_path = get_chatgpt_dir()?.join(HISTORY_FILE);
            let mut history: History = load_json(&history_path)?;
            history.hint = Message {
                role: String::from("system"),
                content: prompt.clone(),
            };
            update_json(&history_path, history)?;
        },
        None => ()
    }

    match matches.get_one::<String>("prompt") {
        Some(prompt) => {
            let history_path = get_chatgpt_dir()?.join(HISTORY_FILE);
            let mut history: History = load_json(&history_path)?;
            history.history.push(Message {
                role: String::from("user"),
                content: prompt.clone(),
            });

            let config_path = get_chatgpt_dir()?.join(CONFIG_FILE);
            let config: Config = load_json(&config_path)?;
            let response = chat(&config, &history.history).await?;

            history.history.push(response);
            update_json(&history_path, history)?;
        },
        None => ()
    }

    Ok(())
}
