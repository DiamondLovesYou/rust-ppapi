// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! PPB related interfaces. Many interfaces have convenience functions to remove much
//! of the verbose-ness of the originals.

#![allow(missing_docs)]
use std::mem;
use std::mem::uninitialized;
use libc;

use super::ffi;
use super::ffi::{Struct_PP_Var, PP_Instance, PP_LogLevel, PP_Resource,
                 Struct_PP_CompletionCallback, PP_Var};
use super::{Ticks, Time, ToFFIBool, Code, Result, ToVar};

pub type Var = ffi::PPB_Var;
pub type Core = ffi::PPB_Core;
pub type Console = ffi::PPB_Console;
pub type VarArray = ffi::Struct_PPB_VarArray_1_0;
pub type VarArrayBuffer = ffi::PPB_VarArrayBuffer;
pub type VarDictionary  = ffi::Struct_PPB_VarDictionary_1_0;
pub type Graphics3D = ffi::PPB_Graphics3D;
pub type Messaging = ffi::Struct_PPB_Messaging_1_2;
pub type MessageLoop = ffi::PPB_MessageLoop;
pub type Instance = ffi::PPB_Instance;
pub type InputEvent = ffi::PPB_InputEvent;
pub type KeyboardInputEvent = ffi::PPB_KeyboardInputEvent;
pub type MouseInputEvent = ffi::PPB_MouseInputEvent;
pub type TouchInputEvent = ffi::PPB_TouchInputEvent;
pub type IMEInputEvent = ffi::PPB_IMEInputEvent;
pub type OpenGLES2 = ffi::PPB_OpenGLES2;
pub type WheelInputEvent = ffi::PPB_WheelInputEvent;
pub type Font = ffi::PPB_Font_Dev;
pub type ImageData = ffi::PPB_ImageData;
pub type UrlLoader = ffi::PPB_URLLoader;
pub type UrlRequestInfo = ffi::PPB_URLRequestInfo;
pub type UrlResponseInfo = ffi::PPB_URLResponseInfo;
pub type View = ffi::Struct_PPB_View_1_2;
pub type FileSystem = ffi::Struct_PPB_FileSystem_1_0;
pub type FileRef = ffi::Struct_PPB_FileRef_1_2;
pub type FileIo = ffi::Struct_PPB_FileIO_1_1;
pub type MediaStreamVideoTrack = ffi::Struct_PPB_MediaStreamVideoTrack_0_1;
pub type VideoFrame = ffi::Struct_PPB_VideoFrame_0_1;
pub type VideoDecoder = ffi::Struct_PPB_VideoDecoder_1_0;

mod consts {
    pub const VAR: &'static str              = "PPB_Var;1.1\0";
    pub const CORE: &'static str             = "PPB_Core;1.0\0";
    pub const CONSOLE: &'static str          = "PPB_Console;1.0\0";
    pub const MESSAGING: &'static str        = "PPB_Messaging;1.2\0";
    pub const MESSAGELOOP: &'static str      = "PPB_MessageLoop;1.0\0";
    pub const VAR_ARRAY: &'static str        = "PPB_VarArray;1.0\0";
    pub const VAR_ARRAY_BUFFER: &'static str = "PPB_VarArrayBuffer;1.0\0";
    pub const VAR_DICTIONARY: &'static str   = "PPB_VarDictionary;1.0\0";
    pub const GRAPHICS_3D: &'static str      = "PPB_Graphics3D;1.0\0";
    pub const INSTANCE: &'static str         = "PPB_Instance;1.0\0";
    pub const INPUT:    &'static str         = "PPB_InputEvent;1.0\0";
    pub const KEYBOARD: &'static str         = "PPB_KeyboardInputEvent;1.2\0";
    pub const MOUSE:    &'static str         = "PPB_MouseInputEvent;1.1\0";
    pub const WHEEL:    &'static str         = "PPB_WheelInputEvent;1.0\0";
    pub const TOUCH:    &'static str         = "PPB_TouchInputEvent;1.0\0";
    pub const IME:      &'static str         = "PPB_IMEInputEvent;1.0\0";
    pub const GLES2:    &'static str         = "PPB_OpenGLES2;1.0\0";
    pub const FONTDEV:  &'static str         = "PPB_Font(Dev);0.6\0";
    pub const IMAGEDATA:&'static str         = "PPB_ImageData;1.0\0";
    pub const URL_LOADER: &'static str       = "PPB_URLLoader;1.0\0";
    pub const URL_REQUEST: &'static str      = "PPB_URLRequestInfo;1.0\0";
    pub const URL_RESPONSE: &'static str     = "PPB_URLResponseInfo;1.0\0";
    pub const VIEW:     &'static str         = "PPB_View;1.2\0";
    pub const FILESYSTEM: &'static str       = "PPB_FileSystem;1.0\0";
    pub const FILEREF: &'static str          = "PPB_FileRef;1.2\0";
    pub const FILEIO: &'static str           = "PPB_FileIo;1.1\0";
    pub const MEDIA_STREAM_VIDEO_TRACK: &'static str = "PPB_MediaStreamVideoTrack;0.1\0";
    pub const VIDEO_FRAME: &'static str      = "PPB_VideoFrame;0.1\0";
    pub const VIDEO_DECODER: &'static str    = "PPB_VideoDecoder;1.0\0";
}
mod globals {
    use super::super::ffi;
    pub static mut BROWSER:      ffi::PPB_GetInterface = None;
    pub static mut VAR:          Option<&'static super::Var> = None;
    pub static mut CORE:         Option<&'static super::Core> = None;
    pub static mut CONSOLE:      Option<&'static super::Console> = None;
    pub static mut ARRAY:        Option<&'static super::VarArray> = None;
    pub static mut ARRAY_BUFFER: Option<&'static super::VarArrayBuffer> = None;
    pub static mut DICTIONARY:   Option<&'static super::VarDictionary>  = None;
    pub static mut GRAPHICS_3D:  Option<&'static super::Graphics3D> = None;
    pub static mut MESSAGING:    Option<&'static super::Messaging> = None;
    pub static mut MESSAGE_LOOP: Option<&'static super::MessageLoop> = None;
    pub static mut INSTANCE:     Option<&'static super::Instance> = None;
    pub static mut INPUT:        Option<&'static super::InputEvent> = None;
    pub static mut KEYBOARD:     Option<&'static super::KeyboardInputEvent> = None;
    pub static mut MOUSE:        Option<&'static super::MouseInputEvent> = None;
    pub static mut WHEEL:        Option<&'static super::WheelInputEvent> = None;
    pub static mut TOUCH:        Option<&'static super::TouchInputEvent> = None;
    pub static mut IME:          Option<&'static super::IMEInputEvent> = None;
    pub static mut GLES2:        Option<&'static super::OpenGLES2> = None;
    pub static mut FONTDEV:      Option<&'static super::Font> = None;
    pub static mut IMAGEDATA:    Option<&'static super::ImageData> = None;
    pub static mut URL_LOADER:   Option<&'static super::UrlLoader> = None;
    pub static mut URL_REQUEST:  Option<&'static super::UrlRequestInfo> = None;
    pub static mut URL_RESPONSE: Option<&'static super::UrlResponseInfo> = None;
    pub static mut VIEW:         Option<&'static super::View> = None;
    pub static mut FILESYSTEM:   Option<&'static super::FileSystem> = None;
    pub static mut FILEREF:      Option<&'static super::FileRef> = None;
    pub static mut FILEIO:       Option<&'static super::FileIo> = None;
    pub static mut MEDIA_STREAM_VIDEO_TRACK: Option<&'static super::MediaStreamVideoTrack> = None;
    pub static mut VIDEO_FRAME:  Option<&'static super::VideoFrame> = None;
    pub static mut VIDEO_DECODER: Option<&'static super::VideoDecoder> = None;
}
#[cold] #[inline(never)] #[doc(hidden)]
pub fn initialize_globals(b: ffi::PPB_GetInterface) {
    unsafe {
        globals::BROWSER       = b;
        globals::VAR           = get_interface(consts::VAR);
        globals::CONSOLE       = get_interface(consts::CONSOLE);
        globals::CORE          = get_interface(consts::CORE);
        globals::ARRAY         = get_interface(consts::VAR_ARRAY);
        globals::ARRAY_BUFFER  = get_interface(consts::VAR_ARRAY_BUFFER);
        globals::DICTIONARY    = get_interface(consts::VAR_DICTIONARY);
        globals::GRAPHICS_3D   = get_interface(consts::GRAPHICS_3D);
        globals::MESSAGING     = get_interface(consts::MESSAGING);
        globals::MESSAGE_LOOP  = get_interface(consts::MESSAGELOOP);
        globals::INSTANCE      = get_interface(consts::INSTANCE);
        globals::INPUT         = get_interface(consts::INPUT);
        globals::KEYBOARD      = get_interface(consts::KEYBOARD);
        globals::MOUSE         = get_interface(consts::MOUSE);
        globals::WHEEL         = get_interface(consts::WHEEL);
        globals::TOUCH         = get_interface(consts::TOUCH);
        globals::IME           = get_interface(consts::IME);
        globals::GLES2         = get_interface(consts::GLES2);
        globals::FONTDEV       = get_interface(consts::FONTDEV);
        globals::IMAGEDATA     = get_interface(consts::IMAGEDATA);
        globals::URL_LOADER    = get_interface(consts::URL_LOADER);
        globals::URL_REQUEST   = get_interface(consts::URL_REQUEST);
        globals::URL_RESPONSE  = get_interface(consts::URL_RESPONSE);
        globals::VIEW          = get_interface(consts::VIEW);
        globals::FILESYSTEM    = get_interface(consts::FILESYSTEM);
        globals::FILEREF       = get_interface(consts::FILEREF);
        globals::FILEIO        = get_interface(consts::FILEIO);
        globals::MEDIA_STREAM_VIDEO_TRACK = get_interface(consts::MEDIA_STREAM_VIDEO_TRACK);
        globals::VIDEO_FRAME   = get_interface(consts::VIDEO_FRAME);
        globals::VIDEO_DECODER = get_interface(consts::VIDEO_DECODER);
    }
}
/// Get the PPB_GetInterface function pointer.
#[inline(never)]
pub unsafe fn get_actual_browser() -> extern "C" fn(*const i8) -> *const libc::c_void {
    globals::BROWSER.expect("Browser GetInterface missing")
}
fn get_interface<T>(name: &'static str) -> Option<&'static T> {
    // we actually have to use a null-terminated str here.
    unsafe {
        let ptr = get_actual_browser()(name.as_ptr() as *const i8) as *const T;

        if ptr.is_null() { None }
        else             { Some(mem::transmute(ptr)) }
    }
}
macro_rules! get_fun(
    (pub fn $ident:ident() -> $ty:ty { $global:ident }) => (
        #[doc = "Returns a static ref to the interface"]
        pub fn $ident() -> &'static $ty {
            #[inline(never)] fn failure() -> ! {
                panic!("Missing browser {} interface", stringify!($ty))
            }
            unsafe {
                if globals::$global.is_none() {
                    failure()
                } else { globals::$global.unwrap() }
            }
        }
    );
);
macro_rules! get_fun_opt(
    (pub fn $ident:ident() -> $ty:ty { $global:ident }) => (
        #[doc = "Returns an optional static ref to the interface"]
        pub fn $ident() -> Option<&'static $ty> {
            unsafe {
                globals::$global
            }
        }
    );
);

