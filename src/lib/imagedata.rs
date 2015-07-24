// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use libc::c_void;
use std::ops;

use super::ffi;
use super::{Resource};
use super::ppb;
use ppb::ImageDataIf;

#[derive(Hash, Eq, PartialEq, Debug)] pub struct ImageData(ffi::PP_Resource);

impl_resource_for!(ImageData, ResourceType::ImageData);

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum Format {
    BGRA = ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL as isize,
    RGBA = ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL as isize,
}
impl Format {
    fn from_ffi(v: ffi::PP_ImageDataFormat) -> Format {
        match v {
            ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL => Format::BGRA,
            ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL => Format::RGBA,
            _ => panic!(),
        }
    }
    pub fn to_ffi(&self) -> ffi::PP_ImageDataFormat {
        match *self {
            Format::BGRA => ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL,
            Format::RGBA => ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL,
        }
    }
    pub fn is_supported(&self) -> bool {
        let ffi_val = *self as ffi::PP_ImageDataFormat;
        (ppb::get_image_data().IsImageDataFormatSupported.unwrap())(ffi_val) != 0
    }
}

#[derive(Clone, Copy)]
pub struct Description {
    pub format: Format,
    pub size: super::Size,

    /** Taken from the Google PPAPI docs
     * This value represents the row width in bytes. This may be different than
     * width * 4 since there may be padding at the end of the lines.
     */
    pub line_stride: u32,
}
impl Description {
    pub fn from_ffi(desc: ffi::Struct_PP_ImageDataDesc) -> Description {
        use std::mem::transmute;
        Description {
            format: Format::from_ffi(desc.format),
            size:   unsafe { transmute(desc.size) },
            line_stride: desc.stride as u32,
        }
    }
}

pub struct MappedImage<'a> {
    img: &'a ImageData,
    pub desc: Description,
    ptr: *mut c_void,
}
pub trait MappedSlice<'a> {
    fn as_imm_slice(&self) -> &'a [u8];
}
impl<'a> MappedSlice<'a> for MappedImage<'a> {
    fn as_imm_slice(&self) -> &'a [u8] {
        use std::slice::from_raw_parts;
        use std::mem::transmute;
        let size = (self.desc.size.height * self.desc.line_stride) as usize;

        unsafe { from_raw_parts(transmute(&self.ptr), size) }
    }
}
impl<'a> ops::Drop for MappedImage<'a> {
    fn drop(&mut self) {
        ppb::get_image_data().unmap(&self.img.unwrap());
    }
}

pub fn native_image_data_format() -> Format {
    Format::from_ffi(ppb::get_image_data().native_image_data_format())
}

impl ImageData {
    pub fn describe(&self) -> Option<Description> {
        ppb::get_image_data()
            .describe(self.unwrap())
            .map(|desc| Description::from_ffi(desc) )
    }
    pub fn map<'a>(&'a self) -> MappedImage<'a> {
        MappedImage {
            img: self,
            desc: self.describe().unwrap(),
            ptr: ppb::get_image_data().map(&self.unwrap()),
        }
    }
}
