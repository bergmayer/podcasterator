@echo off
REM Build script for Podcasterator

echo Installing npm dependencies...
call npm install
if errorlevel 1 exit /b 1

echo Building Podcasterator for Windows...
call npm run tauri build
if errorlevel 1 exit /b 1

echo.
echo Build complete!
echo Binary: src-tauri\target\release\podcasterator.exe
echo Packages: src-tauri\target\release\bundle\
