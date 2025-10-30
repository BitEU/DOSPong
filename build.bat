@echo off
echo Building Terminal Pong...
echo.

cargo build --release

if %errorlevel% == 0 (
    echo.
    echo ========================================
    echo Build successful!
    echo ========================================
    echo.
    echo Executable location:
    echo target\release\terminal-pong.exe
    echo.
    echo To run the game:
    echo   cargo run --release
    echo Or:
    echo   .\target\release\terminal-pong.exe
    echo.
) else (
    echo.
    echo Build failed! Please check the errors above.
    echo Make sure you have Rust installed.
)

pause