get_fun!    (pub fn get_var() -> Var { VAR });
get_fun_opt!(pub fn get_var_opt() -> Var { VAR });
get_fun!    (pub fn get_core() -> Core { CORE });
get_fun_opt!(pub fn get_core_opt() -> Core { CORE });
get_fun!    (pub fn get_console() -> Console { CONSOLE });
get_fun_opt!(pub fn get_console_opt() -> Console { CONSOLE });
get_fun!    (pub fn get_array() -> VarArray { ARRAY });
get_fun_opt!(pub fn get_array_opt() -> VarArray { ARRAY });
get_fun!    (pub fn get_array_buffer() -> VarArrayBuffer { ARRAY_BUFFER });
get_fun_opt!(pub fn get_array_buffer_opt() -> VarArrayBuffer { ARRAY_BUFFER });
get_fun!    (pub fn get_dictionary() -> VarDictionary { DICTIONARY });
get_fun_opt!(pub fn get_dictionary_opt() -> VarDictionary { DICTIONARY });
get_fun!    (pub fn get_graphics_3d() -> Graphics3D { GRAPHICS_3D });
get_fun_opt!(pub fn get_graphics_3d_opt() -> Graphics3D { GRAPHICS_3D });
get_fun!    (pub fn get_messaging() -> Messaging { MESSAGING });
get_fun_opt!(pub fn get_messaging_opt() -> Messaging { MESSAGING });
get_fun!    (pub fn get_message_loop() -> MessageLoop { MESSAGE_LOOP });
get_fun_opt!(pub fn get_message_loop_opt() -> MessageLoop { MESSAGE_LOOP });
get_fun!    (pub fn get_instance() -> Instance { INSTANCE });
get_fun_opt!(pub fn get_instance_opt() -> Instance { INSTANCE });
get_fun!    (pub fn get_input_event() -> InputEvent { INPUT });
get_fun_opt!(pub fn get_input_event_opt() -> InputEvent { INPUT });
get_fun!    (pub fn get_keyboard_event() -> KeyboardInputEvent { KEYBOARD });
get_fun_opt!(pub fn get_keyboard_event_opt() -> KeyboardInputEvent { KEYBOARD });
get_fun!    (pub fn get_mouse_event() -> MouseInputEvent { MOUSE });
get_fun_opt!(pub fn get_mouse_event_opt() -> MouseInputEvent { MOUSE });
get_fun!    (pub fn get_wheel_event() -> WheelInputEvent { WHEEL });
get_fun_opt!(pub fn get_wheel_event_opt() -> WheelInputEvent { WHEEL });
get_fun!    (pub fn get_touch_event() -> TouchInputEvent { TOUCH });
get_fun_opt!(pub fn get_touch_event_opt() -> TouchInputEvent { TOUCH });
get_fun!    (pub fn get_ime_event() -> IMEInputEvent { IME });
get_fun_opt!(pub fn get_ime_event_opt() -> IMEInputEvent { IME });
get_fun!    (pub fn get_gles2() -> OpenGLES2 { GLES2 });
get_fun_opt!(pub fn get_gles2_opt() -> OpenGLES2 { GLES2 });
get_fun!    (pub fn get_font() -> Font { FONTDEV });
get_fun_opt!(pub fn get_font_opt() -> Font { FONTDEV });
get_fun!    (pub fn get_image_data() -> ImageData { IMAGEDATA });
get_fun_opt!(pub fn get_image_data_opt() -> ImageData { IMAGEDATA });
get_fun!    (pub fn get_url_loader() -> UrlLoader { URL_LOADER });
get_fun_opt!(pub fn get_url_loader_opt() -> UrlLoader { URL_LOADER });
get_fun!    (pub fn get_url_request() -> UrlRequestInfo { URL_REQUEST });
get_fun_opt!(pub fn get_url_request_opt() -> UrlRequestInfo { URL_REQUEST });
get_fun!    (pub fn get_url_response() -> UrlResponseInfo { URL_RESPONSE });
get_fun_opt!(pub fn get_url_response_opt() -> UrlResponseInfo { URL_RESPONSE });
get_fun!    (pub fn get_view() -> View { VIEW });
get_fun_opt!(pub fn get_view_opt() -> View { VIEW });
get_fun!    (pub fn get_file_system() -> FileSystem { FILESYSTEM });
get_fun_opt!(pub fn get_file_system_opt() -> FileSystem { FILESYSTEM });
get_fun!    (pub fn get_file_ref() -> FileRef { FILEREF });
get_fun_opt!(pub fn get_file_ref_opt() -> FileRef { FILEREF });
get_fun!    (pub fn get_file_io() -> FileIo { FILEIO });
get_fun_opt!(pub fn get_file_io_opt() -> FileIo { FILEIO });
get_fun!    (pub fn get_media_stream_video_track() -> MediaStreamVideoTrack { MEDIA_STREAM_VIDEO_TRACK });
get_fun_opt!(pub fn get_media_stream_video_track_opt() -> MediaStreamVideoTrack { MEDIA_STREAM_VIDEO_TRACK });
get_fun!    (pub fn get_video_frame() -> VideoFrame { VIDEO_FRAME });
get_fun_opt!(pub fn get_video_frame_opt() -> VideoFrame { VIDEO_FRAME });
get_fun!    (pub fn get_video_decoder() -> VideoDecoder { VIDEO_DECODER });
get_fun_opt!(pub fn get_video_decoder_opt() -> VideoDecoder { VIDEO_DECODER });

