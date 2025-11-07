# 🔍 Spotlight Windows

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](RELEASES.md)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **La puissance de macOS Spotlight, native sur Windows**

Application de recherche desktop ultra-rapide pour Windows, avec UI premium moderne, animations fluides, et installation professionnelle.

![Spotlight Windows Demo](assets/demo.gif)

---

## ✨ Fonctionnalités v0.3.0

### 🎨 **UI Premium Redesign**
- Interface complètement refaite avec design moderne
- Animations fluides (fade-in, hover effects, slide-down)
- Palette de couleurs sombre authentique
- Icônes contextuelles pour 30+ types de fichiers
- Typographie professionnelle et espacement parfait

### ⚡ **Recherche Ultra-Rapide**
- **< 50ms** en moyenne grâce à Tantivy
- **Recherche par nom** (instantanée)
- **Recherche par contenu** (dans les fichiers texte)
- **N-grams** (trouve "doc" dans "documents")
- **Fuzzy matching** (tolère les fautes de frappe)
- **Index persistant** (pas de réindexation au redémarrage)

### 🎯 **Core Features**
- **Ctrl+Space** : Hotkey global pour ouvrir/fermer
- **System Tray** : Icône système avec menu et status
- **Démarrage automatique** : Lance avec Windows
- **GUI natif** : Pas de fenêtre console qui s'ouvre
- **Surveillance temps réel** : Détecte nouveaux fichiers automatiquement

### 📦 **Installation Professionnelle**
- Installateur NSIS avec interface guidée
- Installation en 30 secondes
- Raccourcis automatiques (Bureau + Menu Démarrer)
- Désinstalleur complet
- Intégration Panneau de configuration

---

## 📥 Téléchargement

### Version Actuelle : **v0.3.0** (7 Novembre 2025)

#### **Option 1 : Installateur Windows (Recommandé)**

**⚠️ Note** : L'installateur NSIS doit être compilé sur Windows. Suivez ces étapes :

```powershell
# 1. Cloner le repository
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows

# 2. Compiler l'installateur (nécessite NSIS installé)
.\build_installer.ps1

# Résultat: SpotlightWindows-Setup.exe
```

**Installation** :
1. Double-clic sur `SpotlightWindows-Setup.exe`
2. Suivre l'assistant d'installation
3. L'application démarre automatiquement avec Windows
4. Appuyer sur **Ctrl+Space** pour utiliser

#### **Option 2 : Version Portable**

Téléchargez directement l'exécutable :

**Windows (x86_64)** :
```powershell
# Télécharger depuis bin/dist/
Invoke-WebRequest -Uri "https://github.com/fless-lab/spotlight_windows/raw/claude/rust-desktop-spotlight-search-011CUoqZj9PhKNLXWzyaLPJn/bin/dist/spotlight_windows-v0.3.0.exe" -OutFile "spotlight_windows.exe"

# Lancer
.\spotlight_windows.exe
```

**Ou via le navigateur** :
- 📦 [spotlight_windows-v0.3.0.exe](bin/dist/spotlight_windows-v0.3.0.exe) (12 MB)

**Configuration portable** :
- L'application stocke ses données dans `%APPDATA%\spotlight_windows\`
- Pour démarrage automatique : créer un raccourci dans `shell:startup`

---

## 🚀 Utilisation Rapide

### Premier Lancement
1. **Indexation initiale** : 1-2 minutes (selon nombre de fichiers)
2. **Icône système** apparaît en bas à droite
3. **Tooltip** indique le nombre de fichiers indexés

### Raccourcis Clavier
- **Ctrl+Space** : Ouvrir/Fermer la recherche
- **↑↓** : Naviguer dans les résultats
- **Enter** : Ouvrir le fichier/dossier sélectionné
- **ESC** : Fermer la fenêtre

### Recherche
- **Par nom** : `document`
- **Par contenu** : `TODO refactor` (cherche dans le contenu)
- **Sous-chaîne** : `doc` trouve "documents", "my-docs", etc.
- **Fuzzy** : `dcumnt` trouve "document"

---

## 📊 Performances

Tests sur PC moderne (SSD NVMe, 16GB RAM) :

