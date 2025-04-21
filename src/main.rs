use chrono::prelude::*;
use reqwest::blocking::get;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};

fn merge(strings: &[&str]) -> String {
    let utc = format!("! Last modified: {}", Utc::now().to_string());

    let mut final_merge: Vec<&str> = vec![
        "! Blocklist: Blist",
        &utc,
        "! More info: https://github.com/musdx/blist",
    ];
    let mut set: HashSet<&str> = HashSet::new();
    let mut merged_lines: Vec<&str> = Vec::new();

    for string in strings {
        let lines: Vec<&str> = string.lines().collect();
        for line in lines {
            if !set.contains(line) {
                set.insert(line);
                merged_lines.push(line);
            }
        }
    }

    merged_lines.retain(|line| !line.starts_with('!'));
    final_merge.extend(merged_lines);

    final_merge.join("\n")
}

fn main() -> io::Result<()> {
    let mut file = File::create("blocklist.txt")?;

    let link_hagezi_pro_pp: &str =
        "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/pro.plus.txt";
    let link_oisd: &str = "https://big.oisd.nl";
    let link_urlhaus: &str =
        "https://malware-filter.gitlab.io/malware-filter/urlhaus-filter-agh.txt";
    let link_adguard_dns_filter: &str =
        "https://adguardteam.github.io/AdGuardSDNSFilter/Filters/filter.txt";
    let link_adaway_sefinek: &str =
        "https://blocklist.sefinek.net/generated/v1/adguard/ads/adaway/hosts.fork.txt";

    let hagezi: String = get(link_hagezi_pro_pp).unwrap().text().unwrap();
    let oisd: String = get(link_oisd).unwrap().text().unwrap();
    let urlhaus: String = get(link_urlhaus).unwrap().text().unwrap();
    let adguard: String = get(link_adguard_dns_filter).unwrap().text().unwrap();
    let adaway: String = get(link_adaway_sefinek).unwrap().text().unwrap();

    let blocklist = merge(&[&hagezi, &oisd, &urlhaus, &adguard, &adaway]);

    file.write_all(blocklist.as_bytes())?;
    println!("done");

    Ok(())
}