macro_rules! impl_fun(
    ($fun:expr => ( $($arg:expr),* ) ) => ({
        #[inline(never)] fn failure() -> ! {
            panic!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = if $fun.is_none() { failure() }
                else { $fun.unwrap() };
        f( $($arg),* )
    });
    ($fun:expr => ( $($arg:expr),* ) -> Option<PP_Resource> ) => ({
        let r = impl_fun!($fun => ( $($arg),* ));
        if r == 0 { None }
        else { Some(r) }
    });
    ($fun:expr => ( $($arg:expr),* ) -> Code ) => ({
        let r = impl_fun!($fun => ( $($arg),* ));
        From::from(r)
    });
    ($fun:expr) => ({
        #[inline(never)] fn failure() -> ! {
            panic!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = if $fun.is_none() { failure() }
                else { $fun.unwrap() };
        f()
    })
);

pub trait ResourceInterface {
    #[doc(hidden)]
    fn get_is_fn(&self) -> Option<extern "C" fn(resource: ffi::PP_Resource)
                                                -> ffi::PP_Bool>;
    fn is(&self, r: ffi::PP_Resource) -> bool {
        self.get_is_fn()
            .map(|f| f(r) != ffi::PP_FALSE )
            .unwrap_or(false)
    }
}
macro_rules! resource_interface(
    (impl for $ty:ty => $is_name:ident) => {
        impl ResourceInterface for $ty {
            fn get_is_fn(&self) -> Option<extern "C" fn(resource: ffi::PP_Resource)
                                                        -> ffi::PP_Bool> {
                self.$is_name
            }
        }
    }
);
macro_rules! resource_interface_opt(
    (impl for $ty:ty => $is_name:ident) => {
        impl ResourceInterface for Option<&'static $ty> {
            fn get_is_fn(&self) -> Option<extern "C" fn(resource: ffi::PP_Resource)
                                                        -> ffi::PP_Bool> {
                self.and_then(|s| s.$is_name )
            }
        }
    }
);

pub trait CoreIf {
    fn get_time_ticks(&self) -> Ticks;
    fn get_time(&self) -> Time;
}
impl CoreIf for ffi::Struct_PPB_Core_1_0 {
    fn get_time_ticks(&self) -> Ticks {
        impl_fun!(self.GetTimeTicks)
    }
    fn get_time(&self) -> Time {
        impl_fun!(self.GetTime)
    }
}
pub trait VarIf {
    fn add_ref(&self, var: &Struct_PP_Var);
    fn remove_ref(&self, var: Struct_PP_Var);
    fn var_from_utf8(&self, string: &str) -> Struct_PP_Var;
    fn var_to_utf8(&self, string: &Struct_PP_Var) -> String;
}
impl VarIf for ffi::Struct_PPB_Var_1_2 {
    fn add_ref(&self, var: &Struct_PP_Var) {
        impl_fun!(self.AddRef => (*var))
    }
    fn remove_ref(&self, var: Struct_PP_Var) {
        impl_fun!(self.Release => (var))
    }

    fn var_from_utf8(&self, string: &str) -> Struct_PP_Var {
        impl_fun!(self.VarFromUtf8 => (string.as_ptr() as *const i8, string.len() as u32))
    }
    fn var_to_utf8(&self, string: &Struct_PP_Var) -> String {
        use std::slice::from_raw_parts;
        use std::str::from_utf8_unchecked;
        let mut len: u32 = unsafe { mem::uninitialized() };
        let ptr = impl_fun!(self.VarToUtf8 => (*string, &mut len as *mut u32)) as *const u8;
        let buf = unsafe { from_raw_parts(ptr, len as usize) };
        let slice = unsafe { from_utf8_unchecked(buf) };
        slice.to_string()
    }
}
pub trait ConsoleIf {
    fn log(&self, inst: PP_Instance, lvl: PP_LogLevel, msg: &Struct_PP_Var);
    fn log_with_source(&self,
                       inst: PP_Instance,
                       lvl: PP_LogLevel,
                       src: &Struct_PP_Var,
                       msg: &Struct_PP_Var);
}
impl ConsoleIf for ffi::Struct_PPB_Console_1_0 {
    fn log(&self, inst: PP_Instance, lvl: PP_LogLevel, msg: &Struct_PP_Var) {
        impl_fun!(self.Log => (inst, lvl, *msg))
    }
    fn log_with_source(&self,
                       inst: PP_Instance,
                       lvl: PP_LogLevel,
                       src: &Struct_PP_Var,
                       msg: &Struct_PP_Var) {
        impl_fun!(self.LogWithSource => (inst, lvl, *src, *msg))
    }
}
pub trait VarArrayIf {
    fn create(&self) -> Struct_PP_Var;
    fn get(&self, array: &Struct_PP_Var, index: libc::uint32_t) -> Struct_PP_Var;
    fn set(&self, array: &Struct_PP_Var, index: libc::uint32_t, value: &Struct_PP_Var) -> bool;
    fn get_len(&self, array: &Struct_PP_Var) -> libc::uint32_t;
    fn set_len(&self, array: &Struct_PP_Var, new_len: libc::uint32_t) -> bool;
}
impl VarArrayIf for ffi::Struct_PPB_VarArray_1_0 {
    fn create(&self) -> Struct_PP_Var {
        impl_fun!(self.Create => ())
    }
    fn get(&self, array: &Struct_PP_Var, index: libc::uint32_t) -> Struct_PP_Var {
        impl_fun!(self.Get => (*array, index))
    }
    fn set(&self, array: &Struct_PP_Var, index: libc::uint32_t, value: &Struct_PP_Var) -> bool {
        impl_fun!(self.Set => (*array, index, *value)) != 0
    }
    fn get_len(&self, array: &Struct_PP_Var) -> libc::uint32_t {
        impl_fun!(self.GetLength => (*array))
    }
    fn set_len(&self, array: &Struct_PP_Var, new_len: libc::uint32_t) -> bool {
        impl_fun!(self.SetLength => (*array, new_len)) != 0
    }
}
pub trait VarArrayBufferIf {
    fn create(&self, len: usize) -> Struct_PP_Var;
    fn byte_len(&self, var: &Struct_PP_Var) -> Option<usize>;
    fn map(&self, var: &Struct_PP_Var) -> *mut libc::c_void;
    fn unmap(&self, var: &Struct_PP_Var);
}
impl VarArrayBufferIf for ffi::Struct_PPB_VarArrayBuffer_1_0 {
    fn create(&self, len: usize) -> Struct_PP_Var {
        impl_fun!(self.Create => (len as u32))
    }
    fn byte_len(&self, var: &Struct_PP_Var) -> Option<usize> {
        let mut len: libc::uint32_t = unsafe { mem::uninitialized() };
        if impl_fun!(self.ByteLength => (*var, &mut len as *mut libc::uint32_t)) != 0 {
            Some(len as usize)
        } else {
            None
        }
    }
    fn map(&self, var: &Struct_PP_Var) -> *mut libc::c_void {
        impl_fun!(self.Map => (*var))
    }
    fn unmap(&self, var: &Struct_PP_Var) {
        impl_fun!(self.Unmap => (*var))
    }
}
pub trait VarDictionaryIf {
    fn create(&self) -> Struct_PP_Var;
    fn get(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var) -> Struct_PP_Var;
    fn set(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var, value: &Struct_PP_Var) -> bool;
    fn has_key(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var) -> bool;
    fn delete(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var);
    fn get_keys(&self, dict: &Struct_PP_Var) -> Struct_PP_Var;
}
impl VarDictionaryIf for ffi::Struct_PPB_VarDictionary_1_0 {
    fn create(&self) -> Struct_PP_Var {
        impl_fun!(self.Create => ())
    }
    fn get(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var) -> Struct_PP_Var {
        impl_fun!(self.Get => (*dict, *key))
    }
    fn set(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var, value: &Struct_PP_Var) -> bool {
        impl_fun!(self.Set => (*dict, *key, *value)) != 0
    }
    fn has_key(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var) -> bool {
        impl_fun!(self.HasKey => (*dict, *key)) != 0
    }
    fn delete(&self, dict: &Struct_PP_Var, key: &Struct_PP_Var) {
        impl_fun!(self.Delete => (*dict, *key))
    }
    fn get_keys(&self, dict: &Struct_PP_Var) -> Struct_PP_Var {
        impl_fun!(self.GetKeys => (*dict))
    }
}

