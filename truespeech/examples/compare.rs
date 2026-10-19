//! Decodes raw TrueSpeech frames and compares the samples with FFmpeg's output for the same frames.
//! cargo run --release --example compare -- <frames.bin> <ffmpeg-s16le.pcm>
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames = std::fs::read(&a[1]).expect("frames file");
    let reference: Vec<i16> = std::fs::read(&a[2]).expect("pcm file").chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
    let t = std::time::Instant::now();
    let mut out = Vec::new();
    truespeech::Decoder::new().decode(&frames, &mut out);
    let took = t.elapsed();
    let n = out.len().min(reference.len());
    let diffs = (0..n).filter(|&i| out[i] != reference[i]).count();
    let first = (0..n).find(|&i| out[i] != reference[i]);
    let max = (0..n).map(|i| (out[i] as i32 - reference[i] as i32).abs()).max().unwrap_or(0);
    println!("{} frames, {} samples (ffmpeg {}), decoded in {:.0} ms; differing samples: {diffs}, max difference {max}, first at {:?}",
        frames.len() / 32, out.len(), reference.len(), took.as_secs_f64() * 1000.0, first);
}
