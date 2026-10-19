//! DSP Group TrueSpeech compatible decoder.
//!
//! Rust port of FFmpeg's libavcodec/truespeech.c and truespeech_data.h (FFmpeg n7.1).
//! Copyright (c) 2005 Konstantin Shishkov (original C code and tables).
//!
//! This library is free software; you can redistribute it and/or modify it under the terms of the GNU Lesser General
//! Public License as published by the Free Software Foundation; either version 2.1 of the License, or (at your option)
//! any later version. It is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
//! implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public License
//! for more details (LICENSE in this crate).
//!
//! Arithmetic follows the C code exactly: values stored into 16-bit variables wrap (`as i16`), `>>` on signed values is
//! arithmetic, and sums use wrapping 32-bit arithmetic, so the output matches FFmpeg sample for sample.

/// Samples produced by one 32-byte frame (30 ms at 8 kHz).
pub mod ffi;

pub const FRAME_SAMPLES: usize = 240;
/// Bytes per frame.
pub const FRAME_BYTES: usize = 32;

// ---- tables (FFmpeg's truespeech_data.h, written as raw 16-bit patterns) ----------------------------------------------
const CB0: [u16; 32] = [
    0x8240, 0x8364, 0x84CE, 0x865D, 0x8805, 0x89DE, 0x8BD7, 0x8DF4, 0x9051, 0x92E2, 0x95DE, 0x990F, 0x9C81, 0xA079, 0xA54C, 0xAAD2,
    0xB18A, 0xB90A, 0xC124, 0xC9CC, 0xD339, 0xDDD3, 0xE9D6, 0xF893, 0x096F, 0x1ACA, 0x29EC, 0x381F, 0x45F9, 0x546A, 0x63C3, 0x73B5,
];
const CB1: [u16; 32] = [
    0x9F65, 0xB56B, 0xC583, 0xD371, 0xE018, 0xEBB4, 0xF61C, 0xFF59, 0x085B, 0x1106, 0x1952, 0x214A, 0x28C9, 0x2FF8, 0x36E6, 0x3D92,
    0x43DF, 0x49BB, 0x4F46, 0x5467, 0x5930, 0x5DA3, 0x61EC, 0x65F9, 0x69D4, 0x6D5A, 0x709E, 0x73AD, 0x766B, 0x78F0, 0x7B5A, 0x7DA5,
];
const CB2: [u16; 16] = [0x96F8, 0xA3B4, 0xAF45, 0xBA53, 0xC4B1, 0xCECC, 0xD86F, 0xE21E, 0xEBF3, 0xF640, 0x00F7, 0x0C20, 0x1881, 0x269A, 0x376B, 0x4D60];
const CB3: [u16; 16] = [0xC654, 0xDEF2, 0xEFAA, 0xFD94, 0x096A, 0x143F, 0x1E7B, 0x282C, 0x3176, 0x3A89, 0x439F, 0x4CA2, 0x557F, 0x5E50, 0x6718, 0x6F8D];
const CB4: [u16; 16] = [0xABE7, 0xBBA8, 0xC81C, 0xD326, 0xDD0E, 0xE5D4, 0xEE22, 0xF618, 0xFE28, 0x064F, 0x0EB7, 0x17B8, 0x21AA, 0x2D8B, 0x3BA2, 0x4DF9];
const CB5: [u16; 8] = [0xD51B, 0xF12E, 0x042E, 0x13C7, 0x2260, 0x311B, 0x40DE, 0x5385];
const CB6: [u16; 8] = [0xB550, 0xC825, 0xD980, 0xE997, 0xF883, 0x0752, 0x1811, 0x2E18];
const CB7: [u16; 8] = [0xCEF0, 0xE4F9, 0xF6BB, 0x0646, 0x14F5, 0x23FF, 0x356F, 0x4A8D];

