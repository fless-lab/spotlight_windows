# ===================================
# Script PowerShell : Création Installateur Spotlight Windows
# ===================================
# Utilisation: .\build_installer.ps1
# ===================================

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║    Spotlight Windows - Générateur d'Installateur NSIS     ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ===================================
# Configuration
# ===================================
$NSIS_PATH = "C:\Program Files (x86)\NSIS\makensis.exe"
$NSIS_PATH_ALT = "C:\Program Files\NSIS\makensis.exe"
$SCRIPT_PATH = "installer.nsi"
$EXE_PATH_CROSS = "target\x86_64-pc-windows-gnu\release\spotlight_windows.exe"
$EXE_PATH_NATIVE = "target\release\spotlight_windows.exe"
$OUTPUT_NAME = "SpotlightWindows-Setup.exe"

# ===================================
# Fonction : Vérifier un chemin
# ===================================
function Test-PathExists {
    param([string]$Path, [string]$Name)

    if (Test-Path $Path) {
        Write-Host "[✓] $Name trouvé : $Path" -ForegroundColor Green
        return $true
    } else {
        Write-Host "[✗] $Name introuvable : $Path" -ForegroundColor Red
        return $false
    }
}

# ===================================
# 1. Vérifier NSIS
# ===================================
Write-Host "Étape 1 : Vérification de NSIS..." -ForegroundColor Yellow

$nsisFound = $false
$nsisExe = ""

if (Test-PathExists $NSIS_PATH "NSIS (x86)") {
    $nsisFound = $true
    $nsisExe = $NSIS_PATH
} elseif (Test-PathExists $NSIS_PATH_ALT "NSIS (x64)") {
    $nsisFound = $true
    $nsisExe = $NSIS_PATH_ALT
}

if (-not $nsisFound) {
    Write-Host ""
    Write-Host "❌ ERREUR : NSIS n'est pas installé !" -ForegroundColor Red
    Write-Host ""
    Write-Host "Téléchargez NSIS depuis : https://nsis.sourceforge.io/Download" -ForegroundColor Cyan
    Write-Host "Installez-le, puis relancez ce script." -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

# Afficher la version de NSIS
$nsisVersion = & $nsisExe /VERSION 2>&1
Write-Host "[i] Version NSIS : $nsisVersion" -ForegroundColor Cyan
Write-Host ""

# ===================================
# 2. Vérifier le script NSIS
# ===================================
Write-Host "Étape 2 : Vérification du script installer.nsi..." -ForegroundColor Yellow

if (-not (Test-PathExists $SCRIPT_PATH "Script NSIS")) {
    Write-Host ""
    Write-Host "❌ ERREUR : Le fichier installer.nsi est introuvable !" -ForegroundColor Red
    Write-Host "Assurez-vous d'être dans le bon répertoire." -ForegroundColor Cyan
    Write-Host ""
    exit 1
}
Write-Host ""

# ===================================
# 3. Vérifier l'exécutable
# ===================================
Write-Host "Étape 3 : Vérification de l'exécutable..." -ForegroundColor Yellow

$exeFound = $false
$exePath = ""

if (Test-PathExists $EXE_PATH_CROSS "Exécutable (cross-compile)") {
    $exeFound = $true
    $exePath = $EXE_PATH_CROSS
} elseif (Test-PathExists $EXE_PATH_NATIVE "Exécutable (natif)") {
    $exeFound = $true
    $exePath = $EXE_PATH_NATIVE
}

if (-not $exeFound) {
    Write-Host ""
    Write-Host "❌ ERREUR : spotlight_windows.exe introuvable !" -ForegroundColor Red
    Write-Host ""
    Write-Host "Compilez d'abord l'application avec :" -ForegroundColor Cyan
    Write-Host "  cargo build --release --target x86_64-pc-windows-gnu" -ForegroundColor Yellow
    Write-Host "OU" -ForegroundColor Cyan
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

# Afficher la taille du fichier
$exeSize = (Get-Item $exePath).Length / 1MB
Write-Host "[i] Taille de l'exécutable : $([math]::Round($exeSize, 2)) MB" -ForegroundColor Cyan
Write-Host ""

# ===================================
# 4. Compiler l'installateur
# ===================================
Write-Host "Étape 4 : Compilation de l'installateur NSIS..." -ForegroundColor Yellow
Write-Host ""

# Supprimer l'ancien installateur s'il existe
if (Test-Path $OUTPUT_NAME) {
    Write-Host "[i] Suppression de l'ancien installateur..." -ForegroundColor Cyan
    Remove-Item $OUTPUT_NAME -Force
}

# Compiler avec NSIS
Write-Host "[>] Exécution de makensis..." -ForegroundColor Cyan
& $nsisExe $SCRIPT_PATH

$exitCode = $LASTEXITCODE

Write-Host ""

# ===================================
# 5. Vérifier le résultat
# ===================================
if ($exitCode -eq 0 -and (Test-Path $OUTPUT_NAME)) {
    $installerSize = (Get-Item $OUTPUT_NAME).Length / 1MB

    Write-Host ""
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║              ✓ INSTALLATEUR CRÉÉ AVEC SUCCÈS !            ║" -ForegroundColor Green
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "Fichier créé : $OUTPUT_NAME" -ForegroundColor Cyan
    Write-Host "Taille       : $([math]::Round($installerSize, 2)) MB" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Vous pouvez maintenant distribuer cet installateur !" -ForegroundColor Green
    Write-Host ""

    # Proposer d'ouvrir le répertoire
    $response = Read-Host "Voulez-vous ouvrir le répertoire ? (O/N)"
    if ($response -eq "O" -or $response -eq "o" -or $response -eq "Y" -or $response -eq "y") {
        explorer.exe .
    }

} else {
    Write-Host ""
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Red
    Write-Host "║              ✗ ERREUR LORS DE LA COMPILATION              ║" -ForegroundColor Red
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Red
    Write-Host ""
    Write-Host "Code de sortie : $exitCode" -ForegroundColor Red
    Write-Host "Consultez les messages d'erreur ci-dessus." -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

# ===================================
# 6. Instructions finales
# ===================================
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    PROCHAINES ÉTAPES                       ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Testez l'installateur :" -ForegroundColor Yellow
Write-Host "   .\$OUTPUT_NAME" -ForegroundColor White
Write-Host ""
Write-Host "2. Distribuez l'installateur :" -ForegroundColor Yellow
Write-Host "   - GitHub Releases" -ForegroundColor White
Write-Host "   - Site web de téléchargement" -ForegroundColor White
Write-Host "   - Partage direct" -ForegroundColor White
Write-Host ""
Write-Host "3. L'utilisateur final :" -ForegroundColor Yellow
Write-Host "   - Double-clic sur l'installateur" -ForegroundColor White
Write-Host "   - Suivre les instructions" -ForegroundColor White
Write-Host "   - Appuyer sur Ctrl+Space pour utiliser" -ForegroundColor White
Write-Host ""
Write-Host "Happy Installing! 🚀" -ForegroundColor Green
Write-Host ""
