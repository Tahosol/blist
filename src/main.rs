use chrono::Utc;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;

fn merge(strings: &[String]) -> String {
    let utc = format!("! Last modified: {}", Utc::now().to_string());

    let mut final_merge: Vec<String> = vec![
        "! Blocklist: Blist".to_string(),
        utc,
        "! More info: https://github.com/musdx/blist".to_string(),
    ];

    let mut set: HashSet<String> = HashSet::new();
    let mut merged_lines: Vec<String> = Vec::new();

    for string in strings {
        let lines: Vec<&str> = string.lines().collect();
        for line in lines {
            if line.starts_with("0.0.0.0") || line.starts_with("127.0.0.1") {
                let more_line = format!("||{}^", line.split_whitespace().nth(1).unwrap());
                if !set.contains(&more_line) {
                    set.insert(more_line.clone());
                    merged_lines.push(more_line);
                }
            } else if line.trim().starts_with('!')
                || line.trim().starts_with('#')
                || line.trim().starts_with('[')
            {
                // println!("removed comment");
            } else if !set.contains(line) && (line.starts_with("||") || line.starts_with("@@")) {
                set.insert(line.to_string());
                merged_lines.push(line.to_string());
            } else if line.contains("*") && !set.contains(line) {
                set.insert(line.to_string());
                merged_lines.push(line.to_string());
            } else if !line.trim().contains("||") && !line.trim().contains("^") {
                let good_line = format!("||{}^", line.trim());
                if !set.contains(&good_line) {
                    set.insert(good_line.to_string());
                    merged_lines.push(good_line.to_string());
                }
            } else if !set.contains(line) {
                //for now I am not sure about the fonction of some line
                set.insert(line.to_string());
                merged_lines.push(line.to_string());
            }
        }
    }

    // merged_lines.retain(|line| !line.trim().starts_with('!') && !line.trim().starts_with('#'));
    final_merge.extend(merged_lines);

    final_merge.join("\n")
}

use reqwest::Client;
use tokio::task::JoinHandle;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
    let mut file = File::create("blocklist.txt")?;
    let urls_list = read_urls("list.txt")?;
    let mut content = vec![];

    let client = Client::new();

    let handles: Vec<JoinHandle<Result<String, reqwest::Error>>> = urls_list
        .into_iter()
        .map(|url| {
            let client = client.clone();
            tokio::spawn(async move { fetch_url(&client, &url).await })
        })
        .collect();

    for handle in handles {
        match handle.await {
            Ok(Ok(text)) => content.push(text),
            Ok(Err(e)) => eprintln!("Error fetching url: {:?}", e),
            Err(e) => eprintln!("Join error: {:?}", e),
        }
    }

    let blocklist = merge(&content);

    file.write_all(blocklist.as_bytes())?;
    let end = time.elapsed();
    println!("Done after {:?}", end);

    Ok(())
}

async fn fetch_url(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let res = client.get(url).send().await?;
    let content = res.text().await?;
    Ok(content)
}

fn read_urls(file_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let urls: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    Ok(urls)
}