pub trait MessagingIf {
    fn post_message(&self, instance: PP_Instance, msg: Struct_PP_Var);
}
impl MessagingIf for ffi::Struct_PPB_Messaging_1_2 {
    fn post_message(&self, instance: PP_Instance, msg: Struct_PP_Var) {
        impl_fun!(self.PostMessage => (instance, msg))
    }
}
pub trait MessageLoopIf {
    fn create(&self, instance: &PP_Instance) -> PP_Resource;
    fn get_for_main_thread(&self) -> PP_Resource;
    fn get_current(&self) -> Option<PP_Resource>;
    fn attach_to_current_thread(&self, msg_loop: &PP_Resource) -> libc::int32_t;
    fn run(&self, msg_loop: &PP_Resource) -> libc::int32_t;
    fn post_work(&self, to_loop: &PP_Resource,
                 callback: Struct_PP_CompletionCallback,
                 delay_ms: libc::int64_t) -> libc::int32_t;
    fn post_quit(&self, msg_loop: &PP_Resource, full: bool) -> libc::int32_t;
}
impl MessageLoopIf for ffi::Struct_PPB_MessageLoop_1_0 {
    fn create(&self, instance: &PP_Instance) -> PP_Resource {
        impl_fun!(self.Create => (*instance))
    }
    fn get_for_main_thread(&self) -> PP_Resource {
        impl_fun!(self.GetForMainThread)
    }
    fn get_current(&self) -> Option<PP_Resource> {
        let current = impl_fun!(self.GetCurrent);
        if current == unsafe { mem::transmute(0i32) } {
            None
        } else {
            Some(current)
        }
    }
    fn attach_to_current_thread(&self, msg_loop: &PP_Resource) -> libc::int32_t {
        impl_fun!(self.AttachToCurrentThread => (*msg_loop))
    }
    fn run(&self, msg_loop: &PP_Resource) -> libc::int32_t {
        impl_fun!(self.Run => (*msg_loop))
    }
    fn post_work(&self, to_loop: &PP_Resource,
                 callback: Struct_PP_CompletionCallback,
                 delay_ms: libc::int64_t) -> libc::int32_t {
        impl_fun!(self.PostWork => (*to_loop, callback, delay_ms))
    }
    fn post_quit(&self, msg_loop: &PP_Resource, full: bool) -> libc::int32_t {
        impl_fun!(self.PostQuit => (*msg_loop, full.to_ffi_bool()))
    }
}
pub trait ImageDataIf {
    fn native_image_data_format(&self) -> ffi::PP_ImageDataFormat;
    fn is_image_data_format_supported(&self, format: ffi::PP_ImageDataFormat) -> bool;
    fn create(&self,
              instance: PP_Instance,
              format: ffi::PP_ImageDataFormat,
              size: ffi::PP_Size,
              init_to_zero: bool) -> Option<PP_Resource>;
    fn describe(&self, img: PP_Resource) -> Option<ffi::Struct_PP_ImageDataDesc>;
    fn map(&self, img: &PP_Resource) -> *mut libc::c_void;
    fn unmap(&self, img: &PP_Resource);
}
impl ImageDataIf for ffi::Struct_PPB_ImageData_1_0 {
    fn native_image_data_format(&self) -> ffi::PP_ImageDataFormat {
        impl_fun!(self.GetNativeImageDataFormat)
    }
    fn is_image_data_format_supported(&self, format: ffi::PP_ImageDataFormat) -> bool {
        if impl_fun!(self.IsImageDataFormatSupported => (format)) != 0 {
            true
        } else {
            false
        }
    }
    fn create(&self,
              instance: PP_Instance,
              format: ffi::PP_ImageDataFormat,
              size: ffi::PP_Size,
              init_to_zero: bool) -> Option<PP_Resource> {
        let res = impl_fun!(self.Create => (instance,
                                            format,
                                            &size as *const ffi::PP_Size,
                                            init_to_zero.to_ffi_bool()));
        if res != 0 {
            Some(res)
        } else {
            None
        }
    }
    fn describe(&self, img: PP_Resource) -> Option<ffi::Struct_PP_ImageDataDesc> {
        let mut desc = unsafe { mem::uninitialized() };
        if impl_fun!(self.Describe => (img,
                                       &mut desc as *mut ffi::Struct_PP_ImageDataDesc)) != 0 {
            Some(desc)
        } else {
            None
        }
    }
    fn map(&self, img: &PP_Resource) -> *mut libc::c_void {
        impl_fun!(self.Map => (*img))
    }
    fn unmap(&self, img: &PP_Resource) {
        impl_fun!(self.Unmap => (*img))
    }
}

resource_interface!(impl for ffi::Struct_PPB_URLLoader_1_0 => IsURLLoader);
resource_interface_opt!(impl for ffi::Struct_PPB_URLLoader_1_0 => IsURLLoader);

