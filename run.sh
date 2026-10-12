#!/usr/bin/env bash
# Build and launch Jarvis.
#
# The GUI (jarvis-gui) is the entry window; it starts the voice
# assistant (jarvis-app) on demand. Both binaries are built into the
# same target/ directory so they can find each other at runtime.
#
# Usage:
#   ./run.sh            build (debug) and run
#   ./run.sh release    build (release) and run
set -euo pipefail

# always run from the repository root
cd "$(dirname "$0")"

MODE="${1:-debug}"
case "$MODE" in
    debug|release) ;;
    *) echo "Unknown mode: '$MODE' (use 'debug' or 'release')" >&2; exit 1 ;;
esac

# --- Native library paths (required for linking & runtime) ---
# Vosk: use the Proton-bundled libvosk.so (x86_64-linux-gnu)
VOSK_LIB_DIR="${VOSK_LIB_DIR:-/home/codeosuser/.local/share/Steam/compatibilitytools.d/GE-Proton11-6-x86_64/files/lib/x86_64-linux-gnu}"
if [ ! -f "$VOSK_LIB_DIR/libvosk.so" ]; then
    echo "Warning: libvosk.so not found at $VOSK_LIB_DIR" >&2
    echo "  Set VOSK_LIB_DIR to a directory containing libvosk.so" >&2
    echo "  (e.g. from a Steam Proton installation)" >&2
fi
export LIBRARY_PATH="$VOSK_LIB_DIR:${LIBRARY_PATH:-}"
export LD_LIBRARY_PATH="$VOSK_LIB_DIR:${LD_LIBRARY_PATH:-}"

# libxdo: needed for the tray module on Linux (package: libxdo from AUR,
# or install xdotool which may provide it). Check and warn.
if ! ldconfig -p 2>/dev/null | grep -q "libxdo\.so" && ! pkg-config --exists xdo 2>/dev/null; then
    echo "Warning: libxdo not found (required for system tray on Linux)" >&2
    echo "  Arch: install 'libxdo' from AUR, or 'xdotool' which may include it" >&2
fi

# 1. Frontend. It is embedded into the Rust binary at build time
#    (custom-protocol), so it must exist before `cargo build`.
echo ">> Building frontend"
( cd frontend && npm install && npm run build )

# 2. Rust workspace: builds jarvis-gui and jarvis-app.
echo ">> Building Rust ($MODE)"
if [ "$MODE" = "release" ]; then
    cargo build --release
else
    cargo build
fi

# 3. Copy runtime resources (commands, models, sounds, ...) next to the
#    binaries, where the app expects to find them (current_exe dir).
echo ">> Copying resources into target/"
PYTHON="$(command -v python3 || command -v python)"
if [ -z "$PYTHON" ]; then
    echo "Python is required to copy resources (post_build.py)" >&2
    exit 1
fi
"$PYTHON" post_build.py

# 4. Launch the GUI; it starts the voice assistant (jarvis-app) on demand.
echo ">> Launching Jarvis"
if [ "$MODE" = "release" ]; then
    exec ./target/release/jarvis-gui
else
    exec ./target/debug/jarvis-gui
fi