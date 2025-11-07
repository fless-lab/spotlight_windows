# 📦 Spotlight Windows - Distribution Files

Ce répertoire contient les fichiers de distribution prêts à être partagés avec les utilisateurs.

---

## 📥 Fichiers Disponibles

### **spotlight_windows-v0.3.0.exe**
- **Type** : Version portable
- **Taille** : 12 MB
- **Description** : Exécutable Windows standalone (pas d'installation requise)
- **Usage** : Double-clic pour lancer directement

---

## 🚀 Pour les Utilisateurs Finaux

### Installation Rapide (Portable) :

1. **Télécharger** `spotlight_windows-v0.3.0.exe`
2. **Copier** dans un dossier de votre choix (ex: `C:\Tools\`)
3. **Lancer** en double-cliquant
4. **Utiliser** avec **Ctrl+Space**

### Installation Complète (Avec Installateur) :

**Note** : L'installateur NSIS doit être compilé sur une machine Windows.

1. **Sur Windows**, depuis la racine du projet :
   ```powershell
   .\build_installer.ps1
   ```

2. Cela créera `SpotlightWindows-Setup.exe` (~12-15 MB)

3. **Distribuer** ce fichier aux utilisateurs

4. **L'utilisateur** :
   - Double-clic sur l'installateur
   - Suit l'assistant d'installation
   - L'app démarre automatiquement avec Windows

---

## 🛠️ Pour les Développeurs

### Créer une Nouvelle Release :

#### Étape 1 : Compiler l'Application

```bash
# Cross-compile depuis Linux
cargo build --release --target x86_64-pc-windows-gnu

# OU compile nativement sur Windows
cargo build --release
```

#### Étape 2 : Copier dans dist/

```bash
# Depuis la racine du projet
cp target/x86_64-pc-windows-gnu/release/spotlight_windows.exe \
   bin/dist/spotlight_windows-vX.Y.Z.exe
```

#### Étape 3 : Créer l'Installateur (Windows uniquement)

```powershell
# Sur Windows
.\build_installer.ps1

# Renommer le résultat
mv SpotlightWindows-Setup.exe \
   bin/dist/SpotlightWindows-Setup-vX.Y.Z.exe
```

#### Étape 4 : Mettre à Jour la Documentation

```bash
# Éditer RELEASES.md avec la nouvelle version
# Éditer README.md avec le lien de téléchargement
```

#### Étape 5 : Créer un Git Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

#### Étape 6 : Créer une GitHub Release

```bash
# Avec GitHub CLI
gh release create vX.Y.Z \
  bin/dist/spotlight_windows-vX.Y.Z.exe \
  bin/dist/SpotlightWindows-Setup-vX.Y.Z.exe \
  --title "Spotlight Windows vX.Y.Z" \
  --notes "Voir RELEASES.md pour les détails"
```

---

## 📊 Structure des Versions

### Nomenclature des fichiers :

- **Portable** : `spotlight_windows-vX.Y.Z.exe`
  - Exemple : `spotlight_windows-v0.3.0.exe`

- **Installateur** : `SpotlightWindows-Setup-vX.Y.Z.exe`
  - Exemple : `SpotlightWindows-Setup-v0.3.0.exe`

### Versioning Sémantique :

- **X** : Version majeure (breaking changes)
- **Y** : Version mineure (nouvelles fonctionnalités)
- **Z** : Version patch (bug fixes)

**Exemple** : v0.3.0
- Major: 0 (pre-release)
- Minor: 3 (nouvelles features majeures)
- Patch: 0 (première release de cette minor)

---

## 🔒 Checksums (à venir)

Pour vérifier l'intégrité des téléchargements :

```bash
# Générer SHA256
sha256sum spotlight_windows-v0.3.0.exe > checksums.txt
```

Les checksums seront publiés avec chaque release.

---

## 📦 Contenu des Releases

### Version Portable (.exe)
- ✅ Application standalone
- ✅ Pas d'installation requise
- ✅ Données dans `%APPDATA%\spotlight_windows\`
- ❌ Pas de démarrage automatique (manuel)
- ❌ Pas de raccourcis (à créer manuellement)

### Version Installateur (-Setup.exe)
- ✅ Installation guidée
- ✅ Démarrage automatique avec Windows
- ✅ Raccourcis (Bureau + Menu Démarrer)
- ✅ Enregistrement Panneau de config
- ✅ Désinstalleur inclus
- ✅ Installation propre dans Program Files

---

## 🎯 Distribution

### Canaux de Distribution :

1. **GitHub Releases** (Recommandé)
   - Hébergement gratuit
   - Traçabilité des versions
   - Changelog intégré

2. **Site Web de Téléchargement**
   - Lien direct vers les .exe
   - Page de présentation

3. **Package Managers** (Future)
   - Chocolatey (`choco install spotlight-windows`)
   - Winget (`winget install spotlight-windows`)

---

## 📈 Statistiques de Release

| Version | Date | Taille | Téléchargements |
|---------|------|--------|-----------------|
| v0.3.0 | 2025-11-07 | 12 MB | - |
| v0.2.0 | 2025-11-06 | 11 MB | - |
| v0.1.0 | 2025-11-05 | 10 MB | - |

---

## 🧪 Tests Pré-Release

Avant de publier une release, tester :

### Tests Fonctionnels :
- ✅ Application lance sans erreur
- ✅ Ctrl+Space fonctionne (compilation native Windows)
- ✅ System tray icon apparaît
- ✅ Recherche retourne des résultats
- ✅ Ouverture de fichiers fonctionne
- ✅ ESC ferme la fenêtre

### Tests d'Installation (Installateur) :
- ✅ Installation réussie
- ✅ Raccourcis créés
- ✅ Démarrage automatique configuré
- ✅ Panneau de config : entrée présente
- ✅ Désinstallation propre

### Tests de Performance :
- ✅ Indexation initiale < 5 minutes
- ✅ Recherche < 100ms
- ✅ RAM < 200 MB au repos
- ✅ CPU < 5% au repos

---

## 🐛 Rapport de Bugs

Si des utilisateurs signalent des bugs :

1. Collecter les informations :
   - Version exacte
   - OS et version Windows
   - Logs (`%APPDATA%\spotlight_windows\logs\`)

2. Reproduire le bug

3. Fixer et créer une version patch

4. Publier rapidement (hot-fix)

---

## 📝 Notes de Release

### Checklist avant publication :

- [ ] Code compilé et testé
- [ ] Version incrémentée dans `Cargo.toml`
- [ ] `RELEASES.md` mis à jour
- [ ] `README.md` mis à jour avec lien de téléchargement
- [ ] Fichiers copiés dans `bin/dist/`
- [ ] Installateur compilé (Windows)
- [ ] Tests fonctionnels passés
- [ ] Git tag créé
- [ ] GitHub Release créée
- [ ] Documentation à jour

---

## 🎉 Publication

Une fois tout vérifié :

```bash
# 1. Commit et tag
git add .
git commit -m "chore: Release vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main
git push origin vX.Y.Z

# 2. GitHub Release
gh release create vX.Y.Z \
  bin/dist/*.exe \
  --title "Spotlight Windows vX.Y.Z" \
  --notes-file RELEASES.md

# 3. Annoncer
# - Twitter/X
# - Reddit
# - Discord
# - Blog post
```

---

**Happy Releasing!** 🚀
