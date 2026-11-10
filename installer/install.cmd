@echo off
rem Double-click to install argmem from the extracted zip folder.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
echo.
pause
