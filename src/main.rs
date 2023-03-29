use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, self};
use std::path::{Path, PathBuf};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde_json::json;
use std::io::Write;

extern crate walkdir;

use dotenv::{dotenv, Error};
use std::fs;
use tokio::io::BufWriter;

#[tokio::main]
async fn main() -> io::Result<()> {
    // /Volumes/otherdata/udemy/量化-solana/solana-developer
    // Get the command line arguments.
    let args: Vec<String> = env::args().collect();

    // Check if the user provided a folder path as an argument.
    if args.len() < 2 {
        println!("Usage: {} <folder_path>", args[0]);
        return Ok(());
    }
    let path = &args[1];
    // let path = "/Volumes/otherdata/udemy/量化-solana/solana-developer";
    let subdirs = get_subdirs(path).unwrap();
    for subdir in subdirs {
        let mut stringsPath = path.to_string() + "/" + &*subdir.to_string();
        let files = walkdir::WalkDir::new(&stringsPath)
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
            //get content
            let file_name = file.file_name().unwrap_or_default().to_str().unwrap();
            let path = PathBuf::from(&file);
            //获取path
            let file = File::open(&path)?;
            let reader = io::BufReader::new(file);


            //create new file to save
            // file_name 处理文件名
            //let mut srt_output_path = PathBuf::from(&stringsPath);
            let buf = rename_file(&file_name).unwrap();
            let mut output_filename = PathBuf::from(buf);
            //get mother path
            let mut srt_output_path = PathBuf::from(&stringsPath);
            //conbine two path together
            srt_output_path.push(output_filename);
            println!("{:?}", srt_output_path);
            let mut output_path = PathBuf::from(&srt_output_path);
            let mut output_file = File::create(output_path)?;
            //to chinese
            for line in reader.lines() {
                let line = line.unwrap();
                //get folder path
                if line.chars().next().map_or(false, |c| c.is_alphabetic()) {
                    let mut result = translate_to_chinese(&line).await;
                    //handle error contion
                    let getString = match result {
                        Ok(string) => string,
                        Err(error) => {
                            println!("Error: {}", error);
                            continue; // 例如跳过这个循环的当前迭代
                        }
                    };
                    // //get out
                    let translated_line = getString.replace("\n\n", "");
                    //before handle path problem
                    //stringsPath this is file need to save

                    writeln!(output_file, "{}", translated_line);
                } else {
                    writeln!(output_file, "{}", line);
                    //give e reminder about this line
                    println!("{} is done", line);
                }
            }
            //give e reminder
            println!("{} is done", file_name);
        }
    }
    Ok(())
}



fn rename_file(filename: &str) -> Option<PathBuf> {
    let path = PathBuf::from(filename);
    let stem = path.file_stem()?;
    let extension = path.extension()?;
    let new_stem = format!("{}_zh", stem.to_str()?);
    let new_filename = format!("{}.{}", new_stem, extension.to_str()?);
    let new_path = path.with_file_name(new_filename);
    Some(new_path)
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
        "prompt": format!("翻译成中文,尽量通俗易懂: {}", text),
        "max_tokens": 2048,
        "temperature": 0,
        "top_p": 1,
        "frequency_penalty": 0,
        "presence_penalty": 0
    });
    //

    let response = client
        .post(api_url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;
    //handle err condition
    //every time after we request,we  sleep 1 s
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    if !response.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{}", response.text().await?),
        )));
    }
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


fn get_subdirs(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut subdirs = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            subdirs.push(entry.file_name().into_string().unwrap());
        }
    }
    Ok(subdirs)
}

#[cfg(test)]
mod tests {
    use super::translate_to_chinese;
    use std::env;

    #[tokio::test]
    async fn test_translate_to_chinese() {
        // 请确保设置环境变量 "OPENAI_API_KEY" 为你的API密钥
        env::set_var("OPENAI_API_KEY", "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");

        let english_text = "Hello, world!";
        let expected_chinese_text = "你好，世界！"; // 根据API返回的翻译，可能需要进行调整

        let result = translate_to_chinese(english_text).await;

        match result {
            Ok(translated_text) => {
                assert_eq!(translated_text, expected_chinese_text);
            }
            Err(e) => {
                panic!("Error occurred during translation: {}", e);
            }
        }
    }
}
