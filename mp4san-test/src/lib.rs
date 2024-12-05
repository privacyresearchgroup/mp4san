//! `mp4san` testing library.
//!
//! This crate is separate from mp4san to workaround cargo's inability to specify optional dev-dependencies (see
//! rust-lang/cargo#1596).

#[cfg(feature = "ffmpeg")]
pub mod ffmpeg;

#[cfg(feature = "gpac")]
pub mod gpac;

//
// public types
//

//
// private types
//

#[derive(Debug, thiserror::Error)]
pub enum VerifyError<T> {
    #[error("data longer than expected: frame len {frame_len} > {remaining} remaining")]
    DataLongerThanExpected { frame_len: usize, remaining: usize },

    #[error("data at offset {offset} did not match")]
    DataMismatch { offset: u64, len: usize },

    #[error("data shorter than expected: {remaining} remaining")]
    DataShorterThanExpected { remaining: usize },

    #[error(transparent)]
    Parse(#[from] T),
}

//
// public functions
//

/// Read `data` using ffmpeg, verifying that the demuxed frames match the `expected_media_data`.
#[cfg_attr(not(feature = "ffmpeg"), allow(unused_variables))]
pub fn ffmpeg_assert_eq(data: &[u8], expected_media_data: &[u8]) {
    #[cfg(not(feature = "ffmpeg"))]
    log::info!("not verifying sanitizer output using ffmpeg; ffmpeg feature disabled");
    #[cfg(feature = "ffmpeg")]
    ffmpeg::verify_ffmpeg(data, Some(expected_media_data))
        .unwrap_or_else(|error| panic!("ffmpeg returned an error: {error}\n{error:?}"));
}

/// Read `data` using ffmpeg, verifying that it cannot be demuxed.
#[cfg_attr(not(feature = "ffmpeg"), allow(unused_variables))]
pub fn ffmpeg_assert_invalid(data: &[u8]) {
    #[cfg(not(feature = "ffmpeg"))]
    log::info!("not verifying sanitizer output using ffmpeg; ffmpeg feature disabled");
    #[cfg(feature = "ffmpeg")]
    ffmpeg::verify_ffmpeg(data, None)
        .err()
        .unwrap_or_else(|| panic!("ffmpeg didn't return an error"));
}

/// Read `data` using ffmpeg, verifying that it can be demuxed.
#[cfg_attr(not(feature = "ffmpeg"), allow(unused_variables))]
pub fn ffmpeg_assert_valid(data: &[u8]) {
    #[cfg(not(feature = "ffmpeg"))]
    log::info!("not verifying sanitizer output using ffmpeg; ffmpeg feature disabled");
    #[cfg(feature = "ffmpeg")]
    ffmpeg::verify_ffmpeg(data, None).unwrap_or_else(|error| panic!("ffmpeg returned an error: {error}\n{error:?}"));
}

/// Read `data` using GPAC, verifying that the demuxed frames match the `expected_media_data`.
#[cfg_attr(not(feature = "gpac"), allow(unused_variables))]
pub fn gpac_assert_eq(data: &[u8], expected_media_data: &[u8]) {
    #[cfg(not(feature = "gpac"))]
    log::info!("not verifying sanitizer output using gpac; gpac feature disabled");
    #[cfg(feature = "gpac")]
    gpac::verify_gpac(data, Some(expected_media_data))
        .unwrap_or_else(|error| panic!("gpac returned an error: {error}\n{error:?}"));
}

/// Read `data` using GPAC, verifying that it cannot be demuxed.
#[cfg_attr(not(feature = "gpac"), allow(unused_variables))]
pub fn gpac_assert_invalid(data: &[u8]) {
    #[cfg(not(feature = "gpac"))]
    log::info!("not verifying sanitizer output using gpac; gpac feature disabled");
    #[cfg(feature = "gpac")]
    gpac::verify_gpac(data, None)
        .err()
        .unwrap_or_else(|| panic!("gpac didn't return an error"));
}

/// Read `data` using GPAC, verifying that it can be demuxed.
#[cfg_attr(not(feature = "gpac"), allow(unused_variables))]
pub fn gpac_assert_valid(data: &[u8]) {
    #[cfg(not(feature = "gpac"))]
    log::info!("not verifying sanitizer output using gpac; gpac feature disabled");
    #[cfg(feature = "gpac")]
    gpac::verify_gpac(data, None).unwrap_or_else(|error| panic!("gpac returned an error: {error}\n{error:?}"));
}

pub fn example_ftyp() -> Vec<u8> {
    const EXAMPLE_FTYP: &[&[u8]] = &[
        &[0, 0, 0, 20], // box size
        b"ftyp",        // box type
        b"isom",        // major_brand
        &[0, 0, 0, 0],  // minor_version
        b"isom",        // compatible_brands
    ];
    EXAMPLE_FTYP.concat()
}

pub fn example_mdat() -> Vec<u8> {
    const EXAMPLE_MDAT: &[&[u8]] = &[
        &[0, 0, 0, 8], // box size
        b"mdat",       // box type
    ];
    EXAMPLE_MDAT.concat()
}

pub fn example_moov() -> Vec<u8> {
    const EXAMPLE_MOOV: &[&[u8]] = &[
        &[0, 0, 0, 64], // box size
        b"moov",        // box type
        //
        // trak box (inside moov box)
        //
        &[0, 0, 0, 48], // box size
        b"trak",        // box type
        //
        // mdia box (inside trak box)
        //
        &[0, 0, 0, 40], // box size
        b"mdia",        // box type
        //
        // minf box (inside mdia box)
        //
        &[0, 0, 0, 32], // box size
        b"minf",        // box type
        //
        // stbl box (inside minf box)
        //
        &[0, 0, 0, 24], // box size
        b"stbl",        // box type
        //
        // stco box (inside stbl box)
        //
        &[0, 0, 0, 16], // box size
        b"stco",        // box type
        &[0, 0, 0, 0],  // box version & flags
        &[0, 0, 0, 0],  // entry count
        //
        // mvhd box (inside moov box)
        //
        &[0, 0, 0, 8],
        b"mvhd",
    ];
    EXAMPLE_MOOV.concat()
}
