use std::collections::BTreeMap;
use std::fs;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};

fn find_videos_and_trickplay(folder: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut video_files = Vec::new();
    let mut trickplay_folders = Vec::new();

    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let sub_path = entry.path();

            if sub_path.is_dir() {
                if sub_path.file_name().map_or(false, |name| name.to_string_lossy().ends_with(".trickplay")) {
                    trickplay_folders.push(sub_path);
                } else {
                    let (sub_videos, sub_trickplay) = find_videos_and_trickplay(&sub_path);
                    video_files.extend(sub_videos);
                    trickplay_folders.extend(sub_trickplay);
                }
            } else if let Some(ext) = sub_path.extension() {
                if ext == "mkv" || ext == "mp4" {
                    video_files.push(sub_path);
                }
            }
        }
    }

    (video_files, trickplay_folders)
}

fn main() {
    let paths = [
        ("Films", [
            Path::new("/Volumes/3To/Films"),
            Path::new("/Volumes/470G M2/film"),
            Path::new("/Users/anna/Movies/film"),
        ]),
        ("Anime", [
            Path::new("/Volumes/3To/Anime"),
            Path::new("/Volumes/470G M2/Anime"),
            Path::new("/Users/anna/Movies/Anime"),
        ]),
        ("Séries", [
            Path::new("/Volumes/3To/Séries"),
            Path::new("/Volumes/470G M2/séries"),
            Path::new("/Users/anna/Movies/séries"),
        ]),
    ];

    let mut total_mkv = 0;
    let mut total_trickplay = 0;
    let mut total_missing_trickplay = 0;
    let mut total_orphan_trickplay = 0;
    let mut estimated_time = 0;

    let mut missing_trickplay_by_category: BTreeMap<&str, BTreeMap<&Path, Vec<String>>> = BTreeMap::new();
    let mut orphan_trickplay_by_category: BTreeMap<&str, BTreeMap<&Path, Vec<String>>> = BTreeMap::new();

    for (category, base_paths) in &paths {
        let mut missing_results: BTreeMap<&Path, Vec<String>> = BTreeMap::new();
        let mut orphan_results: BTreeMap<&Path, Vec<String>> = BTreeMap::new();

        for base_path in base_paths {
            if let Ok(entries) = fs::read_dir(base_path) {
                let mut missing_files = Vec::new();
                let mut orphan_files = Vec::new();

                for entry in entries.flatten() {
                    let folder_path = entry.path();
                    if folder_path.is_dir() {
                        let (mkv_files, trickplay_folders) = find_videos_and_trickplay(&folder_path);

                        total_mkv += mkv_files.len();
                        total_trickplay += trickplay_folders.len();

                        let trickplay_names: Vec<&std::ffi::OsStr> = trickplay_folders
                            .iter()
                            .filter_map(|tp| tp.file_name())
                            .collect();

                        let mkv_names: Vec<&std::ffi::OsStr> = mkv_files
                            .iter()
                            .filter_map(|mkv| mkv.file_stem())
                            .collect();

                        // Vérifie les fichiers MKV sans Trickplay
                        let missing_trickplay: Vec<String> = mkv_names
                            .iter()
                            .filter(|name| !trickplay_names.iter().any(|tp| tp.as_bytes().starts_with(name.as_bytes())))
                            .map(|name| name.to_string_lossy().into_owned())
                            .collect();

                        // Vérifie les Trickplay sans MKV
                        let orphan_trickplay: Vec<String> = trickplay_names
                            .iter()
                            .filter(|tp| !mkv_names.iter().any(|mkv| tp.as_bytes().starts_with(mkv.as_bytes())))
                            .map(|tp| tp.to_string_lossy().into_owned())
                            .collect();

                        total_missing_trickplay += missing_trickplay.len();
                        total_orphan_trickplay += orphan_trickplay.len();
                        missing_files.extend(missing_trickplay);
                        orphan_files.extend(orphan_trickplay);
                    }
                }

                if !missing_files.is_empty() {
                    missing_results.insert(base_path, missing_files);
                }
                if !orphan_files.is_empty() {
                    orphan_results.insert(base_path, orphan_files);
                }
            }
        }

        if !missing_results.is_empty() {
            missing_trickplay_by_category.insert(category, missing_results);
        }
        if !orphan_results.is_empty() {
            orphan_trickplay_by_category.insert(category, orphan_results);
        }
    }

    for (category, paths) in &missing_trickplay_by_category {
        println!("\n📂 **{}**", category);
        for (base_path, files) in paths {
            println!("  📁 {:?} ({} manquants)", base_path, files.len());
            for file in files {
                println!("    ❌ {}", file);
            }
        }
    }

    for (category, paths) in &orphan_trickplay_by_category {
        println!("\n📂 **{}** (Trickplay orphelins)", category);
        for (base_path, files) in paths {
            println!("  📁 {:?} ({} orphelins)", base_path, files.len());
            for file in files {
                println!("    🚨 {}", file);
            }
        }
    }

    let computed_mkv = total_trickplay + total_missing_trickplay - total_orphan_trickplay;
    println!("\n--- Résumé ---");
    println!("Total .mkv trouvés       : {}", total_mkv);
    println!("Total .trickplay trouvés : {}", total_trickplay);
    println!("Total manquants          : {}", total_missing_trickplay);
    println!("Total Trickplay orphelins: {}", total_orphan_trickplay);
    println!(
        "Vérification : {} + {} - {} → {}",
        total_trickplay, total_missing_trickplay, total_orphan_trickplay, computed_mkv
    );

    if total_mkv != computed_mkv {
        println!("⚠️  Attention : Il y a une différence de {} fichiers non comptabilisés.",
                 (total_mkv as isize - computed_mkv as isize).abs());
    }

    println!("\n--- Trickplay manquants ---");
    for (category, paths) in &missing_trickplay_by_category {
        let count: usize = paths.values().map(|v| v.len()).sum();
        println!("{} : {} fichiers manquants", category, count);

        let time_per_file = match *category {
            "Anime" => 1,
            "Films" => 40,
            "Séries" => 20,
            _ => 0,
        };
        estimated_time += count * time_per_file;
    }

    println!("\n--- Estimation du temps pour générer les Trickplay ---");
    println!("Temps total estimé : ~{} min ({}h{}min)",
             estimated_time,
             estimated_time / 60,
             estimated_time % 60
    );
}