const PULSE_VALUES: [u16; 120] = [
    0x0E46, 0x0CCC, 0x0B6D, 0x0A28, 0x08FC, 0x07E8, 0x06EB, 0x0604, 0x0532, 0x0474, 0x03C9, 0x0330, 0x02A8, 0x0230, 0x01C7,
    0x016C, 0x011E, 0x00DC, 0x00A5, 0x0078, 0x0054, 0x0038, 0x0023, 0x0014, 0x000A, 0x0004, 0x0001, 0x0000, 0x0000, 0x0000,
    0x0196, 0x017A, 0x015F, 0x0145, 0x012C, 0x0114, 0x00FD, 0x00E7, 0x00D2, 0x00BE, 0x00AB, 0x0099, 0x0088, 0x0078, 0x0069,
    0x005B, 0x004E, 0x0042, 0x0037, 0x002D, 0x0024, 0x001C, 0x0015, 0x000F, 0x000A, 0x0006, 0x0003, 0x0001, 0x0000, 0x0000,
    0x001D, 0x001C, 0x001B, 0x001A, 0x0019, 0x0018, 0x0017, 0x0016, 0x0015, 0x0014, 0x0013, 0x0012, 0x0011, 0x0010, 0x000F,
    0x000E, 0x000D, 0x000C, 0x000B, 0x000A, 0x0009, 0x0008, 0x0007, 0x0006, 0x0005, 0x0004, 0x0003, 0x0002, 0x0001, 0x0000,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
];
const DECAY_994_1000: [u16; 8] = [0x7F3B, 0x7E78, 0x7DB6, 0x7CF5, 0x7C35, 0x7B76, 0x7AB8, 0x79FC];
const ORDER2_COEFFS: [u16; 50] = [
    0xED2F, 0x5239, 0x54F1, 0xE4A9, 0x2620, 0xEE3E, 0x09D6, 0x2C40, 0xEFB5, 0x2BE0,
    0x3FE1, 0x3339, 0x442F, 0xE6FE, 0x4458, 0xF9DF, 0xF231, 0x43DB, 0x3DB0, 0xF705,
    0x4F7B, 0xFEFB, 0x26AD, 0x0CDC, 0x33C2, 0x0739, 0x12BE, 0x43A2, 0x1BDF, 0x1F3E,
    0x0211, 0x0796, 0x2AEB, 0x163F, 0x050D, 0x3A38, 0x0D1E, 0x0D78, 0x150F, 0x3346,
    0x38A4, 0x0B7D, 0x2D5D, 0x1FDF, 0x19B7, 0x2822, 0x0D99, 0x1F12, 0x194C, 0x0CE6,
];
const PULSE_SCALES: [u16; 64] = [
    0x0002, 0x0006, 0xFFFE, 0xFFFA, 0x0004, 0x000C, 0xFFFC, 0xFFF4, 0x0006, 0x0012, 0xFFFA, 0xFFEE, 0x000A, 0x001E, 0xFFF6, 0xFFE2,
    0x0010, 0x0030, 0xFFF0, 0xFFD0, 0x0019, 0x004B, 0xFFE7, 0xFFB5, 0x0028, 0x0078, 0xFFD8, 0xFF88, 0x0040, 0x00C0, 0xFFC0, 0xFF40,
    0x0065, 0x012F, 0xFF9B, 0xFED1, 0x00A1, 0x01E3, 0xFF5F, 0xFE1D, 0x0100, 0x0300, 0xFF00, 0xFD00, 0x0196, 0x04C2, 0xFE6A, 0xFB3E,
    0x0285, 0x078F, 0xFD7B, 0xF871, 0x0400, 0x0C00, 0xFC00, 0xF400, 0x0659, 0x130B, 0xF9A7, 0xECF5, 0x0A14, 0x1E3C, 0xF5EC, 0xE1C4,
];
const DECAY_35_64: [u16; 8] = [0x4666, 0x26B8, 0x154C, 0x0BB6, 0x0671, 0x038B, 0x01F3, 0x0112];
const DECAY_3_4: [u16; 8] = [0x6000, 0x4800, 0x3600, 0x2880, 0x1E60, 0x16C8, 0x1116, 0x0CD1];

/// A 16-bit table entry as the C `int16_t` value, widened to `int`.
#[inline]
fn s(v: u16) -> i32 { v as i16 as i32 }
#[inline]
fn clip(v: i32, lo: i32, hi: i32) -> i32 { v.clamp(lo, hi) }

/// Reads bits MSB-first after byte-swapping each 32-bit word (FFmpeg: bswap_buf + get_bits).
struct Bits { words: [u32; 8], pos: usize }
impl Bits {
    fn new(frame: &[u8]) -> Self {
        let mut words = [0u32; 8];
        for (i, w) in words.iter_mut().enumerate() { *w = u32::from_le_bytes([frame[i * 4], frame[i * 4 + 1], frame[i * 4 + 2], frame[i * 4 + 3]]); }
        Bits { words, pos: 0 }
    }
    fn get(&mut self, n: usize) -> i32 {
        let mut v = 0u32;
        for _ in 0..n {
            let bit = (self.words[self.pos / 32] >> (31 - self.pos % 32)) & 1;
            v = (v << 1) | bit;
            self.pos += 1;
        }
        v as i32
    }
}

/// Decoder state for one stream (one speaker): the codec carries filter memory from frame to frame.
#[derive(Clone)]
pub struct Decoder {
    // input data of the current frame
    vector: [i16; 8],
    offset1: [i32; 2],
    offset2: [i32; 4],
    pulseoff: [i32; 4],
    pulsepos: [i32; 4],
    pulseval: [i32; 4],
    flag: i32,
    // state
    filtbuf: [i32; 146],
    prevfilt: [i32; 8],
    tmp1: [i16; 8],
    tmp2: [i16; 8],
    tmp3: [i16; 8],
    cvector: [i16; 8],
    filtval: i32,
    newvec: [i16; 60],
    filters: [i16; 32],
}

