@echo off
title tiD - CVM Pricing Cockpit

REM Change to the directory where this batch file lives
cd /d "%~dp0"

echo ================================
echo tiD - CVM Pricing Cockpit
echo ================================
echo.

REM Start the server in background
start /b tiD.exe serve

REM Wait a moment for the server to start
timeout /t 2 /nobreak >nul

REM Open browser
start http://127.0.0.1:5002

echo Server running at http://127.0.0.1:5002
echo Press Ctrl+C to stop...
echo.

REM Wait for Ctrl+C
pause >nul

REM Kill the server process
taskkill /f /im tiD.exe >nul 2>&1