pub trait URLLoaderIf {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource>;
    fn open(&self,
            loader: PP_Resource,
            request: PP_Resource,
            callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn get_response_info(&self, loader: PP_Resource) -> Option<PP_Resource>;
    fn read_response_body(&self, loader: PP_Resource, buffer: *mut libc::c_char, bytes: usize,
                          callback: ffi::Struct_PP_CompletionCallback) -> Code<usize>;
}
impl URLLoaderIf for ffi::Struct_PPB_URLLoader_1_0 {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        impl_fun!(self.Create => (instance) -> Option<PP_Resource>)
    }
    fn open(&self,
            loader: PP_Resource,
            request: PP_Resource,
            callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Open => (loader, request, callback) -> Code)
    }
    fn get_response_info(&self, loader: PP_Resource) -> Option<PP_Resource> {
        impl_fun!(self.GetResponseInfo => (loader) -> Option<PP_Resource>)
    }
    fn read_response_body(&self, loader: PP_Resource, buffer: *mut libc::c_char, bytes: usize,
                          callback: ffi::Struct_PP_CompletionCallback) -> Code<usize> {
        impl_fun!(self.ReadResponseBody => (loader, buffer as *mut _,
                                            bytes as libc::int32_t, callback) -> Code)
    }
}
pub trait URLRequestInfoIf {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource>;
    fn property(&self,
                res: PP_Resource,
                prop: ffi::PP_URLRequestProperty,
                value: PP_Var) -> bool;
    fn append_file_to_body(&self,
                           res: PP_Resource,
                           file: PP_Resource,
                           start_offset: Option<i64>,
                           bytes: Option<i64>,
                           last_modified: Option<ffi::PP_Time>) -> bool;
    fn append_blob_to_body(&self,
                           res: PP_Resource,
                           data: &Vec<u8>) -> bool;
}
resource_interface!(impl for ffi::Struct_PPB_URLRequestInfo_1_0 => IsURLRequestInfo);
resource_interface_opt!(impl for ffi::Struct_PPB_URLRequestInfo_1_0 => IsURLRequestInfo);
impl URLRequestInfoIf for ffi::Struct_PPB_URLRequestInfo_1_0 {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        let loader = impl_fun!(self.Create => (instance));
        if loader == 0 {
            None
        } else {
            Some(loader)
        }
    }
    fn property(&self,
                res: PP_Resource,
                prop: ffi::PP_URLRequestProperty,
                value: PP_Var) -> bool {
        let was_set = impl_fun!(self.SetProperty => (res, prop, value));
        was_set != 0
    }
    fn append_file_to_body(&self,
                           res: PP_Resource,
                           file: PP_Resource,
                           start_offset: Option<i64>,
                           bytes: Option<i64>,
                           last_modified: Option<ffi::PP_Time>) -> bool {
        let was_appended = impl_fun!
            (self.AppendFileToBody => (res,
                                       file,
                                       start_offset.unwrap_or(0),
                                       bytes.unwrap_or(-1),
                                       last_modified.unwrap_or(0.0f64)));
        was_appended != 0
    }
    fn append_blob_to_body(&self,
                           res: PP_Resource,
                           data: &Vec<u8>) -> bool {
        let was_appended = impl_fun!
            (self.AppendDataToBody => (res,
                                       data.as_ptr() as *const libc::c_void,
                                       data.len() as u32));
        was_appended != 0
    }

}
pub trait URLResponseInfoIf {
    fn property(&self,
                res: PP_Resource,
                property: ffi::PP_URLResponseProperty) -> ffi::PP_Var;
    fn body_as_file(&self, res: PP_Resource) -> PP_Resource;
}
resource_interface!(impl for ffi::Struct_PPB_URLResponseInfo_1_0 => IsURLResponseInfo);
resource_interface_opt!(impl for ffi::Struct_PPB_URLResponseInfo_1_0 => IsURLResponseInfo);
impl URLResponseInfoIf for ffi::Struct_PPB_URLResponseInfo_1_0 {
    fn property(&self,
                res: PP_Resource,
                property: ffi::PP_URLResponseProperty) -> ffi::PP_Var {
        impl_fun!(self.GetProperty => (res, property))
    }
    fn body_as_file(&self, res: PP_Resource) -> PP_Resource {
        impl_fun!(self.GetBodyAsFileRef => (res))
    }

}
pub trait InputEventIf {
    fn request(&self,
               instance: &PP_Instance,
               classes: u32) -> Code;
    fn request_filtering(&self,
                         instance: &PP_Instance,
                         classes: u32) -> Code;
    fn cancel_requests(&self,
                       instance: &PP_Instance,
                       classes: u32);
    fn type_of(&self, res: &PP_Resource) -> ffi::PP_InputEvent_Type;
    fn timestamp(&self, res: &PP_Resource) -> ffi::PP_TimeTicks;
    fn modifiers(&self, res: &PP_Resource) -> u32;
}
resource_interface!(impl for ffi::Struct_PPB_InputEvent_1_0 => IsInputEvent);
resource_interface_opt!(impl for ffi::Struct_PPB_InputEvent_1_0 => IsInputEvent);
impl InputEventIf for ffi::Struct_PPB_InputEvent_1_0 {
    fn request(&self,
               instance: &PP_Instance,
               classes: u32) -> Code {
        Code::from_i32(impl_fun!(self.RequestInputEvents => (*instance, classes)))
    }
    fn request_filtering(&self,
                         instance: &PP_Instance,
                         classes: u32) -> Code {
        Code::from_i32(impl_fun!(self.RequestFilteringInputEvents => (*instance, classes)))
    }
    fn cancel_requests(&self,
                       instance: &PP_Instance,
                       classes: u32) {
        impl_fun!(self.ClearInputEventRequest => (*instance, classes))
    }
    fn type_of(&self, res: &PP_Resource) -> ffi::PP_InputEvent_Type {
        impl_fun!(self.GetType => (*res))
    }
    fn timestamp(&self, res: &PP_Resource) -> ffi::PP_TimeTicks {
        impl_fun!(self.GetTimeStamp => (*res))
    }
    fn modifiers(&self, res: &PP_Resource) -> u32 {
        impl_fun!(self.GetModifiers => (*res))
    }
}
pub trait MouseInputEventIf {
    fn button(&self, res: &PP_Resource) -> ffi::PP_InputEvent_MouseButton;
    fn point(&self, res: &PP_Resource) -> ffi::PP_FloatPoint;
    fn click_count(&self, res: &PP_Resource) -> i32;
    fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint;
}
resource_interface!(impl for ffi::Struct_PPB_MouseInputEvent_1_1 => IsMouseInputEvent);
resource_interface_opt!(impl for ffi::Struct_PPB_MouseInputEvent_1_1 => IsMouseInputEvent);
impl MouseInputEventIf for ffi::Struct_PPB_MouseInputEvent_1_1 {
    fn button(&self, res: &PP_Resource) -> ffi::PP_InputEvent_MouseButton {
        impl_fun!(self.GetButton => (*res))
    }
    fn point(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        let p = impl_fun!(self.GetPosition => (*res));
        ffi::Struct_PP_FloatPoint {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
    fn click_count(&self, res: &PP_Resource) -> i32 {
        impl_fun!(self.GetClickCount => (*res))
    }
    fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        let p = impl_fun!(self.GetMovement => (*res));
        ffi::Struct_PP_FloatPoint {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}

pub trait KeyboardInputEventIf {
    fn key_code(&self, res: &PP_Resource) -> u32;
    fn text(&self, res: &PP_Resource) -> ffi::PP_Var;
}
resource_interface!(impl for ffi::Struct_PPB_KeyboardInputEvent_1_2 => IsKeyboardInputEvent);
resource_interface_opt!(impl for ffi::Struct_PPB_KeyboardInputEvent_1_2 => IsKeyboardInputEvent);
impl KeyboardInputEventIf for ffi::Struct_PPB_KeyboardInputEvent_1_2 {
    fn key_code(&self, res: &PP_Resource) -> u32 {
        impl_fun!(self.GetKeyCode => (*res))
    }
    fn text(&self, res: &PP_Resource) -> ffi::PP_Var {
        impl_fun!(self.GetCharacterText => (*res))
    }
}
pub trait TouchInputEventIf {
    fn count(&self, res: &PP_Resource, list: ffi::PP_TouchListType) -> u32;
    fn by_index(&self,
                res: &PP_Resource,
                list_type: ffi::PP_TouchListType,
                index: u32) -> ffi::PP_TouchPoint;
    fn by_id(&self,
             res: &PP_Resource,
             list_type: ffi::PP_TouchListType,
             id: u32) -> ffi::PP_TouchPoint;
}
resource_interface!(impl for ffi::Struct_PPB_TouchInputEvent_1_0 => IsTouchInputEvent);
resource_interface_opt!(impl for ffi::Struct_PPB_TouchInputEvent_1_0 => IsTouchInputEvent);
impl TouchInputEventIf for ffi::Struct_PPB_TouchInputEvent_1_0 {
    fn count(&self, res: &PP_Resource, list: ffi::PP_TouchListType) -> u32 {
        impl_fun!(self.GetTouchCount => (*res, list))
    }
    fn by_index(&self,
                res: &PP_Resource,
                list_type: ffi::PP_TouchListType,
                index: u32) -> ffi::PP_TouchPoint {
        impl_fun!(self.GetTouchByIndex => (*res, list_type, index))
    }
    fn by_id(&self,
             res: &PP_Resource,
             list_type: ffi::PP_TouchListType,
             id: u32) -> ffi::PP_TouchPoint {
        impl_fun!(self.GetTouchById => (*res, list_type, id))
    }
}
pub trait WheelInputEventIf {
    fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint;
    fn ticks(&self, res: &PP_Resource) -> ffi::PP_FloatPoint;
    fn scroll_by_page(&self, res: &PP_Resource) -> bool;
}
resource_interface!(impl for ffi::Struct_PPB_WheelInputEvent_1_0 => IsWheelInputEvent);
resource_interface_opt!(impl for ffi::Struct_PPB_WheelInputEvent_1_0 => IsWheelInputEvent);
impl WheelInputEventIf for ffi::Struct_PPB_WheelInputEvent_1_0 {
    fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        impl_fun!(self.GetDelta => (*res))
    }
    fn ticks(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        impl_fun!(self.GetTicks => (*res))
    }
    fn scroll_by_page(&self, res: &PP_Resource) -> bool {
        let r = impl_fun!(self.GetScrollByPage => (*res));
        r != 0
    }
}
pub trait Graphics3DIf {
    fn attrib_max_value(&self, instance: PP_Instance, attribute: i32) -> Result<i32>;
    fn attribs(&self,
               ctxt: PP_Resource,
               attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Result<Vec<u32>>;
    fn status(&self, ctxt: PP_Resource) -> Code;
    fn resize_buffers(&self, ctxt: PP_Resource, width: i32, height: i32) -> Code;
    fn set_attribs(&self,
                   ctxt: PP_Resource,
                   mut attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Code;
    fn swap_buffers(&self,
                    ctxt: PP_Resource,
                    callback: ffi::Struct_PP_CompletionCallback) -> Code;
}
resource_interface!(impl for ffi::Struct_PPB_Graphics3D_1_0 => IsGraphics3D);
resource_interface_opt!(impl for ffi::Struct_PPB_Graphics3D_1_0 => IsGraphics3D);
impl Graphics3DIf for ffi::Struct_PPB_Graphics3D_1_0 {
    fn attrib_max_value(&self, instance: PP_Instance, attribute: i32) -> Result<i32> {
        let mut value: i32 = 0;
        let r = impl_fun!(self.GetAttribMaxValue => (instance,
                                                     attribute,
                                                     &mut value as *mut i32));
        Code::from_i32(r).to_result(|_| value)
    }
    fn attribs(&self,
               ctxt: PP_Resource,
               attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Result<Vec<u32>> {
        use std::ops::Rem;

        let mut attribs: Vec<ffi::PP_Graphics3DAttrib> = attribs
            .into_iter()
            .flat_map(|attrib| {
                let v = vec!(attrib, 0);
                v.into_iter()
            })
            .collect();
        attribs.push(ffi::PP_GRAPHICS3DATTRIB_NONE);
        let r = impl_fun!(self.GetAttribs => (ctxt, attribs.as_mut_ptr() as *mut i32));
        let r = Code::from_i32(r);
        if r.is_ok() {
            Ok(attribs.into_iter()
               .enumerate()
               .fold(vec!(), |mut fold: Vec<u32>, (i, v): (usize, u32)| {
                   if i.rem(2) != 0 {
                       fold.push(v);
                       fold
                   } else {
                       fold
                   }
               }))
        } else {
            Err(r)
        }
    }
    fn status(&self, ctxt: PP_Resource) -> Code {
        Code::from_i32(impl_fun!(self.GetError => (ctxt)))
    }
    fn resize_buffers(&self, ctxt: PP_Resource, width: i32, height: i32) -> Code {
        Code::from_i32(impl_fun!(self.ResizeBuffers => (ctxt, width, height)))
    }
    fn set_attribs(&self,
                   ctxt: PP_Resource,
                   mut attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Code {
        attribs.push(ffi::PP_GRAPHICS3DATTRIB_NONE);
        Code::from_i32(impl_fun!(self.SetAttribs => (ctxt, attribs.as_ptr() as *const i32)))
    }
    fn swap_buffers(&self,
                    ctxt: PP_Resource,
                    callback: ffi::Struct_PP_CompletionCallback) -> Code {
        Code::from_i32(impl_fun!(self.SwapBuffers => (ctxt, callback)))
    }
}
pub trait ViewIf {
    fn rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect>;
    fn is_fullscreen(&self, res: PP_Resource) -> bool;
    fn is_visible(&self, res: PP_Resource) -> bool;
    fn is_page_visible(&self, res: PP_Resource) -> bool;
    fn clip_rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect>;
    fn device_scale(&self, res: PP_Resource) -> f32;
    fn css_scale(&self, res: PP_Resource) -> f32;
}
resource_interface!(impl for ffi::Struct_PPB_View_1_2 => IsView);
resource_interface_opt!(impl for ffi::Struct_PPB_View_1_2 => IsView);
impl ViewIf for ffi::Struct_PPB_View_1_2 {
    fn rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect> {
        let mut dest = unsafe { uninitialized() };
        let ok = impl_fun!(self.GetRect => (res, &mut dest as *mut ffi::Struct_PP_Rect));
        if ok != 0 {
            Some(dest)
        } else {
            None
        }
    }
    fn is_fullscreen(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsFullscreen => (res))) != 0
    }
    fn is_visible(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsVisible => (res))) != 0
    }
    fn is_page_visible(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsPageVisible => (res))) != 0
    }
    fn clip_rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect> {
        let mut dest = unsafe { uninitialized() };
        let ok = impl_fun!(self.GetClipRect => (res, &mut dest as *mut ffi::Struct_PP_Rect));
        if ok != 0 {
            Some(dest)
        } else {
            None
        }
    }
    fn device_scale(&self, res: PP_Resource) -> f32 {
        impl_fun!(self.GetDeviceScale => (res))
    }
    fn css_scale(&self, res: PP_Resource) -> f32 {
        impl_fun!(self.GetCSSScale => (res))
    }
}
pub trait FileSystemIf {
    fn create(&self, inst: PP_Instance, t: ffi::PP_FileSystemType) -> Option<ffi::PP_Resource>;
    fn open(&self, sys: PP_Resource, expected_size: libc::int64_t,
            callback: ffi::PP_CompletionCallback) -> Code;
    fn get_type(&self, res: PP_Resource) -> ffi::PP_FileSystemType;
}
resource_interface!(impl for ffi::Struct_PPB_FileSystem_1_0 => IsFileSystem);
resource_interface_opt!(impl for ffi::Struct_PPB_FileSystem_1_0 => IsFileSystem);
impl FileSystemIf for ffi::Struct_PPB_FileSystem_1_0 {
    fn create(&self, inst: PP_Instance, t: ffi::PP_FileSystemType) -> Option<ffi::PP_Resource> {
        let res = impl_fun!(self.Create => (inst, t));
        if res == 0 { None }
        else        { Some(res) }
    }
    fn open(&self, sys: PP_Resource, expected_size: libc::int64_t,
            callback: ffi::PP_CompletionCallback) -> Code {
        impl_fun!(self.Open => (sys, expected_size, callback) -> Code)
    }
    fn get_type(&self, res: PP_Resource) -> ffi::PP_FileSystemType {
        impl_fun!(self.GetType => (res))
    }
}
pub trait FileRefIf {
    fn create(&self, fs: PP_Resource, path: *const libc::c_char) -> Option<PP_Resource>;
    fn get_file_system_type(&self, f: PP_Resource) -> ffi::PP_FileSystemType;
    fn get_name(&self, f: PP_Resource) -> PP_Var;
    fn get_path(&self, f: PP_Resource) -> PP_Var;
    fn get_parent(&self, f: PP_Resource) -> Option<PP_Resource>;
    fn mkdir(&self, f: PP_Resource, flags: libc::int32_t,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn touch(&self, f: PP_Resource, atime: ffi::PP_Time, mtime: ffi::PP_Time,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn delete(&self, f: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn rename(&self, f: PP_Resource, new_f: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn query(&self, f: PP_Resource, info: &mut ffi::Struct_PP_FileInfo,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;

    fn read_directory_entries(&self, f: PP_Resource, output: ffi::Struct_PP_ArrayOutput,
                              callback: ffi::Struct_PP_CompletionCallback) -> Code;
}
resource_interface!(impl for ffi::Struct_PPB_FileRef_1_2 => IsFileRef);
resource_interface_opt!(impl for ffi::Struct_PPB_FileRef_1_2 => IsFileRef);
impl FileRefIf for ffi::Struct_PPB_FileRef_1_2 {
    fn create(&self, fs: PP_Resource, path: *const libc::c_char) -> Option<PP_Resource> {
        let res = impl_fun!(self.Create => (fs, path));
        if res == 0 { None }
        else        { Some(res) }
    }
    fn get_file_system_type(&self, f: PP_Resource) -> ffi::PP_FileSystemType {
        impl_fun!(self.GetFileSystemType => (f))
    }
    fn get_name(&self, f: PP_Resource) -> PP_Var { impl_fun!(self.GetName => (f)) }
    fn get_path(&self, f: PP_Resource) -> PP_Var { impl_fun!(self.GetPath => (f)) }
    fn get_parent(&self, f: PP_Resource) -> Option<PP_Resource> {
        impl_fun!(self.GetParent => (f) -> Option<PP_Resource>)
    }
    fn mkdir(&self, f: PP_Resource, flags: libc::int32_t,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.MakeDirectory => (f, flags, callback) -> Code)
    }
    fn touch(&self, f: PP_Resource, atime: ffi::PP_Time, mtime: ffi::PP_Time,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Touch => (f, atime, mtime, callback) -> Code)
    }
    fn delete(&self, f: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Delete => (f, callback) -> Code)
    }
    fn rename(&self, f: PP_Resource, new_f: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Rename => (f, new_f, callback) -> Code)
    }
    fn query(&self, f: PP_Resource, info: &mut ffi::Struct_PP_FileInfo,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Query => (f, info as *mut _, callback) -> Code)
    }
    fn read_directory_entries(&self, f: PP_Resource, output: ffi::Struct_PP_ArrayOutput,
                              callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.ReadDirectoryEntries => (f, output, callback) -> Code)
    }
}

