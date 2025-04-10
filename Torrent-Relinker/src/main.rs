use walkdir::WalkDir;
use std::cmp::min;
use serde_json::to_string_pretty;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

fn main() {

    let vec_paths = vec![
        ("/Users/anna/Movies/Films", "/Users/anna/Downloads/AAAA dl/Films", "films.json", "unmatched_films.json", false),
        //("/Volumes/3To/Anime", "/Users/anna/Downloads/Séries", "anime.json", "unmatched_anime.json", true),
        //("/Volumes/3To/Séries", "/Users/anna/Downloads/Séries", "series.json", "unmatched_series.json", true),
    ];

    for (data_path, torrent_path, ok_name, err_name, match_by_folder) in vec_paths {


        println!("Matching files in: {} with {}", data_path, torrent_path);


        let media_items = if match_by_folder {
            get_folders_in_folder(data_path) // Séries et animés -> noms de dossier
        } else {
            get_files_in_folder(data_path, vec!["mkv"]) // Films -> noms de fichier
        };

        println!("Nb media found {}", media_items.len());

        let torrent_files = get_files_in_folder(torrent_path, vec!["torrent"]);



        let mut matches_map = HashMap::new();
        let mut unmatched_files = Vec::new();

        for media in media_items {
            let mut best_match = None;
            let mut best_distance = 11;

            for torrent in &torrent_files {
                let media_clean = clean_name(&media);
                let torrent_clean = clean_name(&torrent);
                let distance = levenshtein(&media_clean, &torrent_clean);

                if distance < best_distance {
                    best_distance = distance;
                    best_match = Some(torrent.clone());
                }
            }

            if let Some(best_match) = best_match {
                let torrent_path = format!("{}/{}", torrent_path, best_match);
                    matches_map.insert(torrent_path, format!("{}/{}", data_path, media));
            } else {
                let media_path = format!("{}/{}", data_path, media);
                unmatched_files.push(media_path);
            }
        }

        let json_output = to_string_pretty(&matches_map).expect("Error generating JSON");

        let mut file = File::create(ok_name).expect("Unable to create file");
        file.write_all(json_output.as_bytes())
            .expect("Unable to write data to file");

        if unmatched_files.len() != 0 {
            let unmatched_json = to_string_pretty(&unmatched_files).expect("Error generating unmatched JSON");

            let mut unmatched_file = File::create(err_name).expect("Unable to create unmatched file");
            unmatched_file.write_all(unmatched_json.as_bytes())
                .expect("Unable to write unmatched files");
        }


        println!("Matches saved to {}", ok_name);
        println!("Unmatched files saved to {}", err_name);


    }


}

fn get_files_in_folder(folder: &str, valid_extensions: Vec<&str>) -> Vec<String> {
    WalkDir::new(folder)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.file_type().is_file() {
                    let ext = e.path().extension()?.to_string_lossy().to_lowercase();
                    if valid_extensions.contains(&ext.as_str()) {
                        return Some(e.path().file_name()?.to_string_lossy().to_string());
                    }
                }
                None
            })
        })
        .collect()
}

fn get_folders_in_folder(folder: &str) -> Vec<String> {
    WalkDir::new(folder)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(e) = entry {
                let path = e.path();
                let absolute_path = path.to_path_buf();

                if absolute_path.ends_with("Séries") || absolute_path.ends_with("Anime") {
                    return None;
                }

                if e.file_type().is_dir() && path.exists() {
                    return path.file_name().map(|name| clean_name(&name.to_string_lossy().into_owned()));
                }
            }
            None
        })
        .collect()
}

fn clean_name(name: &str) -> String {
    let cleaned = name
        .to_lowercase()
        .replace(&['(', ')', '[', ']', '.', '_', '-', ' '][..], "")
        .replace(|c: char| !c.is_alphanumeric(), "");

    let keywords_to_remove = vec!["2160p", "uhd", "10bit", "4k", "dtshdma", "1080p", "hdr", "dv", "x265", "web", "bluray", "ac3", "dts", "51", "71", "truehd", "light", "atm", "vff", "french", "multivf", "multi", "h265", "hevc", "remux", "bdrip", "fwmkv", "dtshd"];
    keywords_to_remove.iter().fold(cleaned, |acc, keyword| acc.replace(keyword, ""))
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }
    else if a.is_empty() { return b.len(); }
    else if b.is_empty() { return a.len(); }

    let mut prev_distances: Vec<usize> = (0..=b.len()).collect();
    let mut curr_distances: Vec<usize> = vec![0; b.len() + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_distances[j + 1] = min(
                curr_distances[j] + 1,
                min(prev_distances[j + 1] + 1, prev_distances[j] + cost),
            );
        }

        prev_distances.clone_from(&curr_distances);
    }

    curr_distances[b.len()]
}
