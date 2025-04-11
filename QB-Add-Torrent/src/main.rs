use anyhow::bail;
use reqwest::{multipart, Client};
use std::{collections::HashMap, sync::LazyLock};
use tokio::fs;

const QB_API_URL: &'static str = "http://127.0.0.1:8080/api/v2";
static QB_API_LOGIN_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/auth/login").into_boxed_str());
static QB_API_ADD_URL: LazyLock<Box<str>> =
    LazyLock::new(|| format!("{QB_API_URL}/torrents/add").into_boxed_str());
const USERNAME: &'static str = "admin";
const PASSWORD: &'static str = "adminadmin";
const JSON_FILE_PATH: &'static str = "/Users/anna/RustroverProjects/torrent_linker/anime.json";

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

async fn import_torrents_from_json() -> anyhow::Result<()> {
    let file_content = fs::read_to_string(JSON_FILE_PATH).await?;
    let json: HashMap<&str, &str> = serde_json::from_str(&file_content)?;
    let session = login_to_qbittorrent().await?;

    if json.is_empty() {
        bail!("Aucun torrent trouvé dans le fichier JSON.");
    }

    for (torrent_path, save_path) in json {
        add_torrent_to_qbittorrent(&session, torrent_path, save_path).await?;
    }

    Ok(())
}

async fn add_torrent_to_qbittorrent(
    session: &Client,
    torrent_path: &str,
    save_path: &str,
) -> anyhow::Result<()> {
    let multipart = multipart::Form::new()
        .file("torrents", torrent_path)
        .await?
        .text("savepath", save_path.to_string())
        .text("autoTMM", "false");
    let resp = session
        .post(QB_API_ADD_URL.as_ref())
        .multipart(multipart)
        .send()
        .await?;
    if resp.status().is_success() {
        println!("[+] Torrent ajouté : {torrent_path} vers {save_path}");
    } else {
        println!("[-] Échec de l'ajout du torrent : {torrent_path}");
        println!("Réponse : {}", resp.text().await?);
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match import_torrents_from_json().await {
        Err(e) => println!("[-] {e:?}"),
        _ => {}
    }
}
