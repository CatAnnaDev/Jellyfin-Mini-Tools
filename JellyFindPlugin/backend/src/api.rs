use actix_web::{get, web, HttpResponse, Responder};
use crate::download::{download, download_ygg, send_to_qbittorrent};
use crate::search::{search_cpasbien, search_ygg};



#[get("/search")]
async fn search(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    println!("Search CPASBIEN {:?}", query);
    if let Some(q) = query.get("q") {
        let results = search_cpasbien(q).await.unwrap_or_default();
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::BadRequest().body("Missing query parameter")
    }
}

#[get("/searchygg")]
async fn searchygg(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    println!("Search YGG {:?}", query);
    if let Some(q) = query.get("q") {
        let results = search_ygg(q).await.unwrap_or_default();
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::BadRequest().body("Missing query parameter")
    }
}

pub enum DownloadResult {
    Magnet(String),
    Torrent(bytes::Bytes),
}

#[get("/dl")]
async fn dl(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    if let Some(q) = query.get("dl") {
        let result = if q.contains("cpasbien") {
            download(q).await.unwrap_or_else(|_| DownloadResult::Magnet(String::new()))
        } else {
            download_ygg(q).await.unwrap_or_else(|_| DownloadResult::Magnet(String::new()))
        };

        match result {
            DownloadResult::Magnet(magnet_url) => {
                println!("Magnet: {}", magnet_url);
                let _ = send_to_qbittorrent(Some(&magnet_url), None).await;
            }
            DownloadResult::Torrent(torrent_bytes) => {
                println!("Torrent size: {} bytes", torrent_bytes.len());
                let _ = send_to_qbittorrent(None, Some(torrent_bytes)).await;
            }
        }

        HttpResponse::Ok().body("okay")
    }
    else {
        HttpResponse::BadRequest().body("Missing query parameter")
    }
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(search).service(searchygg).service(dl);
}
