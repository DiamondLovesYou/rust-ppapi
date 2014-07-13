//! PPB related interfaces. Many interfaces have convenience functions to remove much
//! of the verbose-ness of the originals.

#![allow(missing_doc)]
use core::mem;
use core::mem::uninitialized;
use libc;
use std::ptr::RawPtr;
use std::{intrinsics, str};

use super::ffi;
use super::ffi::{Struct_PP_Var, PP_Instance, PP_LogLevel, PP_Resource,
                 Struct_PP_CompletionCallback, PP_Var};
use super::{Ticks, Time, ToFFIBool, Code, Result, ToVar};

pub type Var = ffi::PPB_Var;
pub type Core = ffi::PPB_Core;
pub type Console = ffi::PPB_Console;
pub type VarArrayBuffer = ffi::PPB_VarArrayBuffer;
pub type Graphics3D = ffi::PPB_Graphics3D;
pub type Messaging = ffi::PPB_Messaging;
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
pub type View = ffi::PPB_View;

mod consts {
    pub static VAR: &'static str              = "PPB_Var;1.1";
    pub static CORE: &'static str             = "PPB_Core;1.0";
    pub static CONSOLE: &'static str          = "PPB_Console;1.0";
    pub static MESSAGING: &'static str        = "PPB_Messaging;1.0";
    pub static MESSAGELOOP: &'static str      = "PPB_MessageLoop;1.0";
    pub static VAR_ARRAY_BUFFER: &'static str = "PPB_VarArrayBuffer;1.0";
    pub static GRAPHICS_3D: &'static str      = "PPB_Graphics3D;1.0";
    pub static INSTANCE: &'static str         = "PPB_Instance;1.0";
    pub static INPUT:    &'static str         = "PPB_InputEvent;1.0";
    pub static KEYBOARD: &'static str         = "PPB_KeyboardInputEvent;1.2";
    pub static MOUSE:    &'static str         = "PPB_MouseInputEvent;1.1";
    pub static WHEEL:    &'static str         = "PPB_WheelInputEvent;1.0";
    pub static TOUCH:    &'static str         = "PPB_TouchInputEvent;1.0";
    pub static IME:      &'static str         = "PPB_IMEInputEvent;1.0";
    pub static GLES2:    &'static str         = "PPB_OpenGLES2;1.0";
    pub static FONTDEV:  &'static str         = "PPB_Font(Dev);0.6";
    pub static IMAGEDATA:&'static str         = "PPB_ImageData;1.0";
    pub static URL_LOADER: &'static str       = "PPB_URLLoader;1.0";
    pub static URL_REQUEST: &'static str      = "PPB_URLRequestInfo;1.0";
    pub static URL_RESPONSE: &'static str     = "PPB_URLResponseInfo;1.0";
    pub static VIEW:     &'static str         = "PPB_View;1.1";
}
mod globals {
    use super::super::ffi;
    pub static mut BROWSER:      Option<ffi::PPB_GetInterface> = None;
    pub static mut VAR:          Option<&'static super::Var> = None;
    pub static mut CORE:         Option<&'static super::Core> = None;
    pub static mut CONSOLE:      Option<&'static super::Console> = None;
    pub static mut ARRAY_BUFFER: Option<&'static super::VarArrayBuffer> = None;
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
}
#[cold] #[inline(never)] #[doc(hidden)]
pub fn initialize_globals(b: ffi::PPB_GetInterface) {
    unsafe {
        globals::BROWSER       = Some(b);
        globals::VAR           = get_interface(consts::VAR);
        globals::CONSOLE       = get_interface(consts::CONSOLE);
        globals::CORE          = get_interface(consts::CORE);
        globals::ARRAY_BUFFER  = get_interface(consts::VAR_ARRAY_BUFFER);
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
        let ptr = name.with_c_str_unchecked(|p| {
                get_actual_browser()(p) as *const T
            });

        if ptr.is_null() { None }
        else             { Some(mem::transmute(ptr)) }
    }
}
macro_rules! get_fun(
    (pub fn $ident:ident() -> $ty:ty { $global:ident }) => (
        #[doc = "Returns a static ref to the interface"]
        pub fn $ident() -> &'static $ty {
            #[inline(never)] fn failure() -> ! {
                fail!("Missing browser {} interface", stringify!($ty))
            }
            unsafe {
                globals::$global.unwrap_or_else(failure)
            }
        }
    );
)
macro_rules! get_fun_opt(
    (pub fn $ident:ident() -> $ty:ty { $global:ident }) => (
        #[doc = "Returns an optional static ref to the interface"]
        pub fn $ident() -> Option<&'static $ty> {
            unsafe {
                globals::$global
            }
        }
    );
)

