use std::{fs::{self, File}, io::{self, Write}, path::{Path, PathBuf}};
use std::collections::HashSet;
use clap::{Parser, ArgAction};
use eframe::egui;
use eframe::egui::{Id, StrokeKind};
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

#[derive(Debug, Serialize, Clone)]
struct FileNode {
    path: String,
    name: String,
    size: u64,
}

#[derive(Debug, Serialize, Clone)]
struct FolderNode {
    path: String,
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

fn visit_dirs(
    dir: &Path,
    debug: bool,
    include_all: bool,
    summary: &mut Summary,
    pb: &ProgressBar,
) -> io::Result<FolderNode> {
    summary.total_folders += 1;
    let mut folder = FolderNode {
        path: dir.display().to_string(),
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
                    folder.path = path.display().to_string();
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
                    folder.files.push(FileNode { path: path.display().to_string(), name: file_name, size });
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
                show_ui(folder_structure);
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
        Box::new(|_cc| Ok(Box::new(JsonViewerApp { root_folder: json_data, pending_deletions: Vec::new(), selected_files: HashSet::new(), confirm_deletion: false }))),
    );
}

#[derive(Debug, Clone)]
struct JsonViewerApp {
    root_folder: FolderNode,
    pending_deletions: Vec<String>,
    selected_files: HashSet<String>,
    confirm_deletion: bool,
}

impl JsonViewerApp {
    fn apply_deletions(&mut self) {
        self.root_folder.files
            .retain(|f| !self.pending_deletions.contains(&f.path));
        for sub in &mut self.root_folder.subfolders {
            Self::apply_deletions_to_folder(sub, &self.pending_deletions);
        }
        self.root_folder.subfolders
            .retain(|d| !self.pending_deletions.contains(&d.path));
        self.pending_deletions.clear();
    }

    fn apply_deletions_to_folder(folder: &mut FolderNode, pending: &[String]) {
        folder.files.retain(|f| !pending.contains(&f.path));
        for sub in &mut folder.subfolders {
            Self::apply_deletions_to_folder(sub, pending);
        }
        folder.subfolders.retain(|d| !pending.contains(&d.path));
    }

    fn select_folder_recursive(&mut self, folder: &FolderNode) {
        self.selected_files.insert(folder.path.clone());
        for file in &folder.files {
            self.selected_files.insert(file.path.clone());
        }
        for subfolder in &folder.subfolders {
            self.select_folder_recursive(subfolder);
        }
    }

    fn deselect_folder_recursive(&mut self, folder: &FolderNode) {
        self.selected_files.remove(&folder.path);
        for file in &folder.files {
            self.selected_files.remove(&file.path);
        }
        for subfolder in &folder.subfolders {
            self.deselect_folder_recursive(subfolder);
        }
    }

    fn are_all_children_selected(&self, folder: &FolderNode) -> bool {
        folder.files.iter().all(|f| self.selected_files.contains(&f.path)) &&
            folder.subfolders.iter().all(|sf| self.are_all_children_selected(sf)) &&
            self.selected_files.contains(&folder.path)
    }

    fn are_any_children_selected(&self, folder: &FolderNode) -> bool {
        folder.files.iter().any(|f| self.selected_files.contains(&f.path)) ||
            folder.subfolders.iter().any(|sf| self.are_any_children_selected(sf)) ||
            self.selected_files.contains(&folder.path)
    }

