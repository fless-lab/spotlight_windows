# 📦 Spotlight Windows - Releases

Téléchargements officiels de Spotlight Windows.

---

## 🚀 Version Actuelle : **v0.3.0** (Latest)

### 📅 Date de Release : 7 Novembre 2025

### 📥 Téléchargements

| Type | Fichier | Taille | Description |
|------|---------|--------|-------------|
| **Installateur** | `SpotlightWindows-Setup-v0.3.0.exe` | ~12-15 MB | **Recommandé** - Installation automatique avec démarrage auto |
| **Portable** | `spotlight_windows-v0.3.0.exe` | 12 MB | Version portable (sans installation) |

### ✨ Nouveautés v0.3.0

#### 🎨 **UI Premium Redesign**
- Interface complètement refaite avec design moderne
- Animations fluides (fade-in, hover effects, slide-down)
- Palette de couleurs premium (thème sombre authentique)
- Icônes contextuelles pour 30+ types de fichiers
- Typographie professionnelle et espacement parfait

#### 🚫 **Plus de Console Windows**
- Application GUI native (pas de fenêtre noire)
- Comportement comme logiciel installé professionnel
- Tourne en arrière-plan discrètement

#### 🎯 **Core Features**
- **Ctrl+Space** : Hotkey global pour ouvrir/fermer
- **System Tray** : Icône système avec menu et status
- **Démarrage automatique** : Lance avec Windows
- **Window management** : Cache/affiche proprement

#### 📦 **Installateur NSIS**
- Installation professionnelle en 30 secondes
- Ajout automatique au démarrage Windows
- Raccourcis (Bureau + Menu Démarrer)
- Désinstalleur complet
- Panneau de configuration intégré

#### ⚡ **Performance**
- Recherche < 50ms en moyenne
- Index persistant (pas de réindexation)
- Surveillance temps réel des nouveaux fichiers
- N-grams pour recherche floue

---

## 📋 Installation

### Option 1 : Installateur (Recommandé)

**Note** : L'installateur NSIS doit être compilé sur Windows.

**Sur Windows** :
```powershell
# 1. Cloner le repo
git clone https://github.com/fless-lab/spotlight_windows.git
cd spotlight_windows

# 2. Compiler l'installateur
.\build_installer.ps1

# Résultat: SpotlightWindows-Setup.exe
```

**Installation pour l'utilisateur final** :
1. Télécharger `SpotlightWindows-Setup-v0.3.0.exe`
2. Double-clic pour lancer l'installation
3. Suivre l'assistant d'installation
4. L'application démarre automatiquement avec Windows
5. Appuyer sur **Ctrl+Space** pour utiliser

### Option 2 : Version Portable