pub trait FileIoIf {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource>;
    fn open(&self, f: PP_Resource, r: PP_Resource, flags: libc::int32_t,
            callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn query(&self, f: PP_Resource, info: &mut ffi::Struct_PP_FileInfo,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn touch(&self, f: PP_Resource, atime: ffi::PP_Time, mtime: ffi::PP_Time,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn read(&self, f: PP_Resource, offset: libc::uint64_t, buffer: *mut libc::c_char,
            bytes: usize, callback: ffi::Struct_PP_CompletionCallback) -> Code<usize>;
    fn write(&self, f: PP_Resource, offset: libc::uint64_t, buffer: *const libc::c_char,
             bytes: usize, callback: ffi::Struct_PP_CompletionCallback) -> Code<usize>;
    fn set_length(&self, f: PP_Resource, length: libc::uint64_t,
                  callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn flush(&self, f: PP_Resource,
             callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn close(&self, f: PP_Resource);
    fn read_to_array(&self, f: PP_Resource, offset: libc::uint64_t, max_read: libc::size_t,
                     output: &mut ffi::Struct_PP_ArrayOutput,
                     callback: ffi::Struct_PP_CompletionCallback) -> Code;
}
resource_interface!(impl for ffi::Struct_PPB_FileIO_1_1 => IsFileIO);
resource_interface_opt!(impl for ffi::Struct_PPB_FileIO_1_1 => IsFileIO);
impl FileIoIf for ffi::Struct_PPB_FileIO_1_1 {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        impl_fun!(self.Create => (instance) -> Option<PP_Resource>)
    }
    fn open(&self, f: PP_Resource, r: PP_Resource, flags: libc::int32_t,
            callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Open => (f, r, flags, callback) -> Code)
    }
    fn query(&self, f: PP_Resource, info: &mut ffi::Struct_PP_FileInfo,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Query => (f, info as *mut _, callback) -> Code)
    }
    fn touch(&self, f: PP_Resource, atime: ffi::PP_Time, mtime: ffi::PP_Time,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Touch => (f, atime, mtime, callback) -> Code)
    }
    fn read(&self, f: PP_Resource, offset: libc::uint64_t, buffer: *mut libc::c_char,
            bytes: usize, callback: ffi::Struct_PP_CompletionCallback) -> Code<usize> {
        impl_fun!(self.Read => (f, offset as libc::int64_t, buffer,
                                bytes as libc::int32_t, callback) -> Code)
    }
    fn write(&self, f: PP_Resource, offset: libc::uint64_t, buffer: *const libc::c_char,
             bytes: usize, callback: ffi::Struct_PP_CompletionCallback) -> Code<usize> {
        impl_fun!(self.Write => (f, offset as libc::int64_t, buffer,
                                 bytes as libc::int32_t, callback) -> Code)
    }
    fn set_length(&self, f: PP_Resource, length: libc::uint64_t,
                  callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.SetLength => (f, length as libc::int64_t, callback) -> Code)
    }
    fn flush(&self, f: PP_Resource,
             callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Flush => (f, callback) -> Code)
    }
    fn close(&self, f: PP_Resource) { impl_fun!(self.Close => (f)) }
    fn read_to_array(&self, f: PP_Resource, offset: libc::uint64_t, max_read: libc::size_t,
                     output: &mut ffi::Struct_PP_ArrayOutput,
                     callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.ReadToArray => (f, offset as libc::int64_t, max_read as libc::int32_t,
                                       output as *mut _, callback) -> Code)
    }
}

