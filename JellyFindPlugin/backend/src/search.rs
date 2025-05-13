use reqwest::{Client, cookie::Jar, Response, Url};
use scraper::{Html, Selector};
use reqwest::header::{USER_AGENT, REFERER, HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING};
use std::sync::Arc;
use reqwest::cookie::CookieStore;
use reqwest::multipart::Form;
use crate::models::Torrent;
use crate::search_ygg::{build_search_query, parse_ygg, Category, Lang, Quality, SubCategory};

pub async fn search_cpasbien(query: &str) -> Result<Vec<Torrent>, Box<dyn std::error::Error>> {
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
                .and_then(|e| Option::from(e.text().collect::<String>()));

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

pub async fn search_ygg(query: &str) -> Result<Vec<Torrent>, Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://www.yggtorrent.top")?;
    let cookie_store = Arc::new(Jar::default());
    let client = build_client(cookie_store.clone())?;

    perform_login(&client, cookie_store.clone()).await?;

    let search_params = build_search_query(
        query,
        Category::Films,
        SubCategory::Films,
        &[Lang::French, Lang::MultiInclusFr],
        &[Quality::HdRip4k, Quality::TvRip4k, Quality::WebDl4k, Quality::WebRip4k],
    );

    let html = search_torrents(&client, &search_params, cookie_store.clone(), &base_url).await?;

    Ok(parse_ygg(&html).await)
}

fn build_client(cookie_store: Arc<Jar>) -> Result<Client, reqwest::Error> {
    Client::builder()
        .cookie_provider(cookie_store)
        .build()
}

pub fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Safari/605.1.15"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.yggtorrent.top/"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
    headers.insert("Priority", HeaderValue::from_static("u=0, i"));
    headers.insert("Cookie", HeaderValue::from_static("ygg_=NHJ1fnGdvoBcJXhZcomtDg1bH6frkNrktbaKFDmRqacHfBKD; x2_promo_details=eyJjb3VudGRvd25fZGF0ZSI6IjA1LzkvMjAyNSAyMzo1OTo1OSIsInRzIjoxNzQ2ODI3OTk5fQ==; account_created=true"));
    headers
}

async fn perform_login(client: &Client, cookie_store: Arc<Jar>) -> Result<(), reqwest::Error> {
    let login_form = Form::new()
        .text("id", "Psyko71")
        .text("pass", "Asterix1928.");

    let response = client
        .post("https://www.yggtorrent.top/auth/process_login")
        .headers(build_headers())
        .multipart(login_form)
        .send()
        .await?;

    println!("Login status: {}", response.status());
    println!("Cookies reçus : {:?}", cookie_store.cookies(&Url::parse("https://www.yggtorrent.top").unwrap()));
    Ok(())
}

async fn search_torrents(client: &Client, params: &Vec<(&str, String)>, cookie_store: Arc<Jar>, base_url: &Url, ) -> Result<String, reqwest::Error> {
    let headers = build_headers();

    let mut res = client
        .get("https://www.yggtorrent.top/engine/search")
        .headers(headers.clone())
        .query(params)
        .send()
        .await?;

    if res.status() == reqwest::StatusCode::FORBIDDEN {
        println!("403 reçu, tentative avec cookies mis à jour...");
        println!("Cookies après 403 : {:?}", cookie_store.cookies(base_url));

        let mut retry_headers = headers.clone();
        retry_headers.remove("Cookie");

        res = client
            .get("https://www.yggtorrent.top/engine/search")
            .headers(retry_headers)
            .query(params)
            .send()
            .await?;
    }

    println!("Search status: {}", res.status());
    res.text().await
}
