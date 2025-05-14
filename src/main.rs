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
            } else if !set.contains(line) && (line.starts_with("||") || line.starts_with("@@")) {
                set.insert(line.to_string());
                merged_lines.push(line.to_string());
            } else if !set.contains(line) {
                //for now I am not sure about the fonction of some line
                set.insert(line.to_string());
                merged_lines.push(line.to_string());
            }
        }
    }

    merged_lines.retain(|line| !line.starts_with('!') && !line.starts_with('#'));
    final_merge.extend(merged_lines);

    final_merge.join("\n")
}

fn main() -> Result<(), Box<dyn Error>> {
    let time = Instant::now();
    let mut file = File::create("blocklist.txt")?;
    let urls_list = read_urls("list.txt")?;
    let mut content = vec![];

    for url in urls_list {
        let response: String = ureq::get(url).call()?.body_mut().read_to_string()?;
        content.push(response);
    }

    let blocklist = merge(&content);

    file.write_all(blocklist.as_bytes())?;
    let end = time.elapsed();
    println!("Done after {:?}", end);

    Ok(())
}

fn read_urls(file_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let urls: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    Ok(urls)
}