pub trait MediaStreamVideoTrackIf {
    //fn create(&self, inst: PP_Instance) -> Option<PP_Resource>;
    fn configure(&self, res: PP_Resource, attrs: &[ffi::PP_MediaStreamVideoTrack_Attrib],
                 callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn get_attrib(&self, res: PP_Resource, attr: ffi::PP_MediaStreamVideoTrack_Attrib) ->
        Code<libc::int32_t>;
    fn get_id(&self, res: PP_Resource) -> ffi::PP_Var;
    fn has_ended(&self, res: PP_Resource) -> bool;
    fn get_frame(&self, res: PP_Resource, frame: &mut PP_Resource,
                 callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn recycle_frame(&self, res: PP_Resource, frame: PP_Resource) -> Code;
    fn close(&self, res: PP_Resource);
    /*fn get_empty_frame(&self, res: PP_Resource, frame: &mut PP_Resource,
                       callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn put_frame(&self, res: PP_Resource, frame: PP_Resource) -> Code;*/
}
resource_interface!(impl for ffi::Struct_PPB_MediaStreamVideoTrack_0_1 => IsMediaStreamVideoTrack);
resource_interface_opt!(impl for ffi::Struct_PPB_MediaStreamVideoTrack_0_1 => IsMediaStreamVideoTrack);
impl MediaStreamVideoTrackIf for ffi::Struct_PPB_MediaStreamVideoTrack_0_1 {
    /*fn create(&self, inst: PP_Instance) -> Option<PP_Resource> {
        let res = impl_fun!(self.Create => (inst));
        if res == 0 { None }
        else        { Some(res) }
    }*/
    fn configure(&self, res: PP_Resource, attrs: &[ffi::PP_MediaStreamVideoTrack_Attrib],
                 callback: ffi::Struct_PP_CompletionCallback) -> Code {
        debug_assert!(attrs.last() == Some(&ffi::PP_MEDIASTREAMVIDEOTRACK_ATTRIB_NONE));
        let code = impl_fun!(self.Configure => (res, attrs.as_ptr() as *const i32, callback));
        From::from(code)
    }
    fn get_attrib(&self, res: PP_Resource, attr: ffi::PP_MediaStreamVideoTrack_Attrib) ->
        Code<libc::int32_t>
    {
        let mut dest: libc::int32_t = unsafe { ::std::mem::uninitialized() };
        let code = impl_fun!(self.GetAttrib => (res, attr, &mut dest as *mut _));
        let code: Code = From::from(code);
        code.map_ok(|_| dest)
    }
    fn get_id(&self, res: PP_Resource) -> ffi::PP_Var {
        impl_fun!(self.GetId => (res))
    }
    fn has_ended(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.HasEnded => (res))) != 0
    }
    fn get_frame(&self, res: PP_Resource, frame: &mut PP_Resource,
                 callback: ffi::Struct_PP_CompletionCallback) -> Code {
        let code = impl_fun!(self.GetFrame => (res, frame as *mut _, callback));
        From::from(code)
    }
    fn recycle_frame(&self, res: PP_Resource, frame: PP_Resource) -> Code {
        let code = impl_fun!(self.RecycleFrame => (res, frame));
        From::from(code)
    }
    fn close(&self, res: PP_Resource) {
        impl_fun!(self.Close => (res));
    }
    /*fn get_empty_frame(&self, res: PP_Resource, frame: &mut PP_Resource,
                       callback: ffi::Struct_PP_CompletionCallback) -> Code {
        let code = impl_fun!(self.GetEmptyFrame => (res, frame as *mut _, callback));
        From::from(code)
    }
    fn put_frame(&self, res: PP_Resource, frame: PP_Resource) -> Code {
        let code = impl_fun!(self.PutFrame => (res, frame));
        From::from(code)
    }*/
}

