use std::{fs::{self, File}, io::{self, Write}, path::{Path, PathBuf}};
use clap::{Parser, ArgAction};
use eframe::egui;
use serde::Serialize;
use indicatif::{ProgressBar, ProgressStyle};

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

    #[arg(short = 'i', long, help = "include all files or just movies, true = all files", action = ArgAction::SetTrue)]
    include_all: bool,

    #[arg(short = 't', long, help = "Output file type for the result, txt or json", default_value = "txt")]
    output_type: String,

    #[arg(long, help = "Simulate the run without writing any file", action = ArgAction::SetTrue)]
    dry_run: bool,

    #[arg(long, help = "Show result in ui", action = ArgAction::SetTrue)]
    ui: bool,

}

#[derive(Default, Debug)]
struct Summary {
    total_files: u64,
    total_folders: u64,
    total_size: u64,
}

#[derive(Debug, Serialize)]
struct FileNode {
    name: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct FolderNode {
    name: String,
    size: u64,
    files: Vec<FileNode>,
    subfolders: Vec<FolderNode>,
}

fn count_entries(dir: &Path, include_all: bool) -> u64 {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            let is_trickplay = path.file_name()
                .map(|n| n.to_string_lossy().contains(".trickplay"))
                .unwrap_or(false);

            if !include_all && is_trickplay {
                continue;
            }

            count += 1;
            if path.is_dir() {
                count += count_entries(&path, include_all);
            }
        }
    }

    count
}

fn visit_dirs(dir: &Path, debug: bool, include_all: bool, summary: &mut Summary, pb: &ProgressBar) -> io::Result<FolderNode> {
    summary.total_folders += 1;
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
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if name.starts_with('.') {
                continue;
            }

            if !include_all && name.contains(".trickplay") {
                continue;
            }

            pb.inc(1);
            
            match visit_dirs(&path, debug, include_all, summary, pb) {
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

            if !include_all && !file_name.ends_with(".mp4") && !file_name.ends_with(".mkv") && !file_name.ends_with(".avi") {
                continue;
            }

            match path.metadata() {
                Ok(metadata) => {
                    let size = metadata.len();
                    folder.size += size;
                    folder.files.push(FileNode { name: file_name, size });
                    summary.total_files += 1;
                    summary.total_size += size;
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

pub enum SizeUnit {
    Decimal, // KB, MB, GB
    Binary,  // KiB, MiB, GiB
}

pub fn format_size(size_in_bytes: u64, decimals: usize, unit_type: SizeUnit, force_unit: Option<usize>, ) -> String {
    let (units, factor): (&[&str], f64) = match unit_type {
        SizeUnit::Decimal => (&["B", "KB", "MB", "GB", "TB", "PB"], 1000.0),
        SizeUnit::Binary  => (&["B", "KiB", "MiB", "GiB", "TiB", "PiB"], 1024.0),
    };

    let mut size = size_in_bytes as f64;
    let mut unit = 0;

    if let Some(forced) = force_unit {
        size /= factor.powi(forced as i32);
        unit = forced;
    } else {
        while size >= factor && unit < units.len() - 1 {
            size /= factor;
            unit += 1;
        }
    }

    format!("{:.*} {}", decimals, size, units[unit])
}

fn sort_folder(folder: &mut FolderNode, sort_by: &str) {
    match sort_by {
        "folder" => {
            folder.subfolders.sort_by(|a, b| b.size.cmp(&a.size));
        }
        _ => {
            folder.files.sort_by(|a, b| b.size.cmp(&a.size));
        }
    }

    for subfolder in &mut folder.subfolders {
        sort_folder(subfolder, sort_by);
    }
}

fn write_tree(folder: &FolderNode, output: &mut File, indent: usize) -> io::Result<()> {
    let prefix = "│   ".repeat(indent);
    writeln!(output, "{}├── {} ({})", prefix, folder.name, format_size(folder.size, 2, SizeUnit::Decimal, None))?;

    for file in &folder.files {
        writeln!(
            output,
            "{}│   ├── {} ({})",
            prefix,
            file.name,
            format_size(file.size, 2, SizeUnit::Decimal, None)
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
    let mut output_path = PathBuf::from(&args.output);
    let mut summary = Summary::default();

    let total = count_entries(&base_path, args.include_all);
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap()
        .progress_chars("#>-"));


    output_path.set_extension(match args.output_type.as_str() {
        "json" => "json",
        _ => "txt",
    });


    match visit_dirs(&base_path, args.debug, args.include_all, &mut summary,  &pb) {
        Ok(mut folder_structure) => {
            pb.finish_with_message("Analyse terminée.");
            sort_folder(&mut folder_structure, &args.sort);


            if args.ui {
                show_ui(folder_structure); // Appelle l'interface graphique
                return;
            }

            if args.dry_run {
                println!("Dry-run mode: no output file written.");
                println!("Summary:\n- Total folders: {}\n- Total files: {}\n- Total size: {}",
                         summary.total_folders,
                         summary.total_files,
                         format_size(summary.total_size, 2, SizeUnit::Decimal, None)
                );
                return;
            }

            match File::create(&output_path) {
                Ok(mut output_file) => {
                    match args.output_type.as_str() {
                        "json" => {
                            match serde_json::to_writer_pretty(&mut output_file, &folder_structure) {
                                Ok(_) => println!("JSON saved to {}", output_path.display()),
                                Err(e) => eprintln!("Failed to write JSON: {}", e),
                            }
                        }
                        _ => {
                            if let Err(e) = write_tree(&folder_structure, &mut output_file, 0) {
                                eprintln!("Failed to write to output file: {}", e);
                            } else {
                                println!("Analysis saved to {}", output_path.display());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create output file: {}", e);
                }
            }

            println!("Summary:\n- Total folders: {}\n- Total files: {}\n- Total size: {}",
                     summary.total_folders,
                     summary.total_files,
                     format_size(summary.total_size, 2, SizeUnit::Decimal, None)
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn show_ui(json_data: FolderNode) {
    let _ = eframe::run_native(
        "Résultat Analyse Dossier",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(JsonViewerApp { root_folder: json_data }))),
    );
}

struct JsonViewerApp {
    root_folder: FolderNode,
}

impl JsonViewerApp {
    fn display_folder_tree(&self, ui: &mut egui::Ui, folder: &FolderNode) {
        ui.collapsing(&folder.name, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Taille: {}", format_size(folder.size, 2,  SizeUnit::Decimal, None)));
            });

            for file in &folder.files {
                ui.horizontal(|ui| {
                    ui.label(format!("Fichier: {}", file.name));
                    ui.label(format!("Taille: {}", format_size(file.size, 2,  SizeUnit::Decimal, None)));
                });
            }

            for subfolder in &folder.subfolders {
                self.display_folder_tree(ui, subfolder);
            }
        });
    }
}

impl eframe::App for JsonViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Arborescence des Dossiers :");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));

                self.display_folder_tree(ui, &self.root_folder);
            });
        });
    }
}