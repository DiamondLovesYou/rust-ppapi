// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// Rust PPApi is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Rust PPApi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Rust PPApi. If not, see <http://www.gnu.org/licenses/>.

use libc::c_void;
use std::ops;
use std::rc::Rc;

use super::ffi;
use super::{Resource};
use super::ppb;
use ppb::ImageDataIf;

#[deriving(Hash, Eq, PartialEq, Show)] pub struct ImageData(ffi::PP_Resource);

impl_resource_for!(ImageData ImageDataRes)

#[deriving(Eq, PartialEq, Hash, Clone)]
pub enum Format {
    BGRA = ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL as int,
    RGBA = ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL as int,
}
impl Format {
    fn from_ffi(v: ffi::PP_ImageDataFormat) -> Format {
        match v {
            ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL => BGRA,
            ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL => RGBA,
            _ => fail!(),
        }
    }
    pub fn to_ffi(&self) -> ffi::PP_ImageDataFormat {
        match *self {
            BGRA => ffi::PP_IMAGEDATAFORMAT_BGRA_PREMUL,
            RGBA => ffi::PP_IMAGEDATAFORMAT_RGBA_PREMUL,
        }
    }
    pub fn is_supported(&self) -> bool {
        let ffi_val = *self as ffi::PP_ImageDataFormat;
        (ppb::get_image_data().IsImageDataFormatSupported.unwrap())(ffi_val) != 0
    }
}

#[deriving(Clone)]
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
        use core::mem::transmute;
        Description {
            format: Format::from_ffi(desc.format),
            size:   unsafe { transmute(desc.size) },
            line_stride: desc.stride as u32,
        }
    }
}

pub struct Map_ {
    pub img: ImageData,
    pub desc: Description,
    ptr: *mut c_void,
}
pub type Map = Rc<Map_>;
pub trait MapImpl {
    fn with_imm_vec<U>(&self, f: |&Vec<u8>, &Description| -> U) -> U;
}
impl MapImpl for Rc<Map_> {
    fn with_imm_vec<U>(&self, f: |&Vec<u8>, &Description| -> U) -> U {
        use core::mem::forget;
        let size = ((**self).desc.size.height * (**self).desc.line_stride) as uint;
        let v = unsafe { Vec::from_raw_parts(size,
                                             size,
                                             (**self).ptr as *mut u8) };
        let ret = f(&v, &(**self).desc);
        unsafe {
            forget(v);
        }
        ret
    }
}

impl ops::Drop for Map_ {
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
    pub fn map(&self) -> Map {
        Rc::new(Map_ {
            img: self.clone(),
            desc: self.describe().unwrap(),
            ptr: ppb::get_image_data().map(&self.unwrap()),
        })
    }
}