| Métrique | Valeur |
|----------|--------|
| **Indexation** | 152,000 fichiers en 101 secondes |
| **Recherche** | < 50ms en moyenne |
| **RAM** | ~150 MB au repos |
| **CPU** | < 3% au repos |
| **Taille index** | ~100 MB pour 100,000 fichiers |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Spotlight Windows v0.3.0                   │
├─────────────────────────────────────────────────────────────┤
│  UI Layer (egui) - SpotlightUI Premium                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • Animations fluides (fade-in, hover, slide-down)   │  │
│  │  • Design moderne (dark theme authentique)           │  │
│  │  • Icônes contextuelles (30+ types)                  │  │
│  │  • Navigation clavier optimisée                      │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Core Features                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  System Tray    │  Global Hotkey  │  Auto-start      │  │
│  │  ─────────────  │  ──────────────  │  ─────────────   │  │
│  │  • Menu         │  • Ctrl+Space   │  • Registry      │  │
│  │  • Status       │  • Show/Hide    │  • Startup       │  │
│  │  • Icon         │  • Toggle       │  • Background    │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Search Engine (Tantivy + Cache)                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • N-gram tokenizer (2-4 grams)                       │  │
│  │  • Fuzzy matching + scoring                           │  │
│  │  • Content search (30+ formats)                       │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Indexer (Tantivy + File Watcher)                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Scanner          │  Watcher      │  Index Manager    │  │
│  │  ───────────────  │  ──────────   │  ──────────────   │  │
│  │  • Async scan     │  • notify     │  • Persistent     │  │
│  │  • Multi-thread   │  • Real-time  │  • BM25 scoring   │  │
│  │  • Smart exclude  │  • Low CPU    │  • Incremental    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 Configuration

