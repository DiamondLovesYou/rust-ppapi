// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ffi;
use ppb::{MediaStreamVideoTrackIf, get_media_stream_video_track};

use super::{Callback, CallbackArgs, StringVar, Code, Result, Resource};
use super::video_frame::{self, VideoFrame};

/// Created on the JS side and sent in a message.
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct VideoTrack(ffi::PP_Resource);

impl_clone_drop_for!(VideoTrack);
impl_resource_for!(VideoTrack, ResourceType::VideoFrameRes);

#[doc(hidden)]
impl From<ffi::PP_Resource> for VideoTrack {
    fn from(v: ffi::PP_Resource) -> VideoTrack {
        debug_assert!(get_media_stream_video_track().is(v));
        VideoTrack(v)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Attr {
    /// The number of buffer frames. Chrome may use more or less. How many
    /// frames to buffer depends on usage - request at least 2 to make sure
    /// latency doesn't cause lost frames. If the plugin expects to hold on to
    /// more than one frame at a time (e.g. to do multi-frame processing), it
    /// should request that many more. If this attribute is not specified or
    /// value 0 is specified for this attribute, the default value will be
    /// used.
    Buffers(u32),

    /// The width of video frames in pixels.
    ///
    /// It should be a multiple of 4. If the specified size is different from
    /// the video source (webcam), frames will be scaled to specified size. If
    /// this attribute is not specified or value 0 is specified, the original
    /// frame size of the video track will be used.
    ///
    /// Maximum value: 4096 (4K resolution).
    Width(u32),
    /// The height of video frames in pixels.
    ///
    /// It should be a multiple of 4. If the specified size is different from
    /// the video source (webcam), frames will be scaled to specified size. If
    /// this attribute is not specified or value 0 is specified, the original
    /// frame size of the video track will be used.
    ///
    /// Maximum value: 4096 (4K resolution).
    Height(u32),

    /// The format of video frames.
    ///
    /// The attribute value is a `video_frame::Format`. If the specified
    /// format is different from the video source (webcam), frames will be
    /// converted to specified format. If this attribute is not specified or
    /// value `Format::Unknown` is specified, the orignal frame format of the
    /// video track will be used.
    Format(video_frame::Format),
}
impl Attr {
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::PP_MediaStreamVideoTrack_Attrib {
        match self {
            Attr::Buffers(..) => ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_BUFFERED_FRAMES,
            Attr::Width(..) => ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_WIDTH,
            Attr::Height(..) => ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_HEIGHT,
            Attr::Format(..) => ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_FORMAT,
        }
    }
}

impl VideoTrack {
    pub fn configure<'a, T: AsRef<&'a [Attr]>, F>(&self, attrs: T, callback: F) -> Code
        where F: Callback
    {
        use std::cmp::min;

        let mut nattrs: Vec<ffi::PP_MediaStreamVideoTrack_Attrib> =
            Vec::with_capacity(attrs.as_ref().len() + 1);
        for attr in attrs.as_ref().iter() {
            nattrs.push(attr.to_ffi());
            match attr {
                &Attr::Buffers(c) => {
                    nattrs.push(c as ffi::PP_MediaStreamVideoTrack_Attrib);
                },
                &Attr::Width(c) | &Attr::Height(c) => {
                    nattrs.push(min(c, 4096) as ffi::PP_MediaStreamVideoTrack_Attrib);
                },
                &Attr::Format(f) => {
                    nattrs.push(f.to_ffi());
                },
            }
        }
        nattrs.push(ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_NONE);

        get_media_stream_video_track()
            .configure(self.unwrap(), nattrs.as_ref(), callback.to_ffi_callback())
    }
    pub fn get_attr(&self, attr: Attr) -> Result<Attr> {
        get_media_stream_video_track()
            .get_attrib(self.unwrap(), attr.to_ffi())
            .map(|i| {
                match attr {
                    Attr::Buffers(..) => Attr::Buffers(i as u32),
                    Attr::Width(..) => Attr::Width(i as u32),
                    Attr::Height(..) => Attr::Height(i as u32),
                    Attr::Format(..) => Attr::Format(From::from(i as u32))
                }
            })
    }
    pub fn get_id(&self) -> StringVar {
        From::from(get_media_stream_video_track().get_id(self.unwrap()))
    }
    pub fn has_ended(&self) -> bool {
        get_media_stream_video_track()
            .has_ended(self.unwrap())
    }
    pub fn get_frame<F>(&self, f: F) -> Code
        where F: CallbackArgs<VideoFrame>
    {
        fn mapper(res: ffi::PP_Resource) -> VideoFrame { From::from(res) }

        let (cb, frame) = f.to_ffi_callback(0, mapper);
        get_media_stream_video_track()
            .get_frame(self.unwrap(), frame, cb)
    }
    pub fn recycle_frame(&self, frame: VideoFrame) -> Code {
        get_media_stream_video_track()
            .recycle_frame(self.unwrap(), frame.unwrap())
    }
    pub fn close(self) {
        get_media_stream_video_track()
            .close(self.unwrap())
    }
}
