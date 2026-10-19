//! C interface of the decoder, for using it as a separate library (truespeech.dll). The app loads it at run time, so
//! the decoder can be replaced by a modified build of this crate, as the LGPL requires.
//!
//!     truespeech_abi_version() -> u32                       // 1
//!     truespeech_new() -> *mut Decoder                       // free with truespeech_free
//!     truespeech_decode_frame(d, frame: *const u8, out: *mut i16)   // 32 bytes in, 240 samples out
//!     truespeech_free(d)

use crate::{Decoder, FRAME_BYTES, FRAME_SAMPLES};

/// Changes when the functions below change in an incompatible way (the app checks it).
pub const ABI_VERSION: u32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn truespeech_abi_version() -> u32 { ABI_VERSION }

#[unsafe(no_mangle)]
pub extern "C" fn truespeech_new() -> *mut Decoder { Box::into_raw(Box::new(Decoder::new())) }

/// Decodes one frame of FRAME_BYTES (32) bytes into FRAME_SAMPLES (240) samples.
///
/// # Safety
/// `d` comes from `truespeech_new` and is not freed; `frame` points to 32 readable bytes; `out` to 240 writable samples.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn truespeech_decode_frame(d: *mut Decoder, frame: *const u8, out: *mut i16) {
    if d.is_null() || frame.is_null() || out.is_null() { return; }
    let (d, frame, out) = unsafe { (&mut *d, std::slice::from_raw_parts(frame, FRAME_BYTES), std::slice::from_raw_parts_mut(out, FRAME_SAMPLES)) };
    d.decode_frame(frame, out);
}

/// # Safety
/// `d` comes from `truespeech_new` (or is null) and is not used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn truespeech_free(d: *mut Decoder) {
    if !d.is_null() { drop(unsafe { Box::from_raw(d) }); }
}
