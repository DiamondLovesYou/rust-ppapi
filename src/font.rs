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

use std::intrinsics::uninit;
use std::result::{Ok};
use std::ptr::RawPtr;
use std::collections::hashmap::HashSet;

use super::{ppb, ffi};
use super::{ToVar, Resource, ToFFIBool};
use super::ffi::{Struct_PP_FontMetrics_Dev, Struct_PP_TextRun_Dev,
                 Struct_PP_FontDescription_Dev, PP_FontFamily_Dev};
use super::StringVar;


#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum Family {
    DefaultFamily,
    SerifFamily,
    SansSerifFamily,
    MonospaceFamily,
}
impl Family {
    fn new_from_ffi(v: ffi::PP_FontFamily_Dev) -> Family {
        match v {
            ffi::PP_FONTFAMILY_DEFAULT => DefaultFamily,
            ffi::PP_FONTFAMILY_SERIF => SerifFamily,
            ffi::PP_FONTFAMILY_SANSSERIF => SansSerifFamily,
            ffi::PP_FONTFAMILY_MONOSPACE => MonospaceFamily,
            _ => unreachable!(),
        }
    }
    fn to_ffi(&self) -> PP_FontFamily_Dev {
        match self {
            &DefaultFamily => ffi::PP_FONTFAMILY_DEFAULT,
            &SerifFamily => ffi::PP_FONTFAMILY_SERIF,
            &SansSerifFamily => ffi::PP_FONTFAMILY_SANSSERIF,
            &MonospaceFamily => ffi::PP_FONTFAMILY_MONOSPACE,
        }
    }
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum Weight {
    ValueWeight(u16),
    NormalWeight,
    BoldWeight,
}

impl Weight {
    fn new_from_ffi(v: ffi::PP_FontWeight_Dev) -> Weight {
        match v {
            ffi::PP_FONTWEIGHT_100 => ValueWeight(100),
            ffi::PP_FONTWEIGHT_200 => ValueWeight(200),
            ffi::PP_FONTWEIGHT_300 => ValueWeight(300),
            ffi::PP_FONTWEIGHT_400 => NormalWeight,
            ffi::PP_FONTWEIGHT_500 => ValueWeight(500),
            ffi::PP_FONTWEIGHT_600 => ValueWeight(600),
            ffi::PP_FONTWEIGHT_700 => BoldWeight,
            ffi::PP_FONTWEIGHT_800 => ValueWeight(800),
            ffi::PP_FONTWEIGHT_900 => ValueWeight(900),
            _ => unreachable!(),
        }
    }
    fn to_ffi(&self) -> ffi::PP_FontWeight_Dev {
        match self {
            &ValueWeight(v) if v <= 100 => ffi::PP_FONTWEIGHT_100,
            &ValueWeight(v) if v > 100 && v <= 200 => ffi::PP_FONTWEIGHT_200,
            &ValueWeight(v) if v > 200 && v <= 300 => ffi::PP_FONTWEIGHT_300,
            &ValueWeight(v) if v > 300 && v <= 400 => ffi::PP_FONTWEIGHT_400,
            &NormalWeight => ffi::PP_FONTWEIGHT_400,
            &ValueWeight(v) if v > 400 && v <= 500 => ffi::PP_FONTWEIGHT_500,
            &ValueWeight(v) if v > 500 && v <= 600 => ffi::PP_FONTWEIGHT_600,
            &ValueWeight(v) if v > 600 && v <= 700 => ffi::PP_FONTWEIGHT_700,
            &BoldWeight => ffi::PP_FONTWEIGHT_700,
            &ValueWeight(v) if v > 700 && v <= 800 => ffi::PP_FONTWEIGHT_800,
            &ValueWeight(_) => ffi::PP_FONTWEIGHT_900,
        }
    }
}
pub type Metrics = ffi::Struct_PP_FontMetrics_Dev;
impl Clone for super::ffi::Struct_PP_FontMetrics_Dev {
    fn clone(&self) -> Metrics {
        use core::mem::transmute_copy;
        unsafe {
            transmute_copy(self)
        }
    }
}
fn new_metrics_from_ffi(metrics: ffi::Struct_PP_FontMetrics_Dev) -> Metrics {
    metrics
}


#[deriving(Eq, PartialEq, Clone, Hash)]
pub struct Description {
    face: Option<StringVar>,
    family: Family,
    size: u32,
    weight: Weight,
    italic: bool,
    small_caps: bool,
    letter_spacing: i32,
    word_spacing: i32,
}
impl Description {
    pub fn new_from_family(fam: Family) -> Description {
        Description {
            face: None,
            family: fam,
            size: 12,
            weight: NormalWeight,
            italic: false,
            small_caps: false,
            letter_spacing: 0,
            word_spacing: 0,
        }
    }

