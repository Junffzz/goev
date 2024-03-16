// pub mod ffmpeg;
pub mod opus;
pub mod os;


// pub mod ffmpeg_binding;

#[allow(
non_snake_case,
non_camel_case_types,
non_upper_case_globals,
improper_ctypes,
clippy::all
)]
pub mod ffmpeg_ffi {
    // pub use crate::ffmpeg_binding::*;
    // include!(concat!("/Users/zhaojunfeng/workspace/rust/devrust/src", "/binding.rs"));
    // pub use rusty_ffmpeg::avutil::{_avutil::*, common::*, error::*, pixfmt::*, rational::*};

    // pub use crate::ffmpeg::*;
    // pub use crate::ffmpeg::utils::error::*;

    pub use rsmpeg::ffi::*;
    pub use rsmpeg::avcodec;
    pub use rsmpeg::avfilter;
    pub use rsmpeg::avformat;
    pub use rsmpeg::avutil;
    pub use rsmpeg::swresample;
    pub use rsmpeg::swscale;

    pub use rsmpeg::error;

    pub use rsmpeg::UnsafeDerefMut;
}