get_fun!    (pub fn get_var() -> Var { VAR })
get_fun_opt!(pub fn get_var_opt() -> Var { VAR })
get_fun!    (pub fn get_core() -> Core { CORE })
get_fun_opt!(pub fn get_core_opt() -> Core { CORE })
get_fun!    (pub fn get_console() -> Console { CONSOLE })
get_fun_opt!(pub fn get_console_opt() -> Console { CONSOLE })
get_fun!    (pub fn get_array_buffer() -> VarArrayBuffer { ARRAY_BUFFER })
get_fun_opt!(pub fn get_array_buffer_opt() -> VarArrayBuffer { ARRAY_BUFFER })
get_fun!    (pub fn get_graphics_3d() -> Graphics3D { GRAPHICS_3D })
get_fun_opt!(pub fn get_graphics_3d_opt() -> Graphics3D { GRAPHICS_3D })
get_fun!    (pub fn get_message_loop() -> MessageLoop { MESSAGE_LOOP })
get_fun_opt!(pub fn get_message_loop_opt() -> MessageLoop { MESSAGE_LOOP })
get_fun!    (pub fn get_instance() -> Instance { INSTANCE })
get_fun_opt!(pub fn get_instance_opt() -> Instance { INSTANCE })
get_fun!    (pub fn get_input_event() -> InputEvent { INPUT })
get_fun_opt!(pub fn get_input_event_opt() -> InputEvent { INPUT })
get_fun!    (pub fn get_keyboard_event() -> KeyboardInputEvent { KEYBOARD })
get_fun_opt!(pub fn get_keyboard_event_opt() -> KeyboardInputEvent { KEYBOARD })
get_fun!    (pub fn get_mouse_event() -> MouseInputEvent { MOUSE })
get_fun_opt!(pub fn get_mouse_event_opt() -> MouseInputEvent { MOUSE })
get_fun!    (pub fn get_wheel_event() -> WheelInputEvent { WHEEL })
get_fun_opt!(pub fn get_wheel_event_opt() -> WheelInputEvent { WHEEL })
get_fun!    (pub fn get_touch_event() -> TouchInputEvent { TOUCH })
get_fun_opt!(pub fn get_touch_event_opt() -> TouchInputEvent { TOUCH })
get_fun!    (pub fn get_ime_event() -> IMEInputEvent { IME })
get_fun_opt!(pub fn get_ime_event_opt() -> IMEInputEvent { IME })
get_fun!    (pub fn get_gles2() -> OpenGLES2 { GLES2 })
get_fun_opt!(pub fn get_gles2_opt() -> OpenGLES2 { GLES2 })
get_fun!    (pub fn get_font() -> Font { FONTDEV })
get_fun_opt!(pub fn get_font_opt() -> Font { FONTDEV })
get_fun!    (pub fn get_image_data() -> ImageData { IMAGEDATA })
get_fun_opt!(pub fn get_image_data_opt() -> ImageData { IMAGEDATA })
get_fun!    (pub fn get_url_loader() -> UrlLoader { URL_LOADER })
get_fun_opt!(pub fn get_url_loader_opt() -> UrlLoader { URL_LOADER })
get_fun!    (pub fn get_url_request() -> UrlRequestInfo { URL_REQUEST })
get_fun_opt!(pub fn get_url_request_opt() -> UrlRequestInfo { URL_REQUEST })
get_fun!    (pub fn get_url_response() -> UrlResponseInfo { URL_RESPONSE })
get_fun_opt!(pub fn get_url_response_opt() -> UrlResponseInfo { URL_RESPONSE })
get_fun!    (pub fn get_view() -> View { VIEW })
get_fun_opt!(pub fn get_view_opt() -> View { VIEW })


