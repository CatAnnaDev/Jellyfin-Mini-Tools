# 📂 Directory Analyzer

Un outil Rust pour analyser de manière récursive la taille et la structure des dossiers. Permet de générer des rapports triés, en texte ou en JSON, avec ou sans interface graphique.

## ✨ Fonctionnalités

- Analyse récursive des fichiers et dossiers
- Tri par taille de fichiers ou de dossiers
- Filtres personnalisés (films uniquement ou tous les fichiers)
- Export des résultats en `.txt` ou `.json`
- Interface utilisateur graphique avec `eframe/egui`
- Support du mode `dry-run` pour simuler l'exécution
- Barre de progression (`indicatif`) pour suivre l’analyse

## 🚀 Installation

Assurez-vous d’avoir [Rust](https://www.rust-lang.org/tools/install) installé, puis :

```bash
git clone https://github.com/CatAnnaDev/Jellyfin-Mini-Tools.git
cd Jellyfin-Mini-Tools/Size-Check
cargo build --release
```
## 🧪 Utilisation

cargo run --release -- [OPTIONS]

## ⚙️ Options disponibles

```text
Option	Description
-p, --path	Dossier de base à analyser (défaut: /Volumes/3To)
-o, --output	Nom du fichier de sortie (défaut: output.txt)
-t, --output-type	Type du fichier de sortie (txt ou json) (défaut: txt)
-s, --sort	Tri par file ou folder (défaut: file)
-i, --include-all	Inclure tous les fichiers, pas seulement les films (.mp4, .mkv, etc.)
-d, --debug	Afficher les logs de débogage
--dry-run	N'écrit pas de fichier, affiche uniquement le résumé
--ui	Affiche les résultats dans une interface graphique
--help	Affiche l’aide
--version	Affiche la version
```
## 📦 Exemple d'utilisation

# Analyse le dossier /Volumes/3To et exporte en JSON
cargo run --release -- -p /Volumes/3To -o result -t json

# Affiche les résultats dans une UI graphique
cargo run --release -- -p ./Videos --ui

# Simule une analyse sans rien écrire
cargo run --release -- -p ./Films --dry-run

# Tri par taille de dossiers
cargo run --release -- -s folder