### Répertoires Indexés par Défaut
- ✅ Profil utilisateur complet (`C:\Users\VotreNom\`)
- ✅ Disques secondaires (D:, E:, F: si existants)
- ✅ Program Files

### Exclusions Intelligentes (Performance)
- ❌ `C:\Windows`
- ❌ `node_modules`
- ❌ `.git`
- ❌ Caches et fichiers temporaires

### Formats Supportés (Recherche Contenu)
- 📝 Texte : `.txt`, `.md`, `.log`
- 💻 Code : `.rs`, `.py`, `.js`, `.ts`, `.java`, `.cpp`, `.go`, etc.
- 🌐 Web : `.html`, `.css`, `.json`, `.xml`, `.yaml`
- 📋 Config : `.toml`, `.ini`, `.conf`
- **30+ formats** au total

---

## 🛠️ Build depuis les Sources

### Prérequis
- **Rust 1.70+** ([installer](https://rustup.rs/))
- **Windows 10/11** (64-bit) pour build natif
- **Linux** pour cross-compilation

### Build
```bash
# Cloner
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows

# Build release (optimisé)
cargo build --release --target x86_64-pc-windows-gnu

# Résultat: target/x86_64-pc-windows-gnu/release/spotlight_windows.exe
```

### Créer l'Installateur (Windows uniquement)
```powershell
# 1. Installer NSIS depuis https://nsis.sourceforge.io/

# 2. Compiler l'installateur
.\build_installer.ps1

# Résultat: SpotlightWindows-Setup.exe (~12-15 MB)
```

📖 **Documentation complète** : [BUILD_INSTALLER.md](BUILD_INSTALLER.md)

---

## 📚 Documentation

### Pour les Utilisateurs
- 📥 [Guide d'installation](INSTALL.md)
- 📦 [Releases et changelog](RELEASES.md)
- ❓ [FAQ et dépannage](INSTALL.md#dépannage)

### Pour les Développeurs
- 🛠️ [Guide de build installateur](BUILD_INSTALLER.md)
- 📂 [Structure du projet](#structure-du-projet)
- 🤝 [Contribution](#contribution)

---

## 🗺️ Roadmap

### ✅ Version 0.3.0 (Actuelle)
- [x] UI Premium redesign complet
- [x] Animations fluides
- [x] System tray icon + menu
- [x] Ctrl+Space hotkey global
- [x] Démarrage automatique
- [x] Installateur NSIS professionnel
- [x] GUI natif (pas de console)

### 🔜 Version 0.4.0 (Prochaine)
- [ ] Recherche contenu PDF (fix crash glyphs)
- [ ] Preview fichiers (images, texte)
- [ ] Calculatrice intégrée
- [ ] Effet blur acrylic Windows natif
- [ ] Amélioration animations

### 🎯 Version 1.0.0
- [ ] Indexation MFT (comme Everything)
- [ ] Recherche sémantique (ML)
- [ ] Multi-langues
- [ ] Plugins/Extensions
- [ ] Thèmes personnalisables

---

## 📂 Structure du Projet

```
spotlight_windows/
├── src/
│   ├── main.rs                  # Entry point + subsystem config
│   ├── config.rs                # Configuration
│   ├── tray.rs                  # System tray icon
│   ├── indexer/
│   │   ├── mod.rs               # Tantivy index manager
│   │   ├── scanner.rs           # Async file scanner
│   │   └── watcher.rs           # Real-time file watcher
│   ├── search/
│   │   └── mod.rs               # Search engine
│   └── ui/
│       ├── mod.rs               # UI module
│       ├── spotlight.rs         # Legacy UI
│       ├── spotlight_v2.rs      # Premium UI ✨
│       └── theme.rs             # Theme config
├── bin/
│   └── dist/
│       ├── spotlight_windows-v0.3.0.exe  # Portable ✨
│       └── README.md            # Distribution docs
├── installer.nsi                # NSIS script
├── build_installer.ps1          # PowerShell automation
├── BUILD_INSTALLER.md           # Build guide
├── INSTALL.md                   # User guide
├── RELEASES.md                  # Releases & changelog ✨
├── Cargo.toml                   # Dependencies
└── README.md                    # This file
```

---

## 🤝 Contribution

Les contributions sont bienvenues !

### Comment Contribuer
1. **Fork** le projet
2. **Créer une branche** : `git checkout -b feature/AmazingFeature`
3. **Commit** : `git commit -m 'feat: Add AmazingFeature'`
4. **Push** : `git push origin feature/AmazingFeature`
5. **Pull Request**

### Conventions
- **Commits** : Convention Conventional Commits
- **Code** : `cargo fmt` + `cargo clippy`
- **Tests** : `cargo test`

---

## 🙏 Remerciements

Ce projet s'inspire de :
- **macOS Spotlight** - Pour l'UX
- **Everything** - Pour la vitesse
- **Rust** - Pour la performance

### Technologies Utilisées
- 🔍 [Tantivy](https://github.com/quickwit-oss/tantivy) - Search engine
- 🖼️ [egui](https://github.com/emilk/egui) - UI framework
- ⚡ [Tokio](https://tokio.rs/) - Async runtime
- 🎯 [global-hotkey](https://github.com/tauri-apps/global-hotkey) - Hotkey
- 🔔 [tray-icon](https://github.com/tauri-apps/tray-icon) - System tray
- 📦 [NSIS](https://nsis.sourceforge.io/) - Installer

---

## 📝 Licence

Ce projet est sous licence MIT. Voir [LICENSE](LICENSE) pour détails.

---

## 💬 Support

### Signaler un Bug
- 🐛 [GitHub Issues](https://github.com/fless-lab/spotlight_windows/issues)
- 📧 Inclure : version, OS, logs (`%APPDATA%\spotlight_windows\logs\`)

### Demander une Fonctionnalité
- 💡 [GitHub Discussions](https://github.com/fless-lab/spotlight_windows/discussions)

### Communauté
- 💬 [Discord](https://discord.gg/spotlight-windows) (à venir)
- 🐦 [Twitter](https://twitter.com/spotlight_win) (à venir)

---

<p align="center">
  <b>Spotlight Windows v0.3.0</b><br>
  Fait avec ❤️ et 🦀 Rust<br>
  <br>
  <a href="RELEASES.md">📦 Télécharger</a> •
  <a href="INSTALL.md">📚 Documentation</a> •
  <a href="https://github.com/fless-lab/spotlight_windows/issues">🐛 Bugs</a>
</p>
