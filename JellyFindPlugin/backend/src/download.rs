use reqwest::{Client, Response, multipart};
use std::sync::LazyLock;
use anyhow::bail;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};

const QB_API_URL: &'static str = "http://127.0.0.1:8585/api/v2";
static QB_API_LOGIN_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/auth/login").into_boxed_str());
static QB_API_ADD_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/torrents/add").into_boxed_str());
const USERNAME: &'static str = "admin";
const PASSWORD: &'static str = "adminadmin";

pub async fn download(url: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://www1.cpasbien.to{url}");
    let mut magnet = String::new();
    let client = Client::builder().build()?;
    let res: Response = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Safari/605.1.15")
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let doc = Html::parse_document(&body);
        let selector = Selector::parse("a.download_magnet").unwrap();

        for element in doc.select(&selector) {
            magnet = element.value().attr("href").unwrap_or("").parse().unwrap();
        }
    }

    Ok(magnet)
}

async fn login_to_qbittorrent() -> anyhow::Result<Client> {
    let client = Client::builder().cookie_store(true).build()?;
    let payload = multipart::Form::new()
        .text("username", USERNAME)
        .text("password", PASSWORD);
    let resp = client
        .post(QB_API_LOGIN_URL.as_ref())
        .multipart(payload)
        .send()
        .await?;
    match resp.error_for_status() {
        Err(e) => bail!("Échec de la connexion à qBittorrent ({e:?})."),
        Ok(_resp) => {
            println!("[+] Connexion réussie à qBittorrent.");
            Ok(client)
        }
    }
}

async fn add_magnet_to_qbittorrent(
    session: &Client,
    magnet_link: &str,
    save_path: &str,
) -> anyhow::Result<()> {
    let params = [
        ("urls", magnet_link),
        ("savepath", save_path),
        ("autoTMM", "false"),
    ];

    let resp = session
        .post(QB_API_ADD_URL.as_ref())
        .form(&params)
        .send()
        .await?;

    if resp.status().is_success() {
        println!("[+] Magnet ajouté : {magnet_link} vers {save_path}");
    } else {
        println!("[-] Échec de l'ajout du magnet : {magnet_link}");
        println!("Réponse : {}", resp.text().await?);
    }

    Ok(())
}

pub async fn send_to_qbittorrent(magnet: &str) -> Result<(), Box<dyn std::error::Error>> {
    let session = login_to_qbittorrent().await?;
    add_magnet_to_qbittorrent(&session, magnet, "/Users/anna/Movies/Films").await?;


    Ok(())
}
