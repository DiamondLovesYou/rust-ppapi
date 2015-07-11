// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ffi;
use ppb::{get_video_frame, VideoFrameIf};

use {TimeDelta, Resource, Size};

#[derive(Hash, Eq, PartialEq, Debug)] pub struct VideoFrame(ffi::PP_Resource);
impl_clone_drop_for!(VideoFrame);
impl_resource_for!(VideoFrame, ResourceType::VideoFrameRes);

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Format {
    Unknown,
    YV12,
    I420,
    BGRA,
}

#[doc(hidden)]
impl From<ffi::PP_VideoFrame_Format> for Format {
    fn from(v: ffi::PP_VideoFrame_Format) -> Format {
        match v {
            ffi::PP_VIDEOFRAME_FORMAT_UNKNOWN => Format::Unknown,
            ffi::PP_VIDEOFRAME_FORMAT_YV12 => Format::YV12,
            ffi::PP_VIDEOFRAME_FORMAT_I420 => Format::I420,
            ffi::PP_VIDEOFRAME_FORMAT_BGRA => Format::BGRA,
            _ => unreachable!(),
        }
    }
}
impl Format {
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::PP_VideoFrame_Format {
        match self {
            Format::YV12 => ffi::PP_VIDEOFRAME_FORMAT_YV12,
            Format::I420 => ffi::PP_VIDEOFRAME_FORMAT_I420,
            Format::BGRA => ffi::PP_VIDEOFRAME_FORMAT_BGRA,
            Format::Unknown => ffi::PP_VIDEOFRAME_FORMAT_UNKNOWN,
        }
    }
}
#[doc(hidden)]
impl From<ffi::PP_Resource> for VideoFrame {
    fn from(v: ffi::PP_Resource) -> VideoFrame {
        debug_assert!(get_video_frame().is(v));
        VideoFrame(v)
    }
}

impl VideoFrame {
    pub fn get_timestamp(&self) -> TimeDelta {
        get_video_frame()
            .get_timestamp(self.unwrap())
    }
    pub fn set_timestamp(&self, ts: TimeDelta) {
        get_video_frame()
            .set_timestamp(self.unwrap(), ts)
    }
    pub fn format(&self) -> Format {
        let f = get_video_frame()
            .get_format(self.unwrap());
        From::from(f)
    }
    pub fn size(&self) -> Option<Size> {
        get_video_frame()
            .get_size(self.unwrap())
            .map(|s| From::from(s) )
    }
    pub fn len(&self) -> usize {
        get_video_frame()
            .get_data_buffer_size(self.unwrap())
    }
}
impl AsRef<[u8]> for VideoFrame {
    fn as_ref(&self) -> &[u8] {
        use std::slice::from_raw_parts;
        let f = get_video_frame();

        let len = f.get_data_buffer_size(self.unwrap());
        let data = f.get_data_buffer(self.unwrap());

        unsafe { from_raw_parts(data, len) }
    }
}