    fn new_from_ffi(v: Struct_PP_FontDescription_Dev) -> Description {
        Description {
            face: Some(StringVar::new_from_var(v.face)),
            family: Family::new_from_ffi(v.family),
            size: v.size,
            weight: Weight::new_from_ffi(v.weight),
            italic: v.italic != 0,
            small_caps: v.small_caps != 0,
            letter_spacing: v.letter_spacing,
            word_spacing: v.word_spacing,
        }
    }
    pub unsafe fn to_ffi(&self) -> Struct_PP_FontDescription_Dev {
        let mut desc: Struct_PP_FontDescription_Dev = uninit();
        desc.face = self.face.to_var();
        desc.family = self.family.to_ffi();
        desc.size = self.size;
        desc.weight = self.weight.to_ffi();
        desc.italic = self.italic.to_ffi_bool();
        desc.small_caps = self.small_caps.to_ffi_bool();
        desc.letter_spacing = self.letter_spacing;
        desc
    }
}

impl super::Font {
    pub fn describe(&self) -> Option<(Description, Metrics)> {
        let mut desc: Struct_PP_FontDescription_Dev = Struct_PP_FontDescription_Dev {
            face: {super::NullVar}.to_var(),
            .. unsafe { uninit() }
        };
        let mut metr: Struct_PP_FontMetrics_Dev     = unsafe { uninit() };

        match (ppb::get_font().Describe.unwrap())
            (self.unwrap(),
             &mut desc as *mut Struct_PP_FontDescription_Dev,
             &mut metr as *mut Struct_PP_FontMetrics_Dev) {
            0 => None,
            _ => Some((Description::new_from_ffi(desc),
                       new_metrics_from_ffi(metr))),
        }
    }

    pub fn measure_text<T: super::ToStringVar +
        super::ToVar>(&self,
                      text: &T,
                      rtl: bool,
                      override_direction: bool) -> Option<i32> {
            let text_run = Struct_PP_TextRun_Dev {
                text: text.to_var(),
                rtl: rtl.to_ffi_bool(),
                override_direction: override_direction.to_ffi_bool(),
            };
            let result = (ppb::get_font().MeasureText.unwrap())
                (self.unwrap(),
                 &text_run as *const Struct_PP_TextRun_Dev);
            if result == -1 { None }
            else            { Some(result) }
        }

    /**!
     * Draws the text to the image buffer.
     *
     * The given point represents the baseline of the left edge of the font,
     * regardless of whether it is left-to-right or right-to-left (in the case of
     *      RTL text, this will actually represent the logical end of the text).
     *
     * The clip is optional and may be NULL. In this case, the text will be
     * clipped to the image.
     *
     * The image_data_is_opaque flag indicates whether subpixel antialiasing can
     * be performed, if it is supported. When the image below the text is
     * opaque, subpixel antialiasing is supported and you should set this to
     * PP_TRUE to pick up the user's default preferences. If your plugin is
     * partially transparent, then subpixel antialiasing is not possible and
     * grayscale antialiasing will be used instead (assuming the user has
     *      antialiasing enabled at all).
     */
    pub fn draw_text<TStr: super::ToStringVar +
        super::ToVar>(&self,
                      image: &super::ImageData,
                      text: &TStr,
                      rtl: bool,
                      override_direction: bool,
                      pos: super::Point,
                      color: u32,
                      clip: Option<super::Rect>,
                      image_data_is_opaque: bool) -> super::Code {
            let font_res = self.unwrap();
            let image_res = image.unwrap();
            let text_run = Struct_PP_TextRun_Dev {
                text: text.to_var(),
                rtl: rtl as u32,
                override_direction: override_direction as u32,
            };
            let pos_ptr = &pos as *const super::Point;
            let clip_ptr = if clip.is_some() { clip.get_ref() as *const super::Rect}
                           else              { RawPtr::null() };
            if (ppb::get_font().DrawTextAt.unwrap())
                (font_res,
                 image_res,
                 &text_run as *const Struct_PP_TextRun_Dev,
                 pos_ptr,
                 color,
                 clip_ptr,
                 image_data_is_opaque as ffi::PP_Bool) != -1 as u32 {
                super::Ok
            } else { super::Failed }
        }

    pub fn char_offset_for_pixel<TStr: super::ToStringVar +
        super::ToVar>(&self,
                      text: &TStr,
                      rtl: bool,
                      override_direction: bool,
                      position: i32) -> Option<u32> {
        let text_run = Struct_PP_TextRun_Dev {
            text: text.to_var(),
            rtl: rtl as u32,
            override_direction: override_direction as u32,
        };
        let result = (ppb::get_font().CharacterOffsetForPixel.unwrap())
                (self.unwrap(),
                 &text_run as *const Struct_PP_TextRun_Dev,
                 position);
        if result == -1 as u32 {
            None
        } else {
            Some(result)
        }
    }

    pub fn pixel_offset_for_character<TStr: super::ToStringVar +
        super::ToVar>(&self,
                      text: &TStr,
                      rtl: bool,
                      override_direction: bool,
                      char_offset: u32) -> Option<i32> {
        let text_run = Struct_PP_TextRun_Dev {
            text: text.to_var(),
            rtl: rtl as u32,
            override_direction: override_direction as u32,
        };
        let result = (ppb::get_font().PixelOffsetForCharacter.unwrap())
                (self.unwrap(),
                 &text_run as *const Struct_PP_TextRun_Dev,
                 char_offset);
        if result == -1 { None }
        else            { Some(result) }
    }
}

impl super::Instance {
    pub fn get_font_families(&self) -> HashSet<String> {
        let mut dest = HashSet::new();
        let fam_str = (ppb::get_font().GetFontFamilies.unwrap())(self.instance);
        let fam_str = StringVar::new_from_var(fam_str).to_string();
        for font in fam_str.as_slice().split('\0') {
            dest.insert(font.to_string());
        }
        dest
    }
}
