use actix_web::{get, web, HttpResponse, Responder};
use crate::download::{download, send_to_qbittorrent};
use crate::search::search_ygg;



#[get("/search")]
async fn search(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    if let Some(q) = query.get("q") {
        let results = search_ygg(q).await.unwrap_or_default();
        HttpResponse::Ok().json(results)
    } else {
        HttpResponse::BadRequest().body("Missing query parameter")
    }
}

#[get("/dl")]
async fn dl(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    if let Some(q) = query.get("dl") {
        let result = download(q).await.unwrap_or_default();
        let _ = send_to_qbittorrent(&*result).await;
        HttpResponse::Ok().json(result)
    }
    else {
        HttpResponse::BadRequest().body("Missing query parameter")
    }
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(search).service(dl);
}
