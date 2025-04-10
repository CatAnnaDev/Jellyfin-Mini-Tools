use std::{fs, io, fs::File, io::Write};
use std::path::{Path, PathBuf};
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(author = "CatAnnaDev", version, about = "Directory Analyzer with Sorting", long_about = None)]
struct ClapArgs {
    #[arg(short = 'p', long, help = "Base directory to analyze", default_value = "/Volumes/3To")]
    path: String,

    #[arg(short = 'o', long, help = "Output file for the result", default_value = "output.txt")]
    output: String,

    #[arg(short = 'd', long, help = "Enable debug logs", action = ArgAction::SetTrue)]
    debug: bool,

    #[arg(short = 's', long, help = "Sorting method (folder or file)", default_value = "file")]
    sort: String,
}

#[derive(Debug)]
struct FileNode {
    name: String,
    size: u64,
}

#[derive(Debug)]
struct FolderNode {
    name: String,
    size: u64,
    files: Vec<FileNode>,
    subfolders: Vec<FolderNode>,
}

fn visit_dirs(dir: &Path, debug: bool) -> io::Result<FolderNode> {
    let mut folder = FolderNode {
        name: dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| dir.display().to_string()),
        size: 0,
        files: vec![],
        subfolders: vec![],
    };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                continue;
            }
            match visit_dirs(&path, debug) {
                Ok(subfolder) => {
                    folder.size += subfolder.size;
                    folder.subfolders.push(subfolder);
                }
                Err(e) => {
                    if debug {
                        eprintln!("Failed to read directory {}: {}", path.display(), e);
                    }
                }
            }
        } else {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if file_name == ".DS_Store" {
                continue;
            }

            match path.metadata() {
                Ok(metadata) => {
                    let size = metadata.len();
                    folder.size += size;
                    folder.files.push(FileNode { name: file_name, size });
                }
                Err(e) => {
                    if debug {
                        eprintln!("Failed to get metadata for {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(folder)
}

fn format_size(size_in_bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size_in_bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:.2} {}", size, UNITS[unit])
}

fn sort_folder(folder: &mut FolderNode, sort_by: &str) {
    folder.files.sort_by(|a, b| b.size.cmp(&a.size));

    folder.subfolders.sort_by(|a, b| b.size.cmp(&a.size));

    for subfolder in &mut folder.subfolders {
        sort_folder(subfolder, sort_by);
    }
}

fn write_tree(folder: &FolderNode, output: &mut File, indent: usize) -> io::Result<()> {
    let prefix = "│   ".repeat(indent);
    writeln!(output, "{}├── {} ({})", prefix, folder.name, format_size(folder.size))?;

    for file in &folder.files {
        writeln!(
            output,
            "{}│   ├── {} ({})",
            prefix,
            file.name,
            format_size(file.size)
        )?;
    }

    for (_, subfolder) in folder.subfolders.iter().enumerate() {
        write_tree(subfolder, output, indent + 1)?;
    }
    Ok(())
}

fn main() {
    let args = ClapArgs::parse();

    let base_path = PathBuf::from(&args.path);
    let output_path = PathBuf::from(&args.output);

    match visit_dirs(&base_path, args.debug) {
        Ok(mut folder_structure) => {
            sort_folder(&mut folder_structure, &args.sort);

            match File::create(&output_path) {
                Ok(mut output_file) => {
                    if let Err(e) = write_tree(&folder_structure, &mut output_file, 0) {
                        eprintln!("Failed to write to output file: {}", e);
                    } else {
                        println!("Analysis saved to {}", output_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create output file: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
