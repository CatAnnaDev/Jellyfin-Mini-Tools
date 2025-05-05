use reqwest::{Client, cookie::Jar, Response};
use scraper::{Html, Selector};
use reqwest::header::{USER_AGENT, REFERER};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct Torrent {
    pub title: String,
    pub url: String,
    pub seeders: Option<u32>,
    pub peers: Option<u32>,
    pub size: Option<f32>,
}

pub async fn search_ygg(query: &str) -> Result<Vec<Torrent>, Box<dyn std::error::Error>> {
    let jar = Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(jar.clone())
        .build()?;

    let url = format!("https://www1.cpasbien.to/search_torrent/{}.html", query);
    println!("{url}");
    let res: Response = client
        .post("https://www1.cpasbien.to/search_torrent/")
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Safari/605.1.15")
        .header(REFERER, &url)
        .form(&[("keywords", query)])
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let doc = Html::parse_document(&body);
        let selector = Selector::parse("a.titre").unwrap();

        let mut torrents = Vec::new();

        for element in doc.select(&selector) {
            let link = element.value().attr("href").unwrap_or("");
            let title = element.text().collect::<Vec<_>>().join(" ");
            let seeders_selector = Selector::parse("span.seed_ok").unwrap();
            let peers_selector = Selector::parse("div.down").unwrap();
            let size_selector = Selector::parse("div.poid").unwrap();

            let seeders = doc.select(&seeders_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<u32>().ok());

            let peers = doc.select(&peers_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<u32>().ok());

            let size = doc.select(&size_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().parse::<f32>().ok());

            torrents.push(Torrent {
                title,
                url: link.to_string(),
                seeders,
                peers,
                size,
            });
        }

        Ok(torrents)
    } else {
        eprintln!("Failed to fetch page, status: {}", res.status());
        Err("Failed to fetch torrents".into())
    }
}
