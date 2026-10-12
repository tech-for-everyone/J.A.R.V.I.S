@echo off
REM Build and launch Jarvis.
REM
REM The GUI (jarvis-gui) is the entry window; it starts the voice
REM assistant (jarvis-app) on demand. Both binaries are built into the
REM same target\ directory so they can find each other at runtime.
REM
REM Usage:
REM   run.bat            build (debug) and run
REM   run.bat release    build (release) and run
setlocal
cd /d "%~dp0"

set "MODE=%~1"
if "%MODE%"=="" set "MODE=debug"
if not "%MODE%"=="debug" if not "%MODE%"=="release" (
    echo Unknown mode: '%MODE%' (use 'debug' or 'release')
    exit /b 1
)

REM --- Native library paths (Windows) ---
REM Vosk and other native deps are pre-built in lib/windows/amd64/
REM post_build.py copies them next to the binaries, so just ensure
REM the folder exists for the linker.
if not exist "lib\windows\amd64\libvosk.dll" (
    echo Warning: lib/windows/amd64/libvosk.dll not found
    echo   Ensure native Windows dependencies are present in lib/windows/amd64/
)

REM 1. Frontend. It is embedded into the Rust binary at build time
REM    (custom-protocol), so it must exist before `cargo build`.
echo >> Building frontend
cd frontend
call npm install
if errorlevel 1 exit /b
call npm run build
if errorlevel 1 exit /b
cd ..

REM 2. Rust workspace: builds jarvis-gui and jarvis-app.
echo >> Building Rust (%MODE%)
if "%MODE%"=="release" (
    cargo build --release
) else (
    cargo build
)
if errorlevel 1 exit /b

REM 3. Copy runtime resources (commands, models, sounds, ...) next to the
REM    binaries, where the app expects to find them (current_exe dir).
echo >> Copying resources into target\
python post_build.py
if errorlevel 1 exit /b

REM 4. Launch the GUI; it starts the voice assistant (jarvis-app) on demand.
echo >> Launching Jarvis
if "%MODE%"=="release" (
    start "" "target\release\jarvis-gui.exe"
) else (
    start "" "target\debug\jarvis-gui.exe"
)