use scraper::{CaseSensitivity, Html, Selector};
use regex::Regex;
use crate::models::Torrent;

pub fn extract_id_from_url(url: &str) -> Option<&str> {
    let re = Regex::new(r"/torrent/[^/]+/[^/]+/(\d+)-").unwrap();
    re.captures(url).and_then(|cap| cap.get(1).map(|m| m.as_str()))
}

pub async fn parse_ygg(body: &str) -> Vec<Torrent> {
    let document = Html::parse_document(body);

    let tr_selector = Selector::parse("table.table > tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a#torrent_name").unwrap();

    let mut torrents = vec![];

    for tr in document.select(&tr_selector) {

        if tr.value().has_class("hidden", CaseSensitivity::AsciiCaseInsensitive) {
            continue;
        }

        let tds: Vec<_> = tr.select(&td_selector).collect();
        if tds.len() >= 9 {
            if let Some(name_tag) = tds[1].select(&a_selector).next() {
                let name = name_tag.text().collect::<Vec<_>>().join("").trim().to_string();
                let url = name_tag.value().attr("href").unwrap_or("").to_string();
                let id = extract_id_from_url(&url).unwrap_or_default().to_string();

                torrents.push(Torrent {
                    title: name,
                    url: format!("https://www.yggtorrent.top/engine/download_torrent?id={}", id),
                    seeders: Some(tds[7].text().collect::<Vec<_>>().join("").trim().to_string().parse().unwrap()),
                    size: Some(tds[5].text().collect::<Vec<_>>().join("").trim().to_string()),
                    peers: Some(tds[8].text().collect::<Vec<_>>().join("").trim().to_string().parse().unwrap()),
                });
            }
        }
    }

    torrents
}


#[derive(Debug)]
pub enum Category {
    Films,
}

#[derive(Debug)]
pub enum SubCategory {
    Animation,
    AnimationSerie,
    Concert,
    Documentaire,
    EmissionTv,
    Films,
    SeriesTv,
    Spectacle,
    Sport,
    VideoClips,
}

#[derive(Debug)]
pub enum Lang {
    Anglais,
    French,
    Muet,
    MultiInclusFr,
    MultiInclusQuebecois,
    Quebecois,
    Vfstfr,
    Vostfr,
}

#[derive(Debug)]
pub enum Quality {
    SdRip,
    Bluray4kFullAndRemux,
    Bluray4kFull,
    Bluray4kRemux,
    DvdR5,
    DvdR9,
    DvdRip,
    HdRip1080,
    HdRip4k,
    HdRip720,
    TvRipSd,
    TvRip1080,
    TvRip4k,
    TvRip720,
    VCD_SVCD_VHSRIP,
    WebDL,
    WebDL1080,
    WebDl4k,
    WebDl720,
    WebRip,
    WebRip1080,
    WebRip4k,
    WebRip720,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Films => "2145",
        }
    }
}

impl SubCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubCategory::Animation => "2178",
            SubCategory::AnimationSerie => "2179",
            SubCategory::Concert => "2180",
            SubCategory::Documentaire => "2181",
            SubCategory::EmissionTv => "2182",
            SubCategory::Films => "2183",
            SubCategory::SeriesTv => "2184",
            SubCategory::Spectacle => "2185",
            SubCategory::Sport => "2186",
            SubCategory::VideoClips => "2187",
        }
    }
}

impl Lang {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lang::Anglais => "1",
            Lang::French => "2",
            Lang::Muet => "3",
            Lang::MultiInclusFr => "4",
            Lang::MultiInclusQuebecois => "5",
            Lang::Quebecois => "6",
            Lang::Vfstfr => "7",
            Lang::Vostfr => "8",
        }
    }
}

impl Quality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Quality::SdRip => "1",
            Quality::Bluray4kFullAndRemux => "2",
            Quality::Bluray4kFull => "3",
            Quality::Bluray4kRemux => "4",
            Quality::DvdR5 => "5",
            Quality::DvdR9 => "6",
            Quality::DvdRip => "7",
            Quality::HdRip1080 => "8",
            Quality::HdRip4k => "9",
            Quality::HdRip720 => "10",
            Quality::TvRipSd => "11",
            Quality::TvRip1080 => "12",
            Quality::TvRip4k => "13",
            Quality::TvRip720 => "14",
            Quality::VCD_SVCD_VHSRIP => "15",
            Quality::WebDL => "16",
            Quality::WebDL1080 => "17",
            Quality::WebDl4k => "18",
            Quality::WebDl720 => "19",
            Quality::WebRip => "20",
            Quality::WebRip1080 => "21",
            Quality::WebRip4k => "22",
            Quality::WebRip720 => "23",
        }
    }
}

pub fn build_search_query(
    name: &str,
    category: Category,
    subcategory: SubCategory,
    langs: &[Lang],
    qualities: &[Quality],
) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("name", name.to_string()),
        ("description", "".into()),
        ("file", "".into()),
        ("uploader", "".into()),
        ("category", category.as_str().to_string()),
        ("sub_category", subcategory.as_str().to_string()),
        ("do", "search".into()),
    ];

    for lang in langs {
        query.push(("option_langue:multiple[]", lang.as_str().to_string()));
    }

    for qual in qualities {
        query.push(("option_qualite[]", qual.as_str().to_string()));
    }

    query
}
