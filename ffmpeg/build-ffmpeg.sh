#!/bin/bash
# Builds the slim ffmpeg the app ships (only what convert.rs uses), as one static ffmpeg.exe.
# Needs MSYS2 with: mingw-w64-x86_64-gcc nasm pkgconf libx264 lame, make, diffutils (see README › Installer).
# Run in the "MSYS2 MINGW64" shell, or: C:\msys64\usr\bin\bash.exe -l tauri/tools/build-ffmpeg.sh  (MSYSTEM=MINGW64)
# Source: ffmpeg-9.0.2 unpacked into tauri/ffmpeg-build/ (not in git). Output: tauri/ffmpeg-build/_build/ffmpeg.exe
set -euo pipefail
SRC="$(cd "$(dirname "$0")/../ffmpeg-build" && pwd)"
OUT="$SRC/_build"
mkdir -p "$OUT" && cd "$OUT"

# What the app's commands use (convert.rs):
#   video: -f matroska -i pipe (raw RGB frames with times) + sound.wav + chat.ass -> libx264 / aac / mov_text in MP4,
#          (configure calls the mov_text encoder "movtext")
#          -vf pad, pixel format rgb24 -> yuv420p, -ar 16000 (resampling), -t, -fps_mode vfr, -movflags +faststart
#   audio: -f wav -i pipe -> libmp3lame in MP3
"$SRC/configure" \
  --prefix="$OUT/install" \
  --arch=x86_64 --target-os=mingw32 \
  --enable-gpl --enable-version3 \
  --enable-static --disable-shared --pkg-config-flags=--static \
  --extra-ldflags=-static --extra-libs=-lpthread \
  --disable-autodetect --enable-w32threads \
  --disable-debug --disable-doc --disable-htmlpages --disable-manpages --disable-podpages --disable-txtpages \
  --disable-ffplay --disable-ffprobe --disable-avdevice --disable-network \
  --disable-everything \
  --enable-libx264 --enable-libmp3lame \
  --enable-protocol=file,pipe \
  --enable-demuxer=matroska,wav,ass \
  --enable-decoder=rawvideo,pcm_s16le,ass,ssa \
  --enable-encoder=libx264,aac,libmp3lame,movtext \
  --enable-muxer=mp4,mp3 \
  --enable-filter=pad,scale,format,null,anull,aresample,aformat,trim,atrim,setpts,asetpts \
  --enable-bsf=null

make -j"$(nproc)"
strip ffmpeg.exe
ls -l ffmpeg.exe
./ffmpeg.exe -hide_banner -version | head -n 1