    fn display_folder_tree(&mut self, ui: &mut egui::Ui, folder: &mut FolderNode) {
        let label_text = format!(
            "{} ({})",
            &folder.name,
            format_size(folder.size, 2, SizeUnit::Decimal, None)
        );

        let header_response = ui.horizontal(|ui| {

            let all_selected = self.are_all_children_selected(folder);
            let any_selected = self.are_any_children_selected(folder);

            let mut is_checked = all_selected;
            let is_mixed = !all_selected && any_selected;

            let checkbox_response = if is_mixed {
                let (rect, response) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                let visuals = ui.style().interact(&response);
                ui.painter().rect(
                    rect.shrink(2.0),
                    3.0,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    StrokeKind::Middle
                );
                let rect_inner = rect.shrink(5.0);
                ui.painter().line_segment(
                    [rect_inner.left_top(), rect_inner.right_bottom()],
                    (1.5, visuals.fg_stroke.color),
                );
                ui.painter().line_segment(
                    [rect_inner.right_top(), rect_inner.left_bottom()],
                    (1.5, visuals.fg_stroke.color),
                );
                response
            } else {
                ui.checkbox(&mut is_checked, "")
            };

            if checkbox_response.clicked() {
                if all_selected || is_mixed {
                    self.deselect_folder_recursive(folder);
                } else {
                    self.select_folder_recursive(folder);
                }
            }


            let collapsing = ui.collapsing(label_text, |ui| {
                for file in &folder.files {
                    let file_selected = self.selected_files.contains(&file.path);
                    let mut file_checked = file_selected;

                    let file_label = format!(
                        "{} ({})",
                        file.name,
                        format_size(file.size, 2, SizeUnit::Decimal, None)
                    );

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut file_checked, "").changed() {
                            if file_checked {
                                self.selected_files.insert(file.path.clone());
                            } else {
                                self.selected_files.remove(&file.path);
                            }
                        }

                        let response = ui.add(egui::Label::new(file_label).sense(egui::Sense::click()));

                        if response.secondary_clicked() {
                            ui.ctx().memory_mut(|mem| {
                                mem.open_popup(Id::new(format!("popup_{}", file.name)));
                            });
                        }

                        egui::popup::popup_below_widget(
                            ui,
                            Id::new(format!("popup_{}", file.name)),
                            &response,
                            egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                if ui.button("Open file").clicked() {
                                    open_file_or_folder(&file.path);
                                    ui.close_menu();
                                }
                                if ui.button("Delete file").clicked() {
                                    self.selected_files.insert(file.path.clone());
                                    self.confirm_deletion = true;
                                    ui.close_menu();
                                }
                            },
                        );
                    });
                }

                let mut i = 0;
                while i < folder.subfolders.len() {
                    let subfolder = &mut folder.subfolders[i];
                    self.display_folder_tree(ui, subfolder);

                    if !Path::new(&subfolder.path).exists() {
                        folder.subfolders.remove(i);
                    } else {
                        i += 1;
                    }
                }
            });

            collapsing.header_response.clone()
        }).inner;

        if header_response.secondary_clicked() {
            ui.ctx().memory_mut(|mem| {
                mem.open_popup(Id::new(format!("popup_{}", folder.name)));
            });
        }

        egui::popup::popup_below_widget(
            ui,
            Id::new(format!("popup_{}", folder.name)),
            &header_response,
            egui::popup::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                if ui.button("Open Folder").clicked() {
                    open_file_or_folder(&folder.path);
                    ui.close_menu();
                }
                if ui.button("Delete Folder").clicked() {
                    self.selected_files.insert(folder.path.clone());
                    self.confirm_deletion = true;
                    ui.close_menu();
                }
            },
        );
    }
}

impl eframe::App for JsonViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Arborescence des Dossiers :");

            if !self.selected_files.is_empty() {
                if ui.button(format!("🗑 Supprimer les éléments sélectionnés ({})", self.selected_files.len())).clicked() {
                    self.confirm_deletion = true;
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                let mut folder = self.root_folder.clone();
                self.display_folder_tree(ui, &mut folder);
            });

            if self.confirm_deletion {
                egui::Window::new("Confirmer la suppression")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Êtes-vous sûr de vouloir supprimer les éléments sélectionnés ?");
                        if ui.button("Oui, supprimer").clicked() {
                            for path in &self.selected_files {
                                delete_file_or_folder(path);
                                self.pending_deletions.push(path.clone());
                            }
                            self.selected_files.clear();
                            self.confirm_deletion = false;
                        }
                        if ui.button("Annuler").clicked() {
                            self.confirm_deletion = false;
                        }
                    });
            }
        });

        self.apply_deletions();
    }
}
use std::process::Command;

fn open_file_or_folder(file_path: &str) {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(file_path)
            .spawn()
            .expect("Failed to open the file or folder");
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(file_path)
            .spawn()
            .expect("Failed to open the file or folder");
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(file_path)
            .spawn()
            .expect("Failed to open the file or folder");
    }
}

fn delete_file_or_folder(file_path: &str) {
    if fs::remove_file(file_path).is_err() {
        if let Err(e) = fs::remove_dir_all(file_path) {
            println!("Failed to delete the file or folder: {}", e);
        }else { 
            println!("Successfully delete the file or folder");
        }
    } else {
        println!("File deleted: {}", file_path);
    }
}
