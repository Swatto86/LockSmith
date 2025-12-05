@echo off
echo ========================================
echo   LockSmith Build Script
echo ========================================
echo.

cd /d "%~dp0src-tauri"

echo Building release binary...
cargo build --release
if errorlevel 1 (
    echo.
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo   Build successful!
echo ========================================
echo.
echo Output: src-tauri\target\release\locksmith.exe
echo.
echo NOTE: Right-click and "Run as administrator" to use.
echo.
pause