impl Default for Decoder { fn default() -> Self { Self::new() } }

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            vector: [0; 8], offset1: [0; 2], offset2: [0; 4], pulseoff: [0; 4], pulsepos: [0; 4], pulseval: [0; 4], flag: 0,
            filtbuf: [0; 146], prevfilt: [0; 8], tmp1: [0; 8], tmp2: [0; 8], tmp3: [0; 8], cvector: [0; 8], filtval: 0,
            newvec: [0; 60], filters: [0; 32],
        }
    }

    /// Decodes whole 32-byte frames from `data` (a trailing partial frame is ignored), appending 240 samples per frame.
    pub fn decode(&mut self, data: &[u8], out: &mut Vec<i16>) {
        for frame in data.chunks_exact(FRAME_BYTES) {
            let start = out.len();
            out.resize(start + FRAME_SAMPLES, 0);
            self.decode_frame(frame, &mut out[start..]);
        }
    }

    /// Decodes one 32-byte frame into 240 samples.
    pub fn decode_frame(&mut self, frame: &[u8], out: &mut [i16]) {
        assert!(frame.len() >= FRAME_BYTES && out.len() >= FRAME_SAMPLES);
        self.read_frame(frame);
        self.correlate_filter();
        self.filters_merge();
        for quart in 0..4 {
            let o = &mut out[quart * 60..quart * 60 + 60];
            self.apply_twopoint_filter(quart);
            self.place_pulses(o, quart);
            self.update_filters(o);
            self.synth(o, quart);
        }
        for i in 0..8 { self.prevfilt[i] = self.cvector[i] as i32; }
    }

    fn read_frame(&mut self, input: &[u8]) {
        let mut b = Bits::new(input);
        self.vector[7] = CB7[b.get(3) as usize] as i16;
        self.vector[6] = CB6[b.get(3) as usize] as i16;
        self.vector[5] = CB5[b.get(3) as usize] as i16;
        self.vector[4] = CB4[b.get(4) as usize] as i16;
        self.vector[3] = CB3[b.get(4) as usize] as i16;
        self.vector[2] = CB2[b.get(4) as usize] as i16;
        self.vector[1] = CB1[b.get(5) as usize] as i16;
        self.vector[0] = CB0[b.get(5) as usize] as i16;
        self.flag = b.get(1);

        self.offset1[0] = b.get(4) << 4;
        self.offset2[3] = b.get(7);
        self.offset2[2] = b.get(7);
        self.offset2[1] = b.get(7);
        self.offset2[0] = b.get(7);

        self.offset1[1] = b.get(4);
        self.pulseval[1] = b.get(14);
        self.pulseval[0] = b.get(14);

        self.offset1[1] |= b.get(4) << 4;
        self.pulseval[3] = b.get(14);
        self.pulseval[2] = b.get(14);

        for q in 0..4 {
            self.offset1[0] |= b.get(1) << q;
            self.pulsepos[q] = b.get(27);
            self.pulseoff[q] = b.get(4);
        }
    }

    fn correlate_filter(&mut self) {
        let mut tmp = [0i16; 8];
        for i in 0..8 {
            if i > 0 {
                tmp[..i].copy_from_slice(&self.cvector[..i]);
                for j in 0..i {
                    let add = (tmp[i - j - 1] as i32).wrapping_mul(self.vector[i] as i32).wrapping_add(0x4000) >> 15;
                    self.cvector[j] = (self.cvector[j] as i32).wrapping_add(add) as i16;
                }
            }
            self.cvector[i] = ((8 - self.vector[i] as i32) >> 3) as i16;
        }
        for i in 0..8 {
            self.cvector[i] = ((self.cvector[i] as i32).wrapping_mul(s(DECAY_994_1000[i])) >> 15) as i16;
        }
        self.filtval = self.vector[0] as i32;
    }

    fn filters_merge(&mut self) {
        if self.flag == 0 {
            for i in 0..8 {
                self.filters[i] = self.prevfilt[i] as i16;
                self.filters[i + 8] = self.prevfilt[i] as i16;
            }
        } else {
            for i in 0..8 {
                let (c, p) = (self.cvector[i] as i32, self.prevfilt[i]);
                self.filters[i] = (c.wrapping_mul(21846).wrapping_add(p.wrapping_mul(10923)).wrapping_add(16384) >> 15) as i16;
                self.filters[i + 8] = (c.wrapping_mul(10923).wrapping_add(p.wrapping_mul(21846)).wrapping_add(16384) >> 15) as i16;
            }
        }
        for i in 0..8 {
            self.filters[i + 16] = self.cvector[i];
            self.filters[i + 24] = self.cvector[i];
        }
    }

    fn apply_twopoint_filter(&mut self, quart: usize) {
        let mut t = self.offset2[quart];
        if t == 127 {
            self.newvec = [0; 60];
            return;
        }
        let mut tmp = [0i16; 146 + 60];
        for i in 0..146 { tmp[i] = self.filtbuf[i] as i16; }
        let off = clip(t / 25 + self.offset1[quart >> 1] + 18, 0, 145) as usize;
        let f = (t % 25) as usize * 2;
        let (f0, f1) = (s(ORDER2_COEFFS[f]), s(ORDER2_COEFFS[f + 1]));
        for i in 0..60 {
            let p = 145 - off + i;
            t = (tmp[p] as i32).wrapping_mul(f0).wrapping_add((tmp[p + 1] as i32).wrapping_mul(f1)).wrapping_add(0x2000) >> 14;
            self.newvec[i] = t as i16;
            tmp[146 + i] = t as i16;
        }
    }

    fn place_pulses(&mut self, out: &mut [i16], quart: usize) {
        out[..60].fill(0);
        let mut tmp = [0i16; 7];
        for i in 0..7 {
            let t = self.pulseval[quart] & 3;
            self.pulseval[quart] >>= 2;
            tmp[6 - i] = PULSE_SCALES[(self.pulseoff[quart] * 4 + t) as usize] as i16;
        }
        let mut next = 0; // index into tmp (C: ptr2)
        let mut coef = self.pulsepos[quart] >> 15;
        let mut p1 = 30usize; // index into PULSE_VALUES (C: ptr1)
        let mut j = 3;
        let mut i = 0;
        while i < 30 && j > 0 {
            let t = s(PULSE_VALUES[p1]);
            p1 += 1;
            if coef >= t { coef -= t; } else { out[i] = tmp[next]; next += 1; p1 += 30; j -= 1; }
            i += 1;
        }
        coef = self.pulsepos[quart] & 0x7FFF;
        p1 = 0;
        j = 4;
        i = 30;
        while i < 60 && j > 0 {
            let t = s(PULSE_VALUES[p1]);
            p1 += 1;
            if coef >= t { coef -= t; } else { out[i] = tmp[next]; next += 1; p1 += 30; j -= 1; }
            i += 1;
        }
    }

    fn update_filters(&mut self, out: &mut [i16]) {
        self.filtbuf.copy_within(60..146, 0);
        for i in 0..60 {
            let (o, n) = (out[i] as i32, self.newvec[i] as i32);
            self.filtbuf[i + 86] = o.wrapping_add(n).wrapping_sub(n >> 3);
            out[i] = o.wrapping_add(n) as i16;
        }
    }

    fn synth(&mut self, out: &mut [i16], quart: usize) {
        let f = quart * 8;
        let filt: [i32; 8] = std::array::from_fn(|k| self.filters[f + k] as i32);

        for i in 0..60 {
            let mut sum: i32 = 0;
            for k in 0..8 { sum = sum.wrapping_add((self.tmp1[k] as i32).wrapping_mul(filt[k])); }
            let sum = (out[i] as i32).wrapping_add((sum.wrapping_add(0x800)) >> 12);
            out[i] = clip(sum, -0x7FFE, 0x7FFE) as i16;
            self.tmp1.copy_within(0..7, 1);
            self.tmp1[0] = out[i];
        }

        let t: [i32; 8] = std::array::from_fn(|k| s(DECAY_35_64[k]).wrapping_mul(filt[k]) >> 15);
        for i in 0..60 {
            let mut sum: i32 = 0;
            for k in 0..8 { sum = sum.wrapping_add((self.tmp2[k] as i32).wrapping_mul(t[k])); }
            self.tmp2.copy_within(0..7, 1);
            self.tmp2[0] = out[i];
            out[i] = (out[i] as i32).wrapping_add(sum.wrapping_neg() >> 12) as i16;
        }

        let t: [i32; 8] = std::array::from_fn(|k| s(DECAY_3_4[k]).wrapping_mul(filt[k]) >> 15);
        for i in 0..60 {
            let mut sum: i32 = (out[i] as i32).wrapping_mul(1 << 12);
            for k in 0..8 { sum = sum.wrapping_add((self.tmp3[k] as i32).wrapping_mul(t[k])); }
            self.tmp3.copy_within(0..7, 1);
            self.tmp3[0] = clip(sum.wrapping_add(0x800) >> 12, -0x7FFE, 0x7FFE) as i16;
            let mut sum = ((self.tmp3[1] as i32).wrapping_mul(self.filtval - (self.filtval >> 2)) >> 4).wrapping_add(sum);
            sum = sum.wrapping_sub(sum >> 3);
            out[i] = clip(sum.wrapping_add(0x800) >> 12, -0x7FFE, 0x7FFE) as i16;
        }
    }
}
