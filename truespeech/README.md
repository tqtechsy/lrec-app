# truespeech

DSP Group TrueSpeech 8.5 decoder (8 kHz, 32-byte frames of 240 samples / 30 ms), used by the LREC Converter for the
sound in Linktivity recordings.

This crate is a Rust port of FFmpeg's `libavcodec/truespeech.c` and `truespeech_data.h` (FFmpeg n7.1),
Copyright (c) 2005 Konstantin Shishkov, licensed under the **GNU Lesser General Public License version 2.1 or later**.
The port is therefore also LGPL-2.1-or-later (full text in `LICENSE`). The tables are FFmpeg's, unchanged.

It is kept separate from the rest of the application, which is not LGPL. Anyone receiving the app is entitled to this
crate's source code and must be able to replace it with a modified version.

Checked to give exactly the same samples as FFmpeg's decoder: `cargo run --release --example compare -- <frames> <ffmpeg.pcm>`.
