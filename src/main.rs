use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, self};
use std::path::{Path, PathBuf};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde_json::json;
extern crate walkdir;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> io::Result<()> {
    let folder = "/Users/houzi/home/rust/rust-srt/srt";
    let files = walkdir::WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "srt")
                && e.path().file_stem().unwrap_or_default().to_str().map_or(false, |stem| stem.ends_with("en"))
        })
        .map(|e| e.path().to_owned())
        .filter(|e| e.extension().map_or(false, |ext| ext == "srt"));

    for file in files {
        println!("{}", file.display());
        let file_name = file.file_name().unwrap_or_default().to_str().unwrap();
        let path = PathBuf::from(&file);
        let file = File::open(&path)?;
        let reader = io::BufReader::new(file);

        let output_filename = format!("{}_zh.srt", file_name);
        let output_path = Path::new(&output_filename);
        let mut output_file = File::create(&output_path)?;

        for line in reader.lines() {
            let line = line.unwrap();
            if line.chars().next().map_or(false, |c| c.is_alphabetic()) {
                let translated_line = translate_to_chinese(&line).await.unwrap().replace("\n\n", "");
                writeln!(output_file, "{}", translated_line)?;
            } else {
                writeln!(output_file, "{}", line)?;
            }
        }
    }
    Ok(())
}

async fn translate_to_chinese(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_url = "https://api.openai.com/v1/completions";
    let api_key = env::var("OPENAI_API_KEY")?; // 从环境变量中获取API密钥
    let model = "text-davinci-003";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    );

    let client = reqwest::Client::new();
    let payload = json!({
        "model": model,
        "prompt": format!("翻译中文: {}", text),
        "max_tokens": 1024,
        "temperature": 0,
        "top_p": 1,
        "frequency_penalty": 0,
        "presence_penalty": 0
    });

    let response = client
        .post(api_url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    let translation = response_json["choices"][0]["text"]
        .as_str()
        .unwrap_or("Translation failed")
        .to_string();
    let translated_text_only = translation
        .split('\n')
        .skip(1) // 跳过原始文本（第一行）
        .collect::<Vec<&str>>()
        .join(""); // 将剩余的翻译文本连接起来
    Ok(translated_text_only)
}

