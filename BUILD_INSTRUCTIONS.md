# 🛠️ Guide de Build et Installation

Ce guide vous aidera à compiler et installer **Spotlight Windows** sur votre machine Windows.

## 📋 Prérequis

### 1. Installer Rust

```bash
# Télécharger et exécuter rustup-init.exe depuis:
# https://rustup.rs/

# Ou via PowerShell:
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

Suivez les instructions à l'écran et choisissez l'installation par défaut.

Vérifiez l'installation:
```bash
rustc --version
cargo --version
```

### 2. Cloner le Repository

```bash
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows
```

## 🚀 Compilation

### Build Debug (pour le développement)

Rapide à compiler mais non optimisé:

```bash
cargo build
```

L'exécutable se trouve dans `./target/debug/spotlight_windows.exe`

### Build Release (recommandé pour l'utilisation)

Optimisé avec LTO, plus lent à compiler mais TRÈS rapide à l'exécution:

```bash
cargo build --release
```

L'exécutable se trouve dans `./target/release/spotlight_windows.exe`

**Temps de compilation estimé:** 3-5 minutes (première fois)

## ▶️ Lancement

### Option 1: Via Cargo (développement)

```bash
# Mode debug
cargo run

# Mode release
cargo run --release
```

### Option 2: Exécuter directement

```bash
# Debug
.\target\debug\spotlight_windows.exe

# Release
.\target\release\spotlight_windows.exe
```

### Option 3: Installer système-wide

```bash
# Installe dans ~/.cargo/bin/ (ajouté au PATH)
cargo install --path .

# Ensuite, lancez depuis n'importe où:
spotlight_windows
```

## ⚙️ Configuration

La configuration se crée automatiquement au premier lancement dans:
```
C:\Users\VotreNom\.config\spotlight_windows\config.toml
```

### Modifier les dossiers à indexer

Éditez `config.toml`:

```toml
[indexer]
include_paths = [
    "C:\\Users\\VotreNom",
    "C:\\Users\\VotreNom\\Documents",
    "D:\\Projects"
]

exclude_paths = [
    "node_modules",
    ".git",
    "target"
]
```

## 🎯 Utilisation

### Premier lancement

1. Lancez l'application
2. **Attendez l'indexation initiale** (visible dans la console)
   - ~500k fichiers = 45 secondes
   - ~100k fichiers = 10 secondes
3. Commencez à taper dans la barre de recherche !

### Recherche

- **Par nom**: Tapez le nom du fichier
  - Exemple: `rapport` trouve `rapport-2024.pdf`

- **Fuzzy**: Fautes de frappe tolérées
  - Exemple: `rpport` trouve quand même `rapport.pdf`

- **Dans le contenu**: Chercher du texte DANS les fichiers
  - Exemple: `TODO fix bug` trouve tous les fichiers contenant cette phrase

- **Par extension**: Utilisez des wildcards
  - Exemple: `*.rs` trouve tous les fichiers Rust

### Navigation

- `↑` / `↓` : Naviguer dans les résultats
- `Enter` : Ouvrir le fichier/dossier
- `Ctrl+L` : Ouvrir l'emplacement du fichier
- `Esc` : Fermer la fenêtre

## 🐛 Résolution de Problèmes

### L'application ne compile pas

**Erreur: "linker 'link.exe' not found"**
```bash
# Installer Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/
# Sélectionnez "C++ build tools"
```

**Erreur: "failed to run custom build command for 'openssl-sys'"**
```bash
# Installer Strawberry Perl
# https://strawberryperl.com/
```

### L'indexation est lente

Réduisez les dossiers à indexer dans `config.toml`:

```toml
[indexer]
include_paths = [
    "C:\\Users\\VotreNom\\Documents"  # Seulement Documents
]

num_threads = 4  # Réduire si CPU faible
```

### Pas de résultats

Vérifiez les logs:
```bash
RUST_LOG=debug cargo run --release
```

Vérifiez que l'indexation est terminée (message dans la console):
```
✅ Scan initial terminé avec succès
```

## 🚀 Optimisation

### Build ultra-optimisé (expérimental)

Ajoutez à `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"

# NOUVEAU: Optimisations expérimentales
[profile.release.package."*"]
opt-level = 3
```

### Activer les instructions CPU natives

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

⚠️ **Attention:** L'exécutable ne marchera que sur votre processeur !

## 📦 Créer un Package Windows

### Option 1: Simple (copier l'exe)

```bash
# Build release
cargo build --release

# Copier l'exe où vous voulez
copy target\release\spotlight_windows.exe C:\Program Files\SpotlightWindows\
```

### Option 2: Installer (MSI/EXE)

Utilisez `cargo-wix`:

```bash
cargo install cargo-wix
cargo wix init
cargo wix
```

L'installeur se trouve dans `target\wix\spotlight_windows-*.msi`

## 🔥 Mode Développement

### Compilation rapide

```bash
# Profil dev optimisé (opt-level=1)
cargo build

# Watch mode (recompile automatiquement)
cargo install cargo-watch
cargo watch -x run
```

### Tests

```bash
cargo test
```

### Linter

```bash
cargo clippy
```

### Format

```bash
cargo fmt
```

## 📊 Benchmarks

Tester les performances:

```bash
cargo build --release
time .\target\release\spotlight_windows.exe
```

Mesurer la taille:

```bash
# Avant strip
dir target\release\spotlight_windows.exe

# Après strip
strip target\release\spotlight_windows.exe
dir target\release\spotlight_windows.exe
```

## 🎓 Pour aller plus loin

### Activer toutes les features

```bash
cargo build --release --all-features
```

### Build pour différentes architectures

```bash
# x86_64 (default)
cargo build --release

# i686 (32-bit)
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

## ❓ Aide

### Documentation Rust

```bash
rustup doc
```

### Documentation du projet

```bash
cargo doc --open
```

### Communauté

- [GitHub Issues](https://github.com/fless-lab/spotlight_windows/issues)
- [GitHub Discussions](https://github.com/fless-lab/spotlight_windows/discussions)

---

**Bon build ! 🦀**
