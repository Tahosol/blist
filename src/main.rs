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

    let mut filter_set: HashSet<String> = HashSet::new();
    let mut sub_domain: Vec<String> = vec![];

    let now = Instant::now();
    for string in strings {
        let lines: Vec<&str> = string.lines().map(|l| l.trim()).collect();
        for line in lines {
            if let Some(url) = clear_url(line) {
                if !url.is_empty() && !has_sub_domain(&url) && !filter_set.contains(&url) {
                    filter_set.insert(url);
                } else if !url.is_empty() {
                    sub_domain.push(url);
                }
            } else if !filter_set.contains(line) {
                filter_set.insert(line.to_string());
            }
        }
    }
    for i in sub_domain {
        let url = get_root_domain(&i);
        if !filter_set.contains(&url) && !filter_set.contains(&i) {
            filter_set.insert(i);
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed in merge: {:.2?}", elapsed);
    for i in filter_set.iter() {
        if i.starts_with("@@") || i.contains("*") || i.contains("/") {
            final_merge.push(i.to_string());
        } else {
            final_merge.push(format!("||{}^", i));
        }
    }

    final_merge.join("\n")
}
fn clear_url(line: &str) -> Option<String> {
    if line.starts_with("0.0.0.0 ") || line.starts_with("127.0.0.1 ") {
        let clean_line = line.replace("0.0.0.0 ", "").replace("127.0.0.1 ", "");
        let clean_line = clean_line.trim();
        return Some(clean_line.to_string());
    } else if line.trim().starts_with('!')
        || line.trim().starts_with('#')
        || line.trim().starts_with('[')
    {
        return Some(String::new());
    } else if line.starts_with("||") {
        let clean_line = line.replace("||", "").replace("^", "");
        return Some(clean_line);
    } else if line.contains("*") || line.starts_with("@@") || line.contains("/") {
        return None;
    }
    Some(line.trim().to_string())
}

fn has_sub_domain(url: &str) -> bool {
    if url != get_root_domain(url) {
        return true;
    }
    false
}
fn get_root_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return domain.to_string();
    }

    let two_part_tlds = [
        "co.uk", "org.uk", "gov.uk", "ac.uk", "com.au", "net.au", "org.au", "co.jp", "co.in",
        "com.br", "com.cn", "com.tw", "com.sg", "com.hk", "com.tr", "com.mx", "com.ru",
    ];

    let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    if two_part_tlds.contains(&last_two.as_str()) && parts.len() >= 3 {
        format!("{}.{}", parts[parts.len() - 3], last_two)
    } else {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    }
}
use reqwest::Client;
use tokio::task::JoinHandle;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
    let mut file = File::create("blocklist.txt")?;
    let urls_list = read_urls("credit.txt")?;
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
