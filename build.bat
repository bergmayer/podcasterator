@echo off
REM Build script for Podcasterator

echo Installing npm dependencies...
call npm install
if errorlevel 1 exit /b 1

echo Building Podcasterator...
call npm run tauri build
if errorlevel 1 exit /b 1

echo.
echo Build complete!
echo.
echo The binary is located at:
echo   src-tauri\target\release\podcasterator.exe
echo.
echo You can copy this binary to any location you prefer.