macro_rules! impl_fun(
    ($fun:expr => ( $($arg:expr),* ) ) => ({
        #[inline(never)] fn failure() -> ! {
            fail!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = $fun.unwrap_or_else(failure);
        f( $($arg),* )
    });
    ($fun:expr) => ({
        #[inline(never)] fn failure() -> ! {
            fail!("Interface function \"{}\" missing!", stringify!($fun))
        }
        let f = $fun.unwrap_or_else(failure);
        f()
    })
)

impl ffi::Struct_PPB_Core_1_0 {
    pub fn get_time_ticks(&self) -> Ticks {
        impl_fun!(self.GetTimeTicks)
    }
    pub fn get_time(&self) -> Time {
        impl_fun!(self.GetTime)
    }
}
impl ffi::Struct_PPB_Var_1_1 {
    pub fn add_ref(&self, var: &Struct_PP_Var) {
        impl_fun!(self.AddRef => (*var))
    }
    pub fn remove_ref(&self, var: Struct_PP_Var) {
        impl_fun!(self.Release => (var))
    }

    pub fn var_from_utf8(&self, string: &str) -> Struct_PP_Var {
        impl_fun!(self.VarFromUtf8 => (string.as_ptr() as *const i8, string.len() as u32))
    }
    pub fn var_to_utf8(&self, string: &Struct_PP_Var) -> String {
        let mut len: u32 = unsafe { intrinsics::uninit() };
        let buf = impl_fun!(self.VarToUtf8 => (*string, &mut len as *mut u32));
        unsafe { str::raw::from_buf_len(buf as *const u8, len as uint) }
    }
}
impl ffi::Struct_PPB_Console_1_0 {
    pub fn log(&self, inst: PP_Instance, lvl: PP_LogLevel, msg: &Struct_PP_Var) {
        impl_fun!(self.Log => (inst, lvl, *msg))
    }
    pub fn log_with_source(&self,
                           inst: PP_Instance,
                           lvl: PP_LogLevel,
                           src: &Struct_PP_Var,
                           msg: &Struct_PP_Var) {
        impl_fun!(self.LogWithSource => (inst, lvl, *src, *msg))
    }
}
impl ffi::Struct_PPB_VarArrayBuffer_1_0 {
    pub fn create(&self, len: uint) -> Struct_PP_Var {
        impl_fun!(self.Create => (len as u32))
    }
    pub fn byte_len(&self, var: &Struct_PP_Var) -> Option<uint> {
        let mut len = unsafe { intrinsics::uninit() };
        if impl_fun!(self.ByteLength => (*var, &mut len as *mut libc::uint32_t)) != 0 {
            Some(len as uint)
        } else {
            None
        }
    }
    pub fn map(&self, var: &Struct_PP_Var) -> *mut libc::c_void {
        impl_fun!(self.Map => (*var))
    }
    pub fn unmap(&self, var: &Struct_PP_Var) {
        impl_fun!(self.Unmap => (*var))
    }
}
impl ffi::Struct_PPB_Messaging_1_0 {
    pub unsafe fn post_message(&self, instance: &PP_Instance, msg: &Struct_PP_Var) {
        impl_fun!(self.PostMessage => (*instance, *msg))
    }
}
impl ffi::Struct_PPB_MessageLoop_1_0 {
    pub fn create(&self, instance: &PP_Instance) -> PP_Resource {
        impl_fun!(self.Create => (*instance))
    }
    pub fn get_for_main_thread(&self) -> PP_Resource {
        impl_fun!(self.GetForMainThread)
    }
    pub fn get_current(&self) -> PP_Resource {
        impl_fun!(self.GetCurrent)
    }
    pub fn attach_to_current_thread(&self, msg_loop: &PP_Resource) -> libc::int32_t {
        impl_fun!(self.AttachToCurrentThread => (*msg_loop))
    }
    pub fn run(&self, msg_loop: &PP_Resource) -> libc::int32_t {
        impl_fun!(self.Run => (*msg_loop))
    }
    pub fn post_work(&self, to_loop: &PP_Resource,
                     callback: Struct_PP_CompletionCallback,
                     delay_ms: libc::int64_t) -> libc::int32_t {
        impl_fun!(self.PostWork => (*to_loop, callback, delay_ms))
    }
    pub fn post_quit(&self, msg_loop: &PP_Resource, full: bool) -> libc::int32_t {
        impl_fun!(self.PostQuit => (*msg_loop, full.to_ffi_bool()))
    }
}
impl ffi::Struct_PPB_ImageData_1_0 {
    pub fn native_image_data_format(&self) -> ffi::PP_ImageDataFormat {
        impl_fun!(self.GetNativeImageDataFormat)
    }
    pub fn is_image_data_format_supported(&self, format: ffi::PP_ImageDataFormat) -> bool {
        if impl_fun!(self.IsImageDataFormatSupported => (format)) != 0 {
            true
        } else {
            false
        }
    }
    pub fn create(&self,
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
    pub fn describe(&self, img: PP_Resource) -> Option<ffi::Struct_PP_ImageDataDesc> {
        let mut desc = unsafe { intrinsics::uninit() };
        if impl_fun!(self.Describe => (img,
                                       &mut desc as *mut ffi::Struct_PP_ImageDataDesc)) != 0 {
            Some(desc)
        } else {
            None
        }
    }
    pub fn map(&self, img: &PP_Resource) -> *mut libc::c_void {
        impl_fun!(self.Map => (*img))
    }
    pub fn unmap(&self, img: &PP_Resource) {
        impl_fun!(self.Unmap => (*img))
    }
}
impl ffi::Struct_PPB_URLLoader_1_0 {
    pub fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        let loader = impl_fun!(self.Create => (instance));
        if loader == 0 {
            None
        } else {
            Some(loader)
        }
    }
    pub fn is(&self, res: PP_Resource) -> bool {
        let is = impl_fun!(self.IsURLLoader => (res));
        is != 0
    }
    pub fn open(&self,
                loader: PP_Resource,
                request: PP_Resource,
                callback: ffi::Struct_PP_CompletionCallback) -> super::Result<()> {
        let code = Code::from_i32(impl_fun!(self.Open => (loader, request, callback)));
        code.to_result(|_| () )
    }
}
impl ffi::Struct_PPB_URLRequestInfo_1_0 {
    pub fn create(&self, instance: PP_Instance) -> Option<PP_Resource> {
        let loader = impl_fun!(self.Create => (instance));
        if loader == 0 {
            None
        } else {
            Some(loader)
        }
    }
    pub fn is(&self, res: PP_Resource) -> bool {
        let is = impl_fun!(self.IsURLRequestInfo => (res));
        is != 0
    }
    pub fn property(&self,
                    res: PP_Resource,
                    prop: ffi::PP_URLRequestProperty,
                    value: PP_Var) -> bool {
        let was_set = impl_fun!(self.SetProperty => (res, prop, value));
        was_set != 0
    }
    pub fn append_file_to_body(&self,
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
    pub fn append_blob_to_body(&self,
                               res: PP_Resource,
                               data: &Vec<u8>) -> bool {
        let was_appended = impl_fun!
            (self.AppendDataToBody => (res,
                                       data.as_ptr() as *const libc::c_void,
                                       data.len() as u32));
        was_appended != 0
    }
    
}
impl ffi::Struct_PPB_URLResponseInfo_1_0 {
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsURLResponseInfo => (*res));
        is != 0
    }
    pub fn property(&self,
                    res: &PP_Resource,
                    property: ffi::PP_URLResponseProperty) -> ffi::PP_Var {
        impl_fun!(self.GetProperty => (*res, property))
    }
    pub fn body_as_file(&self, res: &PP_Resource) -> PP_Resource {
        impl_fun!(self.GetBodyAsFileRef => (*res))
    }
                               
}
impl ffi::Struct_PPB_InputEvent_1_0 {
    pub fn request(&self,
                   instance: &PP_Instance,
                   classes: u32) -> Code {
        Code::from_i32(impl_fun!(self.RequestInputEvents => (*instance, classes)))
    }
    pub fn request_filtering(&self,
                             instance: &PP_Instance,
                             classes: u32) -> Code {
        Code::from_i32(impl_fun!(self.RequestFilteringInputEvents => (*instance, classes)))
    }
    pub fn cancel_requests(&self,
                           instance: &PP_Instance,
                           classes: u32) {
        impl_fun!(self.ClearInputEventRequest => (*instance, classes))
    }
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsInputEvent => (*res));
        is != 0
    }
    pub fn type_of(&self, res: &PP_Resource) -> ffi::PP_InputEvent_Type {
        impl_fun!(self.GetType => (*res))
    }
    pub fn timestamp(&self, res: &PP_Resource) -> ffi::PP_TimeTicks {
        impl_fun!(self.GetTimeStamp => (*res))
    }
    pub fn modifiers(&self, res: &PP_Resource) -> u32 {
        impl_fun!(self.GetModifiers => (*res))
    }
}
impl ffi::Struct_PPB_MouseInputEvent_1_1 {
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsMouseInputEvent => (*res));
        is != 0
    }
    pub fn button(&self, res: &PP_Resource) -> ffi::PP_InputEvent_MouseButton {
        impl_fun!(self.GetButton => (*res))
    }
    pub fn point(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        let p = impl_fun!(self.GetPosition => (*res));
        ffi::Struct_PP_FloatPoint {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
    pub fn click_count(&self, res: &PP_Resource) -> i32 {
        impl_fun!(self.GetClickCount => (*res))
    }
    pub fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        let p = impl_fun!(self.GetMovement => (*res));
        ffi::Struct_PP_FloatPoint {
            x: p.x as f32,
            y: p.y as f32,
        }
    }
}
impl ffi::Struct_PPB_KeyboardInputEvent_1_2 {
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsKeyboardInputEvent => (*res));
        is != 0
    }
    pub fn key_code(&self, res: &PP_Resource) -> u32 {
        impl_fun!(self.GetKeyCode => (*res))
    }
    pub fn text(&self, res: &PP_Resource) -> ffi::PP_Var {
        impl_fun!(self.GetCharacterText => (*res))
    }
}
impl ffi::Struct_PPB_TouchInputEvent_1_0 {
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsTouchInputEvent => (*res));
        is != 0
    }
    pub fn count(&self, res: &PP_Resource, list: ffi::PP_TouchListType) -> u32 {
        impl_fun!(self.GetTouchCount => (*res, list))
    }
    pub fn by_index(&self,
                    res: &PP_Resource,
                    list_type: ffi::PP_TouchListType,
                    index: u32) -> ffi::PP_TouchPoint {
        impl_fun!(self.GetTouchByIndex => (*res, list_type, index))
    }
    pub fn by_id(&self,
                 res: &PP_Resource,
                 list_type: ffi::PP_TouchListType,
                 id: u32) -> ffi::PP_TouchPoint {
        impl_fun!(self.GetTouchById => (*res, list_type, id))
    }
}
impl ffi::Struct_PPB_WheelInputEvent_1_0 {
    pub fn is(&self, res: &PP_Resource) -> bool {
        let is = impl_fun!(self.IsWheelInputEvent => (*res));
        is != 0
    }
    pub fn delta(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        impl_fun!(self.GetDelta => (*res))
    }
    pub fn ticks(&self, res: &PP_Resource) -> ffi::PP_FloatPoint {
        impl_fun!(self.GetTicks => (*res))
    }
    pub fn scroll_by_page(&self, res: &PP_Resource) -> bool {
        let r = impl_fun!(self.GetScrollByPage => (*res));
        r != 0
    }
}
impl ffi::Struct_PPB_Graphics3D_1_0 {
    pub fn attrib_max_value(&self, instance: PP_Instance, attribute: i32) -> Result<i32> {
        let mut value: i32 = 0;
        let r = impl_fun!(self.GetAttribMaxValue => (instance,
                                                     attribute,
                                                     &mut value as *mut i32));
        Code::from_i32(r).to_result(|_| value)
    }
    pub fn attribs(&self,
                   ctxt: PP_Resource,
                   attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Result<Vec<u32>> {
        let attribs: Vec<ffi::PP_Graphics3DAttrib> = attribs
            .move_iter()
            .flat_map(|attrib| {
                let v = vec!(attrib, 0);
                v.move_iter()
            })
            .collect();
        let mut attribs = attribs.append_one(ffi::PP_GRAPHICS3DATTRIB_NONE);
        let r = impl_fun!(self.GetAttribs => (ctxt, attribs.as_mut_ptr() as *mut i32));
        let r = Code::from_i32(r);
        if r.is_ok() {
            Ok(attribs.move_iter()
               .enumerate()
               .fold(vec!(), |fold, (i, v)| {
                   if i.rem(&2) != 0 {
                       fold.append_one(v)
                   } else {
                       fold
                   }
               }))
        } else {
            Err(r)
        }
    }
    pub fn status(&self, ctxt: PP_Resource) -> Code {
        Code::from_i32(impl_fun!(self.GetError => (ctxt)))
    }
    pub fn is(&self, res: PP_Resource) -> bool {
        let r = impl_fun!(self.IsGraphics3D => (res));
        r != 0
    }
    pub fn resize_buffers(&self, ctxt: PP_Resource, width: i32, height: i32) -> Code {
        Code::from_i32(impl_fun!(self.ResizeBuffers => (ctxt, width, height)))
    }
    pub fn set_attribs(&self,
                       ctxt: PP_Resource,
                       mut attribs: Vec<ffi::PP_Graphics3DAttrib>) -> Code {
        attribs.push(ffi::PP_GRAPHICS3DATTRIB_NONE);
        Code::from_i32(impl_fun!(self.SetAttribs => (ctxt, attribs.as_ptr() as *const i32)))
    }
    pub fn swap_buffers(&self,
                        ctxt: PP_Resource,
                        callback: ffi::Struct_PP_CompletionCallback) -> Code {
        Code::from_i32(impl_fun!(self.SwapBuffers => (ctxt, callback)))
    }
}
impl ffi::Struct_PPB_View_1_1 {
    pub fn is(&self, res: PP_Resource) -> bool {
        let r = impl_fun!(self.IsView => (res));
        r != 0
    }
    pub fn rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect> {
        let mut dest = unsafe { uninitialized() };
        let ok = impl_fun!(self.GetRect => (res, &mut dest as *mut ffi::Struct_PP_Rect));
        if ok != 0 {
            Some(dest)
        } else {
            None
        }
    }
    pub fn is_fullscreen(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsFullscreen => (res))) != 0
    }
    pub fn is_visible(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsVisible => (res))) != 0
    }
    pub fn is_page_visible(&self, res: PP_Resource) -> bool {
        (impl_fun!(self.IsPageVisible => (res))) != 0
    }
    pub fn clip_rect(&self, res: PP_Resource) -> Option<ffi::Struct_PP_Rect> {
        let mut dest = unsafe { uninitialized() };
        let ok = impl_fun!(self.GetClipRect => (res, &mut dest as *mut ffi::Struct_PP_Rect));
        if ok != 0 {
            Some(dest)
        } else {
            None
        }
    }
    pub fn device_scale(&self, res: PP_Resource) -> f32 {
        impl_fun!(self.GetDeviceScale => (res))
    }
    pub fn css_scale(&self, res: PP_Resource) -> f32 {
        impl_fun!(self.GetCSSScale => (res))
    }
}
pub trait MessagingInterface {
    fn post_message<T: ToVar>(&self, message: T);
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