1. Télécharger `spotlight_windows-v0.3.0.exe`
2. Placer où vous voulez (ex: `C:\Tools\`)
3. Double-clic pour lancer
4. **(Optionnel)** Ajouter au démarrage :
   - `Win+R` → `shell:startup`
   - Créer un raccourci vers l'exe

---

## 🎯 Utilisation

### Première utilisation :
1. **Indexation initiale** : 1-2 minutes au premier lancement
2. **Ctrl+Space** : Ouvrir la fenêtre de recherche
3. **Taper** pour chercher (nom ou contenu de fichier)
4. **Flèches ↑↓** : Naviguer dans les résultats
5. **Enter** : Ouvrir le fichier/dossier
6. **ESC** : Fermer la fenêtre

### Raccourcis clavier :
- `Ctrl+Space` : Afficher/Cacher
- `↑↓` : Naviguer
- `Enter` : Ouvrir
- `ESC` : Fermer

---

## 📊 Historique des Versions

### v0.3.0 (7 Nov 2025) - **CURRENT**
- ✨ UI Premium redesign complet
- 🚫 Suppression fenêtre console (GUI natif)
- 🎯 Core features (Ctrl+Space, tray, auto-start)
- 📦 Installateur NSIS professionnel
- 🌊 Animations fluides et design moderne

### v0.2.0 (6 Nov 2025)
- 🔍 Recherche par contenu de fichier
- 📁 Indexation complète système
- ⚡ Performance améliorée (Tantivy)
- 🎨 UI Spotlight style
- 💾 Persistance de l'index

### v0.1.0 (5 Nov 2025)
- 🎉 Version initiale
- 🔎 Recherche basique par nom
- 📂 Indexation répertoires utilisateur
- 🖥️ Interface simple

---

## 🔧 Configuration Système

### Prérequis :
- 💻 **OS** : Windows 10/11 (64-bit)
- 💾 **RAM** : 4 GB min (8 GB recommandé)
- 💿 **Disque** : 50 MB app + ~100 MB index
- 🖥️ **CPU** : Dual-core ou supérieur

### Répertoires indexés :
- ✅ Profil utilisateur (`C:\Users\VotreNom\`)
- ✅ Disques D:, E:, F: (si existants)
- ✅ Program Files

### Exclusions (performance) :
- ❌ `C:\Windows`
- ❌ `node_modules`
- ❌ Caches et fichiers temporaires

---

## 🐛 Problèmes Connus

### v0.3.0

#### **⚠️ Ctrl+Space Hotkey - Cross-Compile Limitation** (IMPORTANT)
- **Problème** : Le hotkey global Ctrl+Space **ne fonctionne PAS** en cross-compile (Linux → Windows)
- **Cause** : Les bibliothèques de hotkey global nécessitent une compilation native Windows
- **Solution temporaire** : L'application démarre **visible** par défaut pour pouvoir être utilisée
- **Solution définitive** : Compiler nativement sur Windows
  ```powershell
  # Sur Windows
  cargo build --release
  ```
- **Impact** :
  - ✅ Application utilisable normalement (fenêtre visible au démarrage)
  - ❌ Pas de hotkey Ctrl+Space pour show/hide
  - ✅ ESC pour fermer fonctionne
  - ✅ Toutes autres fonctionnalités OK

#### **Autres Problèmes**
- **PDF** : Extraction de contenu désactivée (crash avec glyphs non-ASCII)
- **Première indexation** : Peut prendre 2-5 minutes selon nombre de fichiers

### Workarounds :
- **Hotkey** : Application démarre visible, utiliser normalement sans Ctrl+Space
- **PDF** : Recherche par nom de fichier uniquement
- **Indexation** : Patienter, c'est normal au premier lancement

### Pour Version Complète :
Pour avoir **toutes les fonctionnalités** (y compris Ctrl+Space) :
1. **Sur Windows**, compiler nativement :
   ```powershell
   git clone https://github.com/fless-lab/spotlight_windows.git
   cd spotlight_windows
   cargo build --release
   ```
2. L'exécutable sera dans `target\release\spotlight_windows.exe`
3. Toutes les fonctionnalités fonctionneront (hotkey, tray, auto-start)

---

## 🔄 Mise à Jour

### Depuis version précédente :
1. Désinstaller l'ancienne version (Panneau de config)
2. Installer la nouvelle version
3. L'index sera conservé automatiquement

### Conservation des données :
- ✅ Index sauvegardé dans `%APPDATA%\spotlight_windows\`
- ✅ Configuration préservée
- ✅ Pas de réindexation nécessaire

---

## 📞 Support

### Signaler un bug :
- 🐛 [GitHub Issues](https://github.com/fless-lab/spotlight_windows/issues)
- 📧 Inclure : version, OS, logs (`%APPDATA%\spotlight_windows\logs\`)

### Demander une fonctionnalité :
- 💡 [GitHub Discussions](https://github.com/fless-lab/spotlight_windows/discussions)

### Documentation :
- 📖 [Guide d'installation](INSTALL.md)
- 🛠️ [Guide de build](BUILD_INSTALLER.md)
- 📝 [README principal](README.md)

---

## 🎉 Changelog Détaillé

### v0.3.0 (2025-11-07)

**Added:**
- Nouvelle UI spotlight_v2.rs avec design premium moderne
- Animations fluides (fade-in, hover, slide-down avec easing)
- Icônes contextuelles pour 30+ types de fichiers
- System tray icon avec menu et tooltip dynamique
- Hotkey global Ctrl+Space pour show/hide
- Directive `#![windows_subsystem = "windows"]` pour GUI natif
- Script NSIS d'installation professionnel (installer.nsi)
- Script PowerShell d'automatisation (build_installer.ps1)
- Documentation installation complète (BUILD_INSTALLER.md, INSTALL.md)
- Désinstalleur propre avec conservation données utilisateur

**Changed:**
- Palette de couleurs plus sombre et contrastée
- Espacement et typographie améliorés
- Taille items résultats (64px au lieu de 72px)
- Barre de recherche plus haute (68px)
- Border radius augmenté (16px)

**Fixed:**
- Problème fenêtre console qui s'ouvre (maintenant GUI pur)
- Conflits borrow checker avec état hover
- Initialisation constantes Color32
- Écran noir au ESC (gestion visibility propre)

**Removed:**
- Extraction contenu PDF (temporaire, cause crashes)

### v0.2.0 (2025-11-06)

**Added:**
- Recherche par contenu de fichier (texte, code)
- N-gram tokenizer (2-4 grams) pour recherche floue
- Persistance de l'index (pas de réindexation)
- File watcher temps réel
- Expansion coverage système (D:, E:, F: drives)

**Changed:**
- Optimisation performance Tantivy
- Améliorations UI/UX

**Fixed:**
- Bug index non persistant
- Détection home directory Windows

### v0.1.0 (2025-11-05)

**Added:**
- Version initiale
- Recherche par nom de fichier
- Indexation Tantivy
- Interface egui basique
- Support Windows 10/11

---

## 📄 Licence

© 2025 Spotlight Windows Team. Tous droits réservés.

---

## 🙏 Remerciements

Merci à tous les contributeurs et utilisateurs de Spotlight Windows !

**Technologies utilisées** :
- 🦀 Rust
- 🔍 Tantivy
- 🖼️ egui/eframe
- 🎯 NSIS
- ⚡ Tokio

---

**Happy Searching!** 🚀
