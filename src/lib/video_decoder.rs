// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

use ffi;

use gles::{self, Context3d, TextureBuffer, TexFormat};
use ppb::{get_video_decoder_opt, VideoDecoderIf};

use super::{GenericResource, Resource, ResourceType, Callback, Code,
            CallbackArgs, StorageToArgsMapper};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VideoDecoder(GenericResource);
impl Resource for VideoDecoder {
    fn unwrap(&self) -> ffi::PP_Resource { self.0.unwrap() }
    fn type_of(&self) -> Option<ResourceType> { Some(ResourceType::VideoDecoder) }
}
#[doc(hidden)]
impl From<ffi::PP_Resource> for VideoDecoder {
    fn from(v: ffi::PP_Resource) -> VideoDecoder {
        debug_assert!(get_video_decoder().is(v));
        VideoDecoder(From::from(v))
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum H264Profile {
    BaseLine,
    Main,
    Extended,
    High,
    High10,
    High422,
    High444Predictive,
    ScalableBaseLine,
    ScalableHigh,
    StereoHigh,
    MultiViewHigh,
}
#[doc(hidden)]
impl Into<ffi::PP_VideoProfile> for H264Profile {
    fn into(self) -> ffi::PP_VideoProfile {
        use self::H264Profile::*;
        match self {
            BaseLine => ffi::PP_VIDEOPROFILE_H264BASELINE,
            Main => ffi::PP_VIDEOPROFILE_H264MAIN,
            Extended => ffi::PP_VIDEOPROFILE_H264EXTENDED,
            High => ffi::PP_VIDEOPROFILE_H264HIGH,
            High10 => ffi::PP_VIDEOPROFILE_H264HIGH10PROFILE,
            High422 => ffi::PP_VIDEOPROFILE_H264HIGH422PROFILE,
            High444Predictive => ffi::PP_VIDEOPROFILE_H264HIGH444PREDICTIVEPROFILE,
            ScalableBaseLine => ffi::PP_VIDEOPROFILE_H264SCALABLEBASELINE,
            ScalableHigh => ffi::PP_VIDEOPROFILE_H264SCALABLEHIGH,
            StereoHigh => ffi::PP_VIDEOPROFILE_H264STEREOHIGH,
            MultiViewHigh => ffi::PP_VIDEOPROFILE_H264MULTIVIEWHIGH,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Profile {
    VP9,
    VP8,
    H264(H264Profile),
}
#[doc(hidden)]
impl Into<ffi::PP_VideoProfile> for Profile {
    fn into(self) -> ffi::PP_VideoProfile {
        use self::Profile::*;
        match self {
            H264(v) => v.into(),
            VP9 => ffi::PP_VIDEOPROFILE_VP9_ANY,
            VP8 => ffi::PP_VIDEOPROFILE_VP8_ANY,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Acceleration {
    Hardware { with_fallback: bool },
    Software,
}
#[doc(hidden)]
impl Into<ffi::PP_HardwareAcceleration> for Acceleration {
    fn into(self) -> ffi::PP_HardwareAcceleration {
        match self {
            Acceleration::Hardware { with_fallback: false, } =>
                ffi::PP_HARDWAREACCELERATION_ONLY,
            Acceleration::Hardware { with_fallback: true,  } =>
                ffi::PP_HARDWAREACCELERATION_WITHFALLBACK,
            Acceleration::Software =>
                ffi::PP_HARDWAREACCELERATION_NONE,
        }
    }
}
impl Default for Acceleration {
    fn default() -> Acceleration { Acceleration::Hardware { with_fallback: true } }
}

pub struct Frame {
    pub tag: u32,
    texture: TextureBuffer,
    texture_target: gles::types::Enum,
    texture_size: super::Size,
    visible_rect: super::Rect,
}
impl Frame {
    pub fn format(&self) -> TexFormat { TexFormat::Rgba }
}
impl From<ffi::Struct_PP_VideoPicture> for Frame {
    fn from(raw: ffi::Struct_PP_VideoPicture) -> Frame {
        Frame {
            tag: raw.decode_id,
            texture: From::from(raw.texture_id),
            texture_target: From::from(raw.texture_target),
            texture_size: From::from(raw.texture_size),
            visible_rect: From::from(raw.visible_rect),
        }
    }
}
impl Into<ffi::Struct_PP_VideoPicture> for Frame {
    fn into(self) -> ffi::Struct_PP_VideoPicture {
        use gles::traits::Buffer;
        ffi::Struct_PP_VideoPicture {
            decode_id: self.tag,
            texture_id: self.texture.unwrap(),
            texture_target: self.texture_target,
            texture_size: self.texture_size.into(),
            visible_rect: self.visible_rect.into(),
        }
    }
}
impl VideoDecoder {
    pub fn initialize<F>(&self, g3d: Context3d, profile: Profile,
                     accel: Acceleration, callback: F) -> Code
        where F: Callback,
    {
        get_video_decoder_opt()
            .map(move |i| {
                let cc = callback.to_ffi_callback();
                let code = i.initialize(self.unwrap(), g3d.unwrap(), profile.into(),
                                        accel.into(), cc.cc());
                cc.drop_with_code(code)
            })
            .unwrap_or(Code::NoInterface)
    }
    /// PPAPI allocates it's own copy of the data before returning.
    pub fn decode<'a, F>(&self, decode_tag: u32, data: Cow<'a, [u8]>, callback: F) -> Code
        where F: Callback
    {
        get_video_decoder_opt()
            .map(move |i| {
                let cc = callback.to_ffi_callback();
                let code = i.decode(self.unwrap(), decode_tag,
                                    data.as_ref().len(), data.as_ref().as_ptr(), cc.cc());
                cc.drop_with_code(code)
            })
            .unwrap_or(Code::NoInterface)
    }
    pub fn flush<F>(&self, callback: F) -> Code where F: Callback {
        get_video_decoder_opt()
            .map(move |i| {
                let cc = callback.to_ffi_callback();
                let code = i.flush(self.unwrap(), cc.cc());
                cc.drop_with_code(code)
            })
            .unwrap_or(Code::NoInterface)
    }
    pub fn reset<F>(&self, callback: F) -> Code where F: Callback {
        get_video_decoder_opt()
            .map(move |i| {
                let cc = callback.to_ffi_callback();
                let code = i.reset(self.unwrap(), cc.cc());
                cc.drop_with_code(code)
            })
            .unwrap_or(Code::NoInterface)
    }
    pub fn get_picture<F>(&self, callback: F) -> Code<Frame>
        where F: CallbackArgs<Frame>
    {
        impl super::InPlaceInit for ffi::Struct_PP_VideoPicture { }
        fn arg_map(raw: ffi::Struct_PP_VideoPicture, _status: Code) -> Frame {
            From::from(raw)
        }
        get_video_decoder_opt()
            .map(move |i| {
                let raw: ffi::Struct_PP_VideoPicture = Default::default();
                let mapper = StorageToArgsMapper(arg_map);
                let cc = callback.to_ffi_callback(raw, mapper);
                let code = i.get_picture(self.unwrap(), cc.raw_args(), cc.cc());
                cc.drop_with_code(code)
            })
            .unwrap_or(Code::NoInterface)
    }
    pub fn recycle(&self, frame: Frame) {
        get_video_decoder_opt()
            .map(move |i| {
                let ffi: ffi::Struct_PP_VideoPicture =
                    frame.into();
                i.recycle_picture(self.unwrap(), &ffi)
            });
    }
}
