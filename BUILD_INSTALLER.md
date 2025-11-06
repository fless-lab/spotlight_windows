# 🚀 Guide de Création de l'Installateur Spotlight Windows

Ce guide explique comment créer l'installateur professionnel `.exe` pour Spotlight Windows.

---

## 📋 Prérequis

### 1. NSIS (Nullsoft Scriptable Install System)

**Télécharger et installer NSIS** :
- 🔗 Site officiel : https://nsis.sourceforge.io/Download
- 📦 Fichier recommandé : `nsis-3.10-setup.exe` (dernière version stable)
- ⚙️ Installation par défaut : `C:\Program Files (x86)\NSIS\`

**Vérifier l'installation** :
```powershell
# Dans PowerShell ou CMD
"C:\Program Files (x86)\NSIS\makensis.exe" /VERSION
# Doit afficher: v3.10 ou supérieur
```

---

## 🛠️ Méthode 1 : Compilation Manuelle (Simple)

### Étape 1 : Compiler l'application Rust

Depuis le répertoire `spotlight_windows/` :

```bash
# Cross-compile pour Windows (depuis Linux)
cargo build --release --target x86_64-pc-windows-gnu

# OU compile nativement sur Windows
cargo build --release
```

L'exécutable sera dans :
- Linux cross-compile : `target/x86_64-pc-windows-gnu/release/spotlight_windows.exe`
- Windows natif : `target/release/spotlight_windows.exe`

### Étape 2 : Compiler l'installateur avec NSIS

**Sur Windows** :

```powershell
# Depuis le répertoire spotlight_windows/
& "C:\Program Files (x86)\NSIS\makensis.exe" installer.nsi
```

**Résultat** :
- ✅ Fichier créé : `SpotlightWindows-Setup.exe` (dans le même répertoire)
- 📦 Taille : ~12-15 MB (compressé avec LZMA)
- 🎯 Prêt à distribuer !

---

## 🚀 Méthode 2 : Script PowerShell Automatisé (Recommandé)

J'ai créé un script PowerShell qui automatise tout :

### Utilisation :

```powershell
# Dans PowerShell (Administrateur)
cd spotlight_windows
.\build_installer.ps1
```

Le script va :
1. ✅ Vérifier que NSIS est installé
2. ✅ Vérifier que l'exécutable existe
3. ✅ Compiler l'installateur
4. ✅ Afficher le résultat

---

## 📦 Résultat Final

Après compilation, vous obtiendrez :

**`SpotlightWindows-Setup.exe`**
- 🎯 Installateur Windows professionnel
- 🔒 Demande les droits administrateur
- 💾 Taille : ~12-15 MB (compressé)
- 🎨 Interface Modern UI 2
- 🇫🇷 En français (avec fallback anglais)

### Fonctionnalités de l'installateur :

✅ **Installation** :
- Installe dans `C:\Program Files\Spotlight Windows\`
- Crée les raccourcis (Bureau + Menu Démarrer)
- Ajoute au démarrage automatique Windows
- Enregistre dans le Panneau de configuration

✅ **Désinstallateur** :
- Accessible via Ajout/Suppression de programmes
- Arrête l'application avant désinstallation
- Retire du démarrage automatique
- Supprime tous les fichiers et raccourcis
- **Garde les données utilisateur** (index, config) pour réinstallation

✅ **Interface utilisateur** :
- Page de bienvenue moderne
- Sélection du répertoire d'installation
- Barre de progression
- Option "Lancer maintenant" à la fin
- Détection de version existante

---

## 🎯 Distribution

### Partager l'installateur :

1. **GitHub Releases** :
   ```bash
   # Créer une release sur GitHub
   gh release create v0.3.0 SpotlightWindows-Setup.exe --title "Spotlight Windows v0.3.0"
   ```

2. **Lien direct** :
   - Hébergez `SpotlightWindows-Setup.exe` sur votre serveur
   - Partagez le lien de téléchargement

3. **README** :
   ```markdown
   ## 📥 Installation

   1. Téléchargez [SpotlightWindows-Setup.exe](lien)
   2. Exécutez l'installateur (droits admin requis)
   3. Suivez les instructions à l'écran
   4. Spotlight Windows démarrera automatiquement !

   **Utilisation** : Appuyez sur `Ctrl+Space` pour ouvrir la recherche
   ```

---

## 🔧 Personnalisation Avancée

### Modifier la version :

Dans `installer.nsi`, ligne 22 :
```nsis
!define VERSION "0.3.0"  ; Changez ici
```

### Ajouter une icône personnalisée :

1. Créez une icône `.ico` (256x256 recommandé)
2. Placez-la dans le répertoire : `assets/icon.ico`
3. Modifiez `installer.nsi` :
   ```nsis
   !define MUI_ICON "assets\icon.ico"
   !define MUI_UNICON "assets\icon.ico"
   ```

### Ajouter une licence :

1. Créez `LICENSE.txt` dans le répertoire
2. Décommentez dans `installer.nsi` :
   ```nsis
   !insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
   ```

---

## ❓ Dépannage

### Erreur : "NSIS not found"
```powershell
# Vérifier le chemin d'installation
where.exe makensis.exe

# Si différent, mettre à jour le chemin dans build_installer.ps1
```

### Erreur : "spotlight_windows.exe not found"
```bash
# Recompiler l'application
cargo build --release --target x86_64-pc-windows-gnu
```

### Erreur : "Access denied"
```powershell
# Lancer PowerShell en Administrateur
Start-Process powershell -Verb runAs
```

---

## 📊 Comparaison des méthodes

| Méthode | Avantages | Inconvénients |
|---------|-----------|---------------|
| **Installateur NSIS** | ✅ Professionnel<br>✅ Démarrage auto<br>✅ Désinstalleur<br>✅ Panneau config | ⚠️ Nécessite NSIS |
| **Copie manuelle** | ✅ Simple<br>✅ Rapide | ❌ Pas d'auto-start<br>❌ Pas de désinstalleur |
| **Script PowerShell** | ✅ Très simple<br>✅ Automatisé | ⚠️ Nécessite NSIS |

---

## 🎉 Résultat Attendu

Après installation, l'utilisateur aura :

1. ✅ **Icône dans le system tray** (en bas à droite)
2. ✅ **Ctrl+Space** fonctionne globalement
3. ✅ **Démarrage automatique** avec Windows
4. ✅ **Raccourcis** sur Bureau et Menu Démarrer
5. ✅ **Désinstallation propre** via Panneau de configuration

---

## 📞 Support

Si vous rencontrez des problèmes :
1. Vérifiez que NSIS est bien installé
2. Vérifiez que l'exécutable `.exe` existe
3. Lancez PowerShell en administrateur
4. Consultez les logs de compilation NSIS

**Happy Installing!** 🚀
