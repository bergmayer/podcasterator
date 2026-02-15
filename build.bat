@echo off
setlocal enabledelayedexpansion

REM Build script for Podcasterator (Windows)
REM
REM Usage:
REM   build.bat              Build raw binary only
REM   build.bat --makebundle Build Windows installer bundles

set "MAKEBUNDLE=false"
for %%a in (%*) do (
    if "%%a"=="--makebundle" (
        set "MAKEBUNDLE=true"
    ) else (
        echo Unknown option: %%a
        echo Usage: build.bat [--makebundle]
        exit /b 1
    )
)

cd /d "%~dp0"

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
