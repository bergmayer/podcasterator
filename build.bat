@echo off
setlocal enabledelayedexpansion

REM Build script for Podcasterator (Windows)
REM
REM Usage:
REM   build.bat              Build raw binary only
REM   build.bat --makebundle Build Windows installer bundles
REM   build.bat --clean      Remove all build artifacts

set "MAKEBUNDLE=false"
set "CLEAN=false"
for %%a in (%*) do (
    if "%%a"=="--makebundle" (
        set "MAKEBUNDLE=true"
    ) else if "%%a"=="--clean" (
        set "CLEAN=true"
    ) else (
        echo Unknown option: %%a
        echo Usage: build.bat [--makebundle] [--clean]
        exit /b 1
    )
)

cd /d "%~dp0"

REM --- Clean ---

if "%CLEAN%"=="true" (
    echo Cleaning build artifacts...
    if exist src-tauri\target rmdir /s /q src-tauri\target
    if exist node_modules rmdir /s /q node_modules
    if exist dist rmdir /s /q dist
    echo Clean complete.
    exit /b 0
)

REM --- Dependency checks ---

set "MISSING="
set "HAS_ERRORS=false"

where rustc >nul 2>&1
if errorlevel 1 (
    set "MISSING=!MISSING!  - Rust (install from https://rustup.rs/)"
    set "MISSING=!MISSING!
"
    set "HAS_ERRORS=true"
)

where cargo >nul 2>&1
if errorlevel 1 (
    set "MISSING=!MISSING!  - Cargo (install from https://rustup.rs/)"
    set "MISSING=!MISSING!
"
    set "HAS_ERRORS=true"
)

where node >nul 2>&1
if errorlevel 1 (
    set "MISSING=!MISSING!  - Node.js (install from https://nodejs.org/)"
    set "MISSING=!MISSING!
"
    set "HAS_ERRORS=true"
)

where npm >nul 2>&1
if errorlevel 1 (
    set "MISSING=!MISSING!  - npm (install from https://nodejs.org/)"
    set "MISSING=!MISSING!
"
    set "HAS_ERRORS=true"
)

if "%HAS_ERRORS%"=="true" (
    echo ERROR: Missing required dependencies:
    echo !MISSING!
    exit /b 1
)

REM --- Build ---

echo Installing npm dependencies...
call npm install || exit /b 1

if "%MAKEBUNDLE%"=="true" (
    echo Building Windows bundles...
    call npm run tauri build -- --bundles nsis,msi || exit /b 1
    echo.
    echo Build complete!
    echo.
    echo Bundles located at:
    echo   src-tauri\target\release\bundle\nsis\
    echo   src-tauri\target\release\bundle\msi\
) else (
    echo Building Podcasterator (binary only^)...
    call npm run tauri build -- --no-bundle || exit /b 1
    echo.
    echo Build complete!
    echo.
    echo The binary is located at:
    echo   src-tauri\target\release\podcasterator.exe
)