pub trait VideoFrameIf {
    fn get_timestamp(&self, res: PP_Resource) -> ffi::PP_TimeDelta;
    fn set_timestamp(&self, res: PP_Resource, ts: ffi::PP_TimeDelta);
    fn get_format(&self, res: PP_Resource) -> ffi::PP_VideoFrame_Format;
    fn get_size(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Size>;
    fn get_data_buffer(&self, res: PP_Resource) -> *const u8;
    fn get_data_buffer_size(&self, res: PP_Resource) -> usize;
}
resource_interface!(impl for ffi::Struct_PPB_VideoFrame_0_1 => IsVideoFrame);
resource_interface_opt!(impl for ffi::Struct_PPB_VideoFrame_0_1 => IsVideoFrame);
impl VideoFrameIf for ffi::Struct_PPB_VideoFrame_0_1 {
    fn get_timestamp(&self, res: PP_Resource) -> ffi::PP_TimeDelta {
        impl_fun!(self.GetTimestamp => (res))
    }
    fn set_timestamp(&self, res: PP_Resource, ts: ffi::PP_TimeDelta) {
        impl_fun!(self.SetTimestamp => (res, ts))
    }
    fn get_format(&self, res: PP_Resource) -> ffi::PP_VideoFrame_Format {
        (impl_fun!(self.GetFormat => (res))) as ffi::PP_VideoFrame_Format
    }
    fn get_size(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Size> {
        let mut size: ffi::Struct_PP_Size = unsafe { ::std::mem::uninitialized() };
        let success = impl_fun!(self.GetSize => (res, &mut size as *mut _));
        if success != 0 {
            Some(size)
        } else {
            None
        }
    }
    fn get_data_buffer(&self, res: PP_Resource) -> *const u8 {
        (impl_fun!(self.GetDataBuffer => (res))) as *const _
    }
    fn get_data_buffer_size(&self, res: PP_Resource) -> usize {
        (impl_fun!(self.GetDataBufferSize => (res))) as usize
    }
}

pub trait VideoDecoderIf {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource>;
    fn initialize(&self, decoder: PP_Resource, g3d: PP_Resource, profile: ffi::PP_VideoProfile,
                  accel: ffi::PP_HardwareAcceleration,
                  callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn decode(&self, decoder: PP_Resource, decode_tag: u32, size: usize, data: *const u8,
              callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn get_picture(&self, decoder: PP_Resource, picture: *mut ffi::Struct_PP_VideoPicture,
                   callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn recycle_picture(&self, decoder: PP_Resource, picture: &ffi::Struct_PP_VideoPicture);
    fn flush(&self, decoder: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code;
    fn reset(&self, decoder: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code;
}
resource_interface!(impl for ffi::Struct_PPB_VideoDecoder_1_0 => IsVideoDecoder);
resource_interface_opt!(impl for ffi::Struct_PPB_VideoDecoder_1_0 => IsVideoDecoder);
impl VideoDecoderIf for ffi::Struct_PPB_VideoDecoder_1_0 {
    fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        impl_fun!(self.Create => (instance) -> Option<PP_Resource>)
    }
    fn initialize(&self, decoder: PP_Resource, g3d: PP_Resource, profile: ffi::PP_VideoProfile,
                  accel: ffi::PP_HardwareAcceleration,
                  callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Initialize => (decoder, g3d, profile, accel, callback) -> Code)
    }
    fn decode(&self, decoder: PP_Resource, decode_tag: u32, size: usize, data: *const u8,
              callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Decode => (decoder, decode_tag, size as libc::uint32_t,
                                  data as *const libc::c_void, callback) -> Code)
    }
    fn get_picture(&self, decoder: PP_Resource, picture: *mut ffi::Struct_PP_VideoPicture,
                   callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.GetPicture => (decoder, picture, callback) -> Code)
    }
    fn recycle_picture(&self, decoder: PP_Resource, picture: &ffi::Struct_PP_VideoPicture) {
        impl_fun!(self.RecyclePicture => (decoder, picture as *const _))
    }
    fn flush(&self, decoder: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Flush => (decoder, callback) -> Code)
    }
    fn reset(&self, decoder: PP_Resource, callback: ffi::Struct_PP_CompletionCallback) -> Code {
        impl_fun!(self.Reset => (decoder, callback) -> Code)
    }
}

pub trait ConsoleInterface {
    fn log<T: ToVar>(&self, lvl: ffi::PP_LogLevel, value: T) {
        self.log_to_browser(lvl, value.to_var());
    }
    fn log_to_browser(&self, lvl: ffi::PP_LogLevel, value: ffi::PP_Var);
    fn log_with_source<ST: ToVar, VT: ToVar>(&self, lvl: ffi::PP_LogLevel, source: ST, value: VT);
}
impl ConsoleInterface for super::Console {
    fn log_to_browser(&self, lvl: ffi::PP_LogLevel, value: ffi::PP_Var) {
        (get_console().Log.unwrap())(self.unwrap(), lvl, value)
    }
    fn log_with_source<ST: ToVar, VT: ToVar>(&self,
                                             lvl: ffi::PP_LogLevel,
                                             source: ST,
                                             value: VT) {
        (get_console().LogWithSource.unwrap())(self.unwrap(),
                                               lvl,
                                               source.to_var(),
                                               value.to_var())
    }
}

pub trait TextInputController {
    fn cancel_composition_text(&self);
    fn set_text_input_type(&self, input_type: ffi::PP_TextInput_Type);
    fn update_caret_position(&self, caret: ffi::PP_Rect);
    fn update_surrounding_text<T: ToVar>(&self, text: T, caret: u32, anchor: u32);
}
