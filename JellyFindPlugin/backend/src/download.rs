
use reqwest::{Client, multipart};
use std::sync::LazyLock;
use anyhow::bail;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};
use crate::api::DownloadResult;
use crate::search::build_headers;

const QB_API_URL: &'static str = "http://127.0.0.1:8585/api/v2";
static QB_API_LOGIN_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/auth/login").into_boxed_str());
static QB_API_ADD_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/torrents/add").into_boxed_str());

const USERNAME: &'static str = "admin";
const PASSWORD: &'static str = "adminadmin";


pub async fn download(url: &str) -> Result<DownloadResult, reqwest::Error> {
    let url = format!("https://www1.cpasbien.to{url}");

    let client = Client::builder().build()?;
    let res = client
        .get(&url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Safari/605.1.15")
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.text().await?;
        let doc = Html::parse_document(&body);
        let selector = Selector::parse("a.download_magnet").unwrap();

        if let Some(element) = doc.select(&selector).next() {
            let magnet = element.value().attr("href").unwrap_or("").to_string();
            return Ok(DownloadResult::Magnet(magnet));
        } else {
            eprintln!("Magnet not found in cpasbien page");
        }
    } else {
        eprintln!("Failed to fetch the URL: {}", url);
    }

    Ok(DownloadResult::Magnet(String::new()))
}


pub async fn download_ygg(url: &str) -> Result<DownloadResult, reqwest::Error> {
    let client = Client::builder().build()?;
    let res = client
        .get(url)
        .headers(build_headers())
        .send()
        .await?;

    let bytes = res.bytes().await?;
    Ok(DownloadResult::Torrent(bytes))
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

pub async fn add_torrent_to_qbittorrent(
    session: &Client,
    torrent_content: &[u8],
    save_path: &str,
) -> anyhow::Result<()> {

    let form = multipart::Form::new()
        .part("torrents", multipart::Part::bytes(torrent_content.to_vec())
            .file_name("The.Machine.2023.MULTi.HDR.2160p.WEB-DL.x265-Slay3R.mkv.torrent")
            .mime_str("application/x-bittorrent")?)
        .text("savepath", save_path.to_string())
        .text("autoTMM", "false");



    let resp = session
        .post(QB_API_ADD_URL.as_ref())
        .multipart(form)
        .send()
        .await?;

    if resp.status().is_success() {
        println!("[+] Torrent ajouté vers {}", save_path);
    } else {
        println!("[-] Échec de l'ajout du torrent ");
        println!("Réponse : {}", resp.text().await?);
    }

    Ok(())
}

pub async fn send_to_qbittorrent(magnet: Option<&str>, direct: Option<bytes::Bytes>) -> Result<(), Box<dyn std::error::Error>> {
    let session = login_to_qbittorrent().await?;
    
    if let Some(magnet_link) = magnet {
        add_magnet_to_qbittorrent(&session, magnet_link, "/Users/anna/Movies/Films").await?;
    }
    
    if let Some(direct) = direct {
        add_torrent_to_qbittorrent(&session, &*direct, "/Users/anna/Movies/Films").await?;
    }
    
    


    Ok(())
}
