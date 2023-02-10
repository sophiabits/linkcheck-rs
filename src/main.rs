use std::collections::{HashSet, VecDeque};

use clap::Parser;
use lazy_static::lazy_static;
use reqwest;
use scraper::{Html, Selector};

lazy_static! {
    static ref ANCHOR: Selector = make_selector("a");
}

#[derive(Parser)]
struct Cli {
    domain: String,
}

fn get_url(url: &String) -> reqwest::Result<String> {
    let res = reqwest::blocking::get(url)?;
    res.text()
}

fn is_url_external(cli: &Cli, url: &String) -> bool {
    !url.starts_with(&format!("https://{}", cli.domain))
}

fn is_url_scrapeable(url: &String) -> bool {
    !url.starts_with("mailto:") && !url.starts_with("#")
}

fn make_selector(selector: &str) -> Selector {
    Selector::parse(selector).unwrap()
}

fn normalize_url(cli: &Cli, url: &String) -> String {
    if url.starts_with("/") {
        return format!("https://{}{}", cli.domain, url);
    }

    url.clone()
}

fn main() {
    let cli = &Cli::parse();
    let mut errors = Vec::new();

    let mut queue = VecDeque::new();
    queue.push_back(String::from("https://sophiabits.com"));

    let mut visited = HashSet::new();

    while let Some(url) = queue.pop_front() {
        visited.insert(url.clone());
        let url = normalize_url(&cli, &url);

        let res = match get_url(&url) {
            Ok(r) => r,
            Err(_err) => {
                errors.push(url.clone());
                String::from("")
            }
        };

        if is_url_external(&cli, &url) {
            // Only check 1 level deep externally
            continue;
        }

        for href in Html::parse_document(&res)
            .select(&ANCHOR)
            .map(|it| it.value().attr("href"))
            .flatten()
            .map(String::from)
            .filter(|it| !visited.contains(it) && is_url_scrapeable(it))
        {
            queue.push_back(href);
        }
    }

    for error_url in errors {
        println!("ERROR: {}", error_url);
    }
}
