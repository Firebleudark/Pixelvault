@echo off
chcp 65001 > nul
cd /d "%~dp0"

echo ========================================================
echo    🔐  PIXELVAULT - Windows Terminal
echo ========================================================

:CHECK_BINARY
if exist "target\release\pixelvault.exe" goto FOUND
echo ⚠️  Binaire introuvable. Tentative de compilation...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Erreur de compilation.
    pause
    exit /b %ERRORLEVEL%
)

:FOUND
REM Add release folder to PATH for this session
set "PATH=%CD%\target\release;%PATH%"

echo.
echo Le programme est prêt.
echo Commandes disponibles :
echo   pixelvault init     - Créer un nouveau coffre
echo   pixelvault add      - Ajouter un mot de passe
echo   pixelvault get      - Récupérer un mot de passe
echo   pixelvault list     - Voir les entrées
echo   pixelvault --help   - Voir toute l'aide
echo.
echo Tapez vos commandes ci-dessous.
echo ========================================================
echo.

REM Open a new command prompt that stays open
cmd /k
