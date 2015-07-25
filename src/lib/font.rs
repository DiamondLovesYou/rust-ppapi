// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem;
use std::ptr;
use std::collections::HashSet;

use super::{ppb, ffi};
use super::{ToVar, Resource, ToFFIBool};
use super::ffi::{Struct_PP_FontMetrics_Dev, Struct_PP_TextRun_Dev,
                 Struct_PP_FontDescription_Dev, PP_FontFamily_Dev};
use super::StringVar;

use imagedata;


#[derive(Eq, PartialEq, Clone, Hash, Copy)]
pub enum Family {
    DefaultFamily,
    SerifFamily,
    SansSerifFamily,
    MonospaceFamily,
}
impl Family {
    fn new_from_ffi(v: ffi::PP_FontFamily_Dev) -> Family {
        match v {
            ffi::PP_FONTFAMILY_DEFAULT => Family::DefaultFamily,
            ffi::PP_FONTFAMILY_SERIF => Family::SerifFamily,
            ffi::PP_FONTFAMILY_SANSSERIF => Family::SansSerifFamily,
            ffi::PP_FONTFAMILY_MONOSPACE => Family::MonospaceFamily,
            _ => unreachable!(),
        }
    }
    fn to_ffi(&self) -> PP_FontFamily_Dev {
        match self {
            &Family::DefaultFamily => ffi::PP_FONTFAMILY_DEFAULT,
            &Family::SerifFamily => ffi::PP_FONTFAMILY_SERIF,
            &Family::SansSerifFamily => ffi::PP_FONTFAMILY_SANSSERIF,
            &Family::MonospaceFamily => ffi::PP_FONTFAMILY_MONOSPACE,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Copy)]
pub enum Weight {
    ValueWeight(u16),
    NormalWeight,
    BoldWeight,
}

impl Weight {
    fn new_from_ffi(v: ffi::PP_FontWeight_Dev) -> Weight {
        match v {
            ffi::PP_FONTWEIGHT_100 => Weight::ValueWeight(100),
            ffi::PP_FONTWEIGHT_200 => Weight::ValueWeight(200),
            ffi::PP_FONTWEIGHT_300 => Weight::ValueWeight(300),
            ffi::PP_FONTWEIGHT_400 => Weight::NormalWeight,
            ffi::PP_FONTWEIGHT_500 => Weight::ValueWeight(500),
            ffi::PP_FONTWEIGHT_600 => Weight::ValueWeight(600),
            ffi::PP_FONTWEIGHT_700 => Weight::BoldWeight,
            ffi::PP_FONTWEIGHT_800 => Weight::ValueWeight(800),
            ffi::PP_FONTWEIGHT_900 => Weight::ValueWeight(900),
            _ => unreachable!(),
        }
    }
    fn to_ffi(&self) -> ffi::PP_FontWeight_Dev {
        match self {
            &Weight::ValueWeight(v) if v <= 100 => ffi::PP_FONTWEIGHT_100,
            &Weight::ValueWeight(v) if v > 100 && v <= 200 => ffi::PP_FONTWEIGHT_200,
            &Weight::ValueWeight(v) if v > 200 && v <= 300 => ffi::PP_FONTWEIGHT_300,
            &Weight::ValueWeight(v) if v > 300 && v <= 400 => ffi::PP_FONTWEIGHT_400,
            &Weight::NormalWeight => ffi::PP_FONTWEIGHT_400,
            &Weight::ValueWeight(v) if v > 400 && v <= 500 => ffi::PP_FONTWEIGHT_500,
            &Weight::ValueWeight(v) if v > 500 && v <= 600 => ffi::PP_FONTWEIGHT_600,
            &Weight::ValueWeight(v) if v > 600 && v <= 700 => ffi::PP_FONTWEIGHT_700,
            &Weight::BoldWeight => ffi::PP_FONTWEIGHT_700,
            &Weight::ValueWeight(v) if v > 700 && v <= 800 => ffi::PP_FONTWEIGHT_800,
            &Weight::ValueWeight(_) => ffi::PP_FONTWEIGHT_900,
        }
    }
}
pub type Metrics = ffi::Struct_PP_FontMetrics_Dev;
fn new_metrics_from_ffi(metrics: ffi::Struct_PP_FontMetrics_Dev) -> Metrics {
    metrics
}


#[derive(Eq, PartialEq, Clone, Hash)]
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
            weight: Weight::NormalWeight,
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
        let mut desc: Struct_PP_FontDescription_Dev = mem::uninitialized();
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

#[derive(Hash, Eq, PartialEq, Debug)] pub struct Font(ffi::PP_Resource);

impl_resource_for!(Font, ResourceType::Font);

impl Font {
    pub fn describe(&self) -> Option<(Description, Metrics)> {
        let mut desc: Struct_PP_FontDescription_Dev = Struct_PP_FontDescription_Dev {
            face: {super::NullVar}.to_var(),
            .. unsafe { mem::uninitialized() }
        };
        let mut metr: Struct_PP_FontMetrics_Dev     = unsafe { mem::uninitialized() };

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
    pub fn draw_text<TStr: super::ToStringVar + super::ToVar>
        (&self,
         image: &imagedata::ImageData,
         text: &TStr,
         rtl: bool,
         override_direction: bool,
         pos: super::Point,
         color: u32,
         clip: Option<super::Rect>,
         image_data_is_opaque: bool) -> bool
    {
        let font_res = self.unwrap();
        let image_res = image.unwrap();
        let text_run = Struct_PP_TextRun_Dev {
            text: text.to_var(),
            rtl: rtl as u32,
            override_direction: override_direction as u32,
        };
        let pos: ffi::PP_Point = pos.into();
        let pos_ptr = &pos as *const ffi::PP_Point;
        let clip: Option<ffi::PP_Rect> = clip.map(|c| c.into() );
        let clip_ptr = if clip.is_some() {
            clip.as_ref().unwrap() as *const _
        } else {
            ptr::null()
        };
        if (ppb::get_font().DrawTextAt.unwrap())
            (font_res,
             image_res,
             &text_run as *const Struct_PP_TextRun_Dev,
             pos_ptr,
             color,
             clip_ptr,
             image_data_is_opaque as ffi::PP_Bool) != ffi::PP_FALSE
        {
            true
        } else {
            false
        }
    }

    pub fn char_offset_for_pixel<TStr: super::ToStringVar + super::ToVar>
        (&self,
         text: &TStr,
         rtl: bool,
         override_direction: bool,
         position: i32) -> Option<u32>
    {
        let text_run = Struct_PP_TextRun_Dev {
            text: text.to_var(),
            rtl: rtl as u32,
            override_direction: override_direction as u32,
        };
        let result = (ppb::get_font().CharacterOffsetForPixel.unwrap())
                (self.unwrap(),
                 &text_run as *const Struct_PP_TextRun_Dev,
                 position);
        if result as i32 == -1 {
            None
        } else {
            Some(result)
        }
    }

    pub fn pixel_offset_for_character<TStr: super::ToStringVar + super::ToVar>
        (&self,
         text: &TStr,
         rtl: bool,
         override_direction: bool,
         char_offset: u32) -> Option<i32>
    {
        let text_run = Struct_PP_TextRun_Dev {
            text: text.to_var(),
            rtl: rtl as u32,
            override_direction: override_direction as u32,
        };
        let result = (ppb::get_font().PixelOffsetForCharacter.unwrap())
            (self.unwrap(),
             &text_run as *const Struct_PP_TextRun_Dev,
             char_offset);
        if result as i32 == -1 { None }
        else                   { Some(result) }
    }
}
pub trait FontFamilies {
    fn get_font_families(&self) -> HashSet<String>;
}
impl FontFamilies for super::Instance {
    fn get_font_families(&self) -> HashSet<String> {
        use std::borrow::ToOwned;
        let mut dest = HashSet::new();
        let fam_str = (ppb::get_font().GetFontFamilies.unwrap())(self.instance);
        let fam_str = StringVar::new_from_var(fam_str);
        for font in fam_str.as_ref().split('\0') {
            dest.insert(font.to_owned());
        }
        dest
    }
}
