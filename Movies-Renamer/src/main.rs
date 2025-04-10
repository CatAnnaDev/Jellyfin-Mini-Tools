use std::fs;
use std::path::Path;

fn main() {
    let movies_array = [Path::new("/Users/anna/Movies/Films"), Path::new("/Volumes/470G M2/film"), Path::new("/Users/anna/Movies/film")];

    for movie_dir in movies_array {
        if !movie_dir.is_dir() {
            eprintln!("Le chemin spécifié n'est pas un répertoire valide.");
            return;
        }

        if let Err(e) = organize_movies(movie_dir) {
            eprintln!("Erreur lors de l'organisation des films : {}", e);
        }
    }
}

fn organize_movies(movie_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(movie_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".trickplay") {
                    let title_with_year = extract_title_with_year(file_name);
                    let new_dir = movie_dir.join(title_with_year);
                    if !new_dir.exists() { fs::create_dir(&new_dir)?; }
                    let target_path = new_dir.join(file_name);
                    fs::rename(&path, &target_path)?;
                    println!("trickplay Déplacé: {} -> {}", path.display(), target_path.display());
                }
            }
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let title_with_year = extract_title_with_year(file_name);
            let new_dir = movie_dir.join(title_with_year);
            if !new_dir.exists() { fs::create_dir(&new_dir)?; }
            let target_path = new_dir.join(file_name);
            fs::rename(&path, &target_path)?;
            println!("Déplacé: {} -> {}", path.display(), target_path.display());
        }
    }

    Ok(())
}

fn extract_title_with_year(file_name: &str) -> String {
    let mut title = String::new();
    let mut found_year = false;

    if let Some(start) = file_name.find('(') {
        if let Some(end) = file_name[start..].find(')') {
            let year_section = &file_name[start + 1..start + end];
            if year_section.len() == 4 && year_section.chars().all(|c| c.is_digit(10)) {
                title = file_name[..start].trim().to_string();
                title.push_str(" (");
                title.push_str(year_section);
                title.push(')');
                found_year = true;
            }
        }
    }

    if !found_year {
        let parts: Vec<&str> = file_name.split(['.', ' '].as_ref()).collect();
        for part in parts {
            if part.len() == 4 && part.chars().all(|c| c.is_digit(10)) {
                title.push_str(&file_name[..file_name.find(part).unwrap_or(0)].trim());
                title.push_str(part);
                break;
            }
        }
    }

    if title.is_empty() {
        title = "Unknown Title".to_string();
    }

    title.replace(".", " ").trim().to_string()
}
