@echo off
echo Starting Terminal Pong...
echo.

if exist "target\release\terminal-pong.exe" (
    target\release\terminal-pong.exe
) else (
    echo Executable not found! Building first...
    echo.
    cargo build --release
    if %errorlevel% == 0 (
        echo.
        echo Build complete! Starting game...
        target\release\terminal-pong.exe
    ) else (
        echo Build failed! Please run build.bat first.
        pause
    )
)