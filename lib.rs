//! Rust idiomatic wrapper for the Pepper API.
#![crate_name = "ppapi"]
#![crate_type = "rlib"]
#![experimental]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(default_type_params, struct_variant)]
//#![warn(missing_doc)]

#![allow(dead_code)]

extern crate std;
extern crate native;
#[phase(plugin, link)]
extern crate log;
extern crate collections;
extern crate sync;
extern crate rand;
extern crate serialize;
extern crate http;
extern crate iurl = "url";
extern crate libc;
extern crate core;
extern crate alloc;
extern crate native;
extern crate rustrt;

use core::mem;
use std::{slice, cmp, io, hash, num, collections};
use std::ptr;
use std::intrinsics;
use std::ops;
use std::iter;
use std::clone;
use std::str;
use std::str::MaybeOwned;
use std::result;
use std::collections::hashmap::HashMap;
use std::fmt;
use std::local_data;

use log::LogRecord;

use sync::mutex::Mutex;

#[allow(missing_doc)] pub mod ffi;
pub mod ppp;
pub mod pp;
pub mod ppb;
pub mod gles;
pub mod font;
pub mod imagedata;
pub mod input;
pub mod url;


#[cfg(target_os = "nacl")]
#[link(name = "ppapi_stub", kind = "static")]
extern {}
#[link(name = "helper", kind = "static")]
extern {}

pub type Result<T> = result::Result<T, Code>;

pub fn mount<'s, 't, 'f, 'd>(source: &'s str,
                             target: &'t str,
                             filesystem_type: &'f str,
                             data: &'d str) -> Code {
    let csource = source.to_c_str();
    let ctarget = target.to_c_str();
    let cfilesystem_type = filesystem_type.to_c_str();
    let cdata = data.to_c_str();

    match unsafe {
        ffi::mount(csource.as_ptr(), 
                   ctarget.as_ptr(), 
                   cfilesystem_type.as_ptr(), 
                   0,
                   cdata.as_ptr() as *const libc::c_void)
    } {
        c if c >= 0 => Ok,
        -1 => Failed,
        _ => {
            warn!("Unrecognized failure code");
            Failed
        }
    }
}

pub trait ToFFIBool {
    fn to_ffi_bool(&self) -> ffi::PP_Bool;
}
impl ToFFIBool for bool {
    fn to_ffi_bool(&self) -> ffi::PP_Bool {
        if *self {
            ffi::PP_TRUE
        } else {
            ffi::PP_FALSE
        }
    }
}

#[deriving(Clone, Eq, PartialEq)]
pub enum Code {
    Ok                = ffi::PP_OK as int,
    BadResource       = ffi::PP_ERROR_BADRESOURCE as int,
    BadArgument       = ffi::PP_ERROR_BADARGUMENT as int,
    WrongThread       = ffi::PP_ERROR_WRONG_THREAD as int,
    InProgress        = ffi::PP_ERROR_INPROGRESS as int,
    Failed            = ffi::PP_ERROR_FAILED as int,
    NotSupported      = ffi::PP_ERROR_NOTSUPPORTED as int,
    NoMemory          = ffi::PP_ERROR_NOMEMORY as int,
    ContextLost       = ffi::PP_ERROR_CONTEXT_LOST as int,
    CompletionPending = ffi::PP_OK_COMPLETIONPENDING as int,
}
impl fmt::Show for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ok => f.pad("ok"),
            BadResource => f.pad("bad resource"),
            BadArgument => f.pad("bad argument"),
            WrongThread => f.pad("wrong thread"),
            InProgress  => f.pad("in-progress"),
            Failed      => f.pad("failed"),
            NotSupported=> f.pad("not supported"),
            NoMemory    => f.pad("no memory"),
            ContextLost => f.pad("context lost"),
            CompletionPending => f.pad("completion callback pending"),
        }
    }
}
impl Code {
    pub fn from_i32(code: i32) -> Code {
        match code {
            ffi::PP_OK => Ok,
            ffi::PP_OK_COMPLETIONPENDING => CompletionPending,
            ffi::PP_ERROR_BADRESOURCE => BadResource,
            ffi::PP_ERROR_BADARGUMENT => BadArgument,
            ffi::PP_ERROR_WRONG_THREAD => WrongThread,
            ffi::PP_ERROR_INPROGRESS => InProgress,
            ffi::PP_ERROR_FAILED => Failed,
            ffi::PP_ERROR_NOTSUPPORTED => NotSupported,
            ffi::PP_ERROR_NOMEMORY => NoMemory,
            ffi::PP_ERROR_CONTEXT_LOST => ContextLost,
            _ => fail!("Invalid code!"),
        }
    }
    pub fn to_i32(self) -> i32 {
        match self {
            Ok                => ffi::PP_OK,
            CompletionPending => ffi::PP_OK_COMPLETIONPENDING,
            BadResource => ffi::PP_ERROR_BADRESOURCE,
            BadArgument => ffi::PP_ERROR_BADARGUMENT,
            WrongThread => ffi::PP_ERROR_WRONG_THREAD,
            InProgress  => ffi::PP_ERROR_INPROGRESS,
            Failed      => ffi::PP_ERROR_FAILED,
            NotSupported=> ffi::PP_ERROR_NOTSUPPORTED,
            NoMemory    => ffi::PP_ERROR_NOMEMORY,
            ContextLost => ffi::PP_ERROR_CONTEXT_LOST,
        }
    }
    pub fn to_result<T>(self, ok: |Code| -> T) -> Result<T> {
        if self.is_ok() {
            result::Ok(ok(self))
        } else {
            result::Err(self)
        }
    }
    pub fn is_ok(&self) -> bool {
        match self {
            &Ok | &CompletionPending => true,
            _ => false,
        }
    }
    pub fn expect(self, msg: &str) {
        if !self.is_ok() {
            fail!("Expected success: Code: {code:s} Message: {msg:s}",
                  code=self.to_string(), msg=msg)
        }
    }
    pub fn map<T>(self, take: T) -> Option<T> {
        if self.is_ok() {
            Some(take)
        } else {
            None
        }
    }
}

impl ops::Add<ffi::Struct_PP_Point, ffi::Struct_PP_Point> for ffi::Struct_PP_Point {
    fn add(&self, rhs: &ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl ops::Sub<ffi::Struct_PP_Point, ffi::Struct_PP_Point> for ffi::Struct_PP_Point {
    fn sub(&self, rhs: &ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl ops::Mul<ffi::Struct_PP_Point, ffi::Struct_PP_Point> for ffi::Struct_PP_Point {
    fn mul(&self, rhs: &ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}
impl ops::Div<ffi::Struct_PP_Point, ffi::Struct_PP_Point> for ffi::Struct_PP_Point {
    fn div(&self, rhs: &ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
impl ops::Add<ffi::Struct_PP_Size, ffi::Struct_PP_Size> for ffi::Struct_PP_Size {
    fn add(&self, rhs: &ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}
impl ops::Sub<ffi::Struct_PP_Size, ffi::Struct_PP_Size> for ffi::Struct_PP_Size {
    fn sub(&self, rhs: &ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
impl ops::Mul<ffi::Struct_PP_Size, ffi::Struct_PP_Size> for ffi::Struct_PP_Size {
    fn mul(&self, rhs: &ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}
impl ops::Div<ffi::Struct_PP_Size, ffi::Struct_PP_Size> for ffi::Struct_PP_Size {
    fn div(&self, rhs: &ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}
impl num::Zero for ffi::Struct_PP_Point {
    fn zero() -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: 0,
            y: 0,
        }
    }
    fn is_zero(&self) -> bool {
        match self {
            &ffi::Struct_PP_Point {
                x: 0, y: 0
            } => true,
            _ => false,
        }
    }
}
impl cmp::PartialEq for ffi::Struct_PP_Point {
    fn eq(&self, rhs: &ffi::Struct_PP_Point) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }
}
impl cmp::Eq for ffi::Struct_PP_Point {}

impl cmp::PartialEq for ffi::Struct_PP_FloatPoint {
    fn eq(&self, rhs: &ffi::Struct_PP_FloatPoint) -> bool {
        self.x == rhs.x && self.y == rhs.y
    }
}
impl cmp::Eq for ffi::Struct_PP_FloatPoint {}
impl clone::Clone for ffi::Struct_PP_FloatPoint {
    fn clone(&self) -> ffi::Struct_PP_FloatPoint {
        ffi::Struct_PP_FloatPoint {
            x: self.x,
            y: self.y,
        }
    }
}

pub type Point = ffi::PP_Point;
pub type FloatPoint = ffi::PP_FloatPoint;
pub type TouchPoint = ffi::PP_TouchPoint;
pub type Rect = ffi::PP_Rect;
pub type Ticks = ffi::PP_TimeTicks;
pub type Time = ffi::PP_Time;

// duplicated here so we don't have such a long name for this.
#[deriving(Eq, PartialEq, Hash, Clone)]
pub struct Size {
    pub width:  u32,
    pub height: u32,
}
impl Size {
    pub fn new(width: u32, height: u32) -> Size {
        Size {
            width: width,
            height: height,
        }
    }
    // This uses a by-val so we get a 'free' compile time assertion;
    // if Self && ffi::PP_Size aren't the same size rustc will refuse
    // to compile this mod, though with a not very helpful message.
    fn to_ffi(self) -> ffi::PP_Size {
        use core::mem::transmute;
        unsafe { transmute(self) }
    }
}

pub trait ToOption<From> {
    fn to_option(from: &From) -> Option<Self>;
}

pub enum ResourceType {
    WheelInputEventRes,
    WebSocketRes,
    ViewRes,
    UrlResponseInfoRes,
    UrlRequestInfoRes,
    UrlLoaderRes,
    UdpSocketRes,
    TrueTypeFontRes,
    TouchInputEventRes,
    TcpSocketRes,
    NetworkMonitorRes,
    NetworkListRes,
    NetworkAddressRes,
    MouseInputEventRes,
    MessageLoopRes,
    KeyboardInputEventRes,
    ImageDataRes,
    IMEInputEventRes,
    HostResolverRes,
    Graphics3DRes,
    Graphics2DRes,
    FontRes,
    FilesystemRes,
    FileRefRes,
    FileIoRes,
    AudioConfigRes,
    AudioRes,
}

pub trait Resource {
    fn unwrap(&self) -> ffi::PP_Resource;

    fn type_of(&self) -> ResourceType;
}
pub trait ContextResource {
    fn get_device(&self) -> ffi::PP_Resource;
}
#[deriving(Hash, Eq, PartialEq)] pub struct Context3d(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct Context2d(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct View(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct MessageLoop(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct KeyboardInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct MouseInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct WheelInputEvent(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct TouchInputEvent(ffi::PP_Resource);
#[deriving(Eq)]
pub struct IMEInputEvent {
    res: ffi::PP_Resource,
    pub string: String,
    segments_len: uint,
}
#[deriving(Hash, Eq, PartialEq)] pub struct UrlLoader(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct UrlRequestInfo(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct UrlResponseInfo(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct Font(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct ImageData(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct FileRef(ffi::PP_Resource);
#[deriving(Clone, Hash, Eq, PartialEq)]
pub struct FileSliceRef(FileRef,
                        Option<i64>,
                        Option<i64>);
#[deriving(Hash, Eq, PartialEq)] pub struct FileIo(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq)] pub struct Filesystem(ffi::PP_Resource);

macro_rules! impl_resource_for(
    ($ty:ty $type_:ident) => (
        impl Resource for $ty {
            #[inline]
            fn unwrap(&self) -> ffi::PP_Resource {
                unsafe { mem::transmute_copy(self) }
            }
            #[inline]
            fn type_of(&self) -> ResourceType {
                $type_
            }
        }
        impl $ty {
            pub fn new(res: ffi::PP_Resource) -> $ty {
                unsafe {
                    mem::transmute_copy(&res)
                }
            }
        }
        impl ToOption<ffi::PP_Resource> for $ty {
            fn to_option(from: &ffi::PP_Resource) -> Option<$ty> {
                if *from == 0 {
                    None
                } else {
                    Some(unsafe {
                        mem::transmute_copy(from)
                    })
                }
            }
        }
    )
)
macro_rules! impl_clone_drop_for(
    ($ty:ty) => (
        impl Clone for $ty {
            fn clone(&self) -> $ty {
                (ppb::get_core().AddRefResource.unwrap())(self.unwrap());
                unsafe {
                    mem::transmute_copy(self)
                }
            }
        }
        impl Drop for $ty {
            fn drop(&mut self) {
                (ppb::get_core().ReleaseResource.unwrap())(self.unwrap());
            }
        }
    )
)
impl_resource_for!(Context3d Graphics3DRes)
impl_clone_drop_for!(Context3d)
impl_resource_for!(Context2d Graphics2DRes)
impl_clone_drop_for!(Context2d)
impl_resource_for!(View ViewRes)
impl_clone_drop_for!(View)
impl_resource_for!(MessageLoop MessageLoopRes)
impl_clone_drop_for!(MessageLoop)
impl_resource_for!(KeyboardInputEvent KeyboardInputEventRes)
impl_clone_drop_for!(KeyboardInputEvent)
impl_resource_for!(MouseInputEvent MouseInputEventRes)
impl_clone_drop_for!(MouseInputEvent)
impl_resource_for!(WheelInputEvent WheelInputEventRes)
impl_clone_drop_for!(WheelInputEvent)
impl_resource_for!(TouchInputEvent TouchInputEventRes)
impl_clone_drop_for!(TouchInputEvent)
impl_resource_for!(Font FontRes)
impl_clone_drop_for!(Font)
impl_resource_for!(ImageData ImageDataRes)
impl_clone_drop_for!(ImageData)
impl_resource_for!(FileRef FileRefRes)
impl_clone_drop_for!(FileRef)
impl_resource_for!(FileIo FileIoRes)
impl_clone_drop_for!(FileIo)
impl_resource_for!(Filesystem FilesystemRes)
impl_clone_drop_for!(Filesystem)

impl Resource for IMEInputEvent {
    fn unwrap(&self) -> ffi::PP_Resource {
        self.res
    }
    fn type_of(&self) -> ResourceType {
        IMEInputEventRes
    }
}
impl IMEInputEvent {
    pub fn new(res: ffi::PP_Resource) -> IMEInputEvent {
        let var = (ppb::get_ime_event().GetText.unwrap())(res);
        let string = StringVar::new_from_var(var).to_string();
        let seg_len = (ppb::get_ime_event().GetSegmentNumber.unwrap())(res);
        IMEInputEvent {
            res: res,
            string: string,
            segments_len: seg_len as uint,
        }
    }
}
impl cmp::PartialEq for IMEInputEvent {
    fn eq(&self, rhs: &IMEInputEvent) -> bool {
        self.res == rhs.res
    }
}
impl<T: std::hash::Writer> hash::Hash<T> for IMEInputEvent {
    fn hash(&self, s: &mut T) {
        self.res.hash(s)
    }
}

impl_clone_drop_for!(IMEInputEvent)
impl_resource_for!(UrlLoader UrlLoaderRes)
impl_clone_drop_for!(UrlLoader)
impl_resource_for!(UrlRequestInfo UrlRequestInfoRes)
impl_clone_drop_for!(UrlRequestInfo)
impl_resource_for!(UrlResponseInfo UrlResponseInfoRes)
impl_clone_drop_for!(UrlResponseInfo)

impl ContextResource for Context3d {
    fn get_device(&self) -> ffi::PP_Resource {
        self.unwrap()
    }
}
impl ContextResource for Context2d {
    fn get_device(&self) -> ffi::PP_Resource {
        self.unwrap()
    }
}
impl View {
    #[inline] pub fn rect(&self) -> Option<Rect> {
        ppb::get_view().rect(self.unwrap())
    }
    #[inline] pub fn is_fullscreen(&self) -> bool {
        ppb::get_view().is_fullscreen(self.unwrap())
    }
    #[inline] pub fn is_visible(&self) -> bool {
        ppb::get_view().is_visible(self.unwrap())
    }
    #[inline] pub fn is_page_visible(&self) -> bool {
        ppb::get_view().is_page_visible(self.unwrap())
    }
    #[inline] pub fn clip_rect(&self) -> Option<Rect> {
        ppb::get_view().clip_rect(self.unwrap())
    }
    #[inline] pub fn device_scale(&self) -> f32 {
        ppb::get_view().device_scale(self.unwrap())
    }
    #[inline] pub fn css_scale(&self) -> f32 {
        ppb::get_view().css_scale(self.unwrap())
    }
}
impl MessageLoop {
    pub fn get_main_loop() -> MessageLoop {
        MessageLoop((ppb::get_message_loop().GetForMainThread.unwrap())())
    }
    pub fn is_attached() -> bool {
        unsafe {
            (ppb::get_message_loop().GetCurrent.unwrap())() == mem::transmute(0i32)
        }
    }
    pub fn current() -> MessageLoop {
        MessageLoop((ppb::get_message_loop().GetCurrent.unwrap())())
    }
    pub fn attach_to_current_thread(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().AttachToCurrentThread.unwrap())(self.unwrap()))
    }
    /// Blocking
    pub fn run_loop(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().Run.unwrap())(self.unwrap()))
    }
    pub fn post_work(&self, work: Box<proc()>, delay: i64) -> Code {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let work: Box<proc()> = unsafe { mem::transmute(user) };
            if status != ffi::PP_OK {
                warn!("work_callback called without status == ffi::PP_OK");
                return;
            }
            (*work)();
        }

        let comp_cb = unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(work))
        };
        match (ppb::get_message_loop().PostWork.unwrap())(self.unwrap(), comp_cb, delay) {
            ffi::PP_ERROR_BADARGUMENT => fail!("internal error: completion callback was null?"),
            c => Code::from_i32(c),
        }
    }
    pub fn pause_loop(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().PostQuit.unwrap())(self.unwrap(), ffi::PP_FALSE))
    }

    ///
    pub fn shutdown(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().PostQuit.unwrap())(self.unwrap(), ffi::PP_TRUE))
    }
}

#[deriving(Clone)]
pub enum AnyVar {
    Null,
    Undefined,
    Bool(bool),
    I32(i32),
    F64(f64),
    String(StringVar),
    Object(ObjectVar),
    Array(ArrayVar),
    Dictionary(DictionaryVar),
    ArrayBuffer(ArrayBufferVar),
}
#[deriving(Clone, Eq, PartialEq)]
pub struct NullVar;
#[deriving(Clone, Eq, PartialEq)]
pub struct UndefinedVar;
#[deriving(Eq, PartialEq, Hash)]
pub struct StringVar     (i64);
#[deriving(Eq, PartialEq, Hash)]
pub struct ObjectVar     (i64);
#[deriving(Eq, PartialEq, Hash)]
pub struct ArrayVar      (i64);
#[deriving(Eq, PartialEq, Hash)]
pub struct DictionaryVar (i64);
#[deriving(Eq, PartialEq, Hash)]
pub struct ArrayBufferVar(i64);

pub trait ByRefVar {
    fn get_id(&self) -> i64;
}
pub trait ToVar {
    fn to_var(&self) -> ffi::PP_Var;
    #[inline]
    fn to_any(&self) -> AnyVar {
        AnyVar::new(self.to_var())
    }
}
// this is here to help the macros.
trait VarCtor {
    // assume var is of the correct type.
    fn ctor(var: ffi::PP_Var) -> Self;
}
pub trait FromVar {
    fn from_var(var: ffi::PP_Var) -> Option<Self>;
}
impl<T: ToVar> ToVar for Option<T> {
    fn to_var(&self) -> ffi::PP_Var {
        match self {
            &Some(ref var) => var.to_var(),
            &None => {UndefinedVar}.to_var(),
        }
    }
}
/// by default all functions return false/None so one doesn't have to impl all of them.
pub trait Var: clone::Clone {
    #[inline] fn is_null(&self) -> bool { false }
    #[inline] fn is_undefined(&self) -> bool { false }
    #[inline] fn is_a_bool(&self) -> bool { false }
    #[inline] fn is_an_i32(&self) -> bool { false }
    #[inline] fn is_a_f64(&self) -> bool { false }
    #[inline] fn is_a_string(&self) -> bool { false }
    #[inline] fn is_an_object(&self) -> bool { false }
    #[inline] fn is_an_array(&self) -> bool { false }
    #[inline] fn is_a_dictionary(&self) -> bool { false }
    #[inline] fn is_an_array_buffer(&self) -> bool { false }
    #[inline] fn is_a_resource(&self) -> bool { false }
}

impl Var for AnyVar {
    #[inline]
    fn is_null(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_NULL }
    #[inline]
    fn is_undefined(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_UNDEFINED }
    #[inline]
    fn is_a_bool(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_BOOL }
    #[inline]
    fn is_an_i32(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_INT32 }
    #[inline]
    fn is_a_f64(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_DOUBLE }
    #[inline]
    fn is_a_string(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_STRING }
    #[inline]
    fn is_an_object(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_OBJECT }
    #[inline]
    fn is_an_array(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_ARRAY }
    #[inline]
    fn is_a_dictionary(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_DICTIONARY }
    #[inline]
    fn is_an_array_buffer(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_ARRAY_BUFFER }
    #[inline]
    fn is_a_resource(&self) -> bool { self.to_var()._type == ffi::PP_VARTYPE_RESOURCE }
}
impl clone::Clone for ffi::PP_Var {
    fn clone(&self) -> ffi::PP_Var {
        *self
    }
}
impl Var for ffi::PP_Var {
    #[inline]
    fn is_null(&self) -> bool { self._type == ffi::PP_VARTYPE_NULL }
    #[inline]
    fn is_undefined(&self) -> bool { self._type == ffi::PP_VARTYPE_UNDEFINED }
    #[inline]
    fn is_a_bool(&self) -> bool { self._type == ffi::PP_VARTYPE_BOOL }
    #[inline]
    fn is_an_i32(&self) -> bool { self._type == ffi::PP_VARTYPE_INT32 }
    #[inline]
    fn is_a_f64(&self) -> bool { self._type == ffi::PP_VARTYPE_DOUBLE }
    #[inline]
    fn is_a_string(&self) -> bool { self._type == ffi::PP_VARTYPE_STRING }
    #[inline]
    fn is_an_object(&self) -> bool { self._type == ffi::PP_VARTYPE_OBJECT }
    #[inline]
    fn is_an_array(&self) -> bool { self._type == ffi::PP_VARTYPE_ARRAY }
    #[inline]
    fn is_a_dictionary(&self) -> bool { self._type == ffi::PP_VARTYPE_DICTIONARY }
    #[inline]
    fn is_an_array_buffer(&self) -> bool { self._type == ffi::PP_VARTYPE_ARRAY_BUFFER }
    #[inline]
    fn is_a_resource(&self) -> bool { self._type == ffi::PP_VARTYPE_RESOURCE }
}

macro_rules! impl_clone_drop_for(
    ($ty:ty -> $is_true_name:ident) => (
        impl Drop for $ty {
            fn drop(&mut self) {
                (ppb::get_var().Release.unwrap())(self.to_var());
            }
        }
        impl clone::Clone for $ty {
            fn clone(&self) -> $ty {
                (ppb::get_var().AddRef.unwrap())(self.to_var());
                unsafe {
                    mem::transmute_copy(self)
                }
            }
        }
        impl Var for $ty {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl<'a> Var for &'a $ty {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl Var for Box<$ty> {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl FromVar for $ty {
            fn from_var(var: ffi::PP_Var) -> Option<$ty> {
                if unsafe { var.$is_true_name() && ffi::id_from_var(var) != 0 } {
                    Some(VarCtor::ctor(var))
                } else {
                    None
                }
            }
        }
    )
)
impl_clone_drop_for!(StringVar -> is_a_string)
impl_clone_drop_for!(ObjectVar -> is_an_object)
impl_clone_drop_for!(ArrayVar -> is_an_array)
impl_clone_drop_for!(DictionaryVar -> is_a_dictionary)
impl_clone_drop_for!(ArrayBufferVar -> is_an_array_buffer)

macro_rules! impl_var_for(
    ($ty:ty -> $is_true_name:ident) => (
        impl Var for $ty {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl<'a> Var for &'a $ty {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl Var for Box<$ty> {
            #[inline] fn $is_true_name(&self) -> bool { true }
        }
        impl FromVar for $ty {
            fn from_var(var: ffi::PP_Var) -> Option<$ty> {
                if var.$is_true_name() {
                    Some(VarCtor::ctor(var))
                } else {
                    None
                }
            }
        }
    )
)
impl_var_for!(NullVar -> is_null)
impl_var_for!(UndefinedVar -> is_undefined)
impl_var_for!(bool -> is_a_bool)
impl_var_for!(i32 -> is_an_i32)
impl_var_for!(f64 -> is_a_f64)

impl VarCtor for NullVar {
    fn ctor(_: ffi::PP_Var) -> NullVar {
        NullVar
    }
}
impl VarCtor for UndefinedVar {
    fn ctor(_: ffi::PP_Var) -> UndefinedVar {
        UndefinedVar
    }
}
impl VarCtor for bool {
    fn ctor(v: ffi::PP_Var) -> bool {
        unsafe {
            ffi::bool_from_var(v) != 0
        }
    }
}
impl VarCtor for i32 {
    fn ctor(v: ffi::PP_Var) -> i32 {
        unsafe {
            ffi::i32_from_var(v)
        }
    }
}
impl VarCtor for f64 {
    fn ctor(v: ffi::PP_Var) -> f64 {
        unsafe {
            ffi::f64_from_var(v)
        }
    }
}
impl VarCtor for StringVar {
    fn ctor(v: ffi::PP_Var) -> StringVar {
        StringVar::new_from_var(v)
    }
}
impl VarCtor for ObjectVar {
    fn ctor(v: ffi::PP_Var) -> ObjectVar {
        ObjectVar::new_from_var(v)
    }
}
impl VarCtor for ArrayVar {
    fn ctor(v: ffi::PP_Var) -> ArrayVar {
        ArrayVar::new_from_var(v)
    }
}
impl VarCtor for DictionaryVar {
    fn ctor(v: ffi::PP_Var) -> DictionaryVar {
        DictionaryVar::new_from_var(v)
    }
}
impl VarCtor for ArrayBufferVar {
    fn ctor(v: ffi::PP_Var) -> ArrayBufferVar {
        ArrayBufferVar::new_from_var(v)
    }
}

pub trait ToStringVar {
    fn to_string_var(&self) -> StringVar;
}

impl<'a> Var for &'a str {
    #[inline] fn is_a_string(&self) -> bool { true }
}
impl<'a> ToStringVar for &'a str {
    #[inline] fn to_string_var(&self) -> StringVar {
        StringVar::new_from_str(*self)
    }
}
impl ToStringVar for StringVar {
    fn to_string_var(&self) -> StringVar {
        self.clone()
    }
}
impl Var for String {
    #[inline] fn is_a_string(&self) -> bool { true }
}
impl ToStringVar for String {
    #[inline] fn to_string_var(&self) -> StringVar {
        StringVar::new_from_str(self.as_slice())
    }
}
impl<'a, T: ToVar> Var for &'a [T] {
    #[inline] fn is_an_array(&self) -> bool { true }
}
impl<T: ToVar + Clone> Var for Vec<T> {
    #[inline] fn is_an_array(&self) -> bool { true }
}

impl ToVar for AnyVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        match self {
            &Null => {NullVar}.to_var(),
            &Undefined => {UndefinedVar}.to_var(),
            &Bool(b) => b.to_var(),
            &I32(i) => i.to_var(),
            &F64(f) => f.to_var(),
            &String(ref v) => v.to_var(),
            &Object(ref v) => v.to_var(),
            &Array(ref v) => v.to_var(),
            &Dictionary(ref v) => v.to_var(),
            &ArrayBuffer(ref v) => v.to_var(),
        }
    }
    #[inline]
    fn to_any(&self) -> AnyVar {
        self.clone()
    }
}

impl ToVar for NullVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::make_null_var()
        }
    }
}
impl ToVar for UndefinedVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::make_undefined_var()
        }
    }
}
impl ToVar for StringVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::string_id_to_var(self.get_id())
        }
    }
}
impl ToVar for ObjectVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::object_id_to_var(self.get_id())
        }
    }
}
impl ToVar for ArrayVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::array_id_to_var(self.get_id())
        }
    }
}
impl ToVar for DictionaryVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::dictionary_id_to_var(self.get_id())
        }
    }
}
impl ToVar for ArrayBufferVar {
    #[inline]
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::array_buffer_id_to_var(self.get_id())
        }
    }
}
macro_rules! impl_by_ref_var(
    ($ty:ty) => (
        impl ByRefVar for $ty {
            fn get_id(&self) -> i64 {
                unsafe { mem::transmute_copy(self) }
            }
        }
    )
)
impl_by_ref_var!(StringVar)
impl_by_ref_var!(ObjectVar)
impl_by_ref_var!(ArrayVar)
impl_by_ref_var!(DictionaryVar)
impl_by_ref_var!(ArrayBufferVar)

macro_rules! impl_to_var_int(
    ($ty:ty) => (
        impl<'s> ToVar for &'s $ty {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::i32_to_var(**self as i32) };
            }
        }
        impl ToVar for $ty {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::i32_to_var(*self as i32) };
            }
        }
        impl ToVar for Box<$ty> {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::i32_to_var(**self as i32) };
            }
        }
    )
)
impl_to_var_int!(i8)
impl_to_var_int!(i16)
impl_to_var_int!(i32)

macro_rules! impl_to_var_float(
    ($ty:ty) => (
        impl<'s> ToVar for &'s $ty {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::f64_to_var(**self as f64) };
            }
        }
        impl ToVar for $ty {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::f64_to_var(*self as f64) };
            }
        }
        impl ToVar for Box<$ty> {
            fn to_var(&self) -> ffi::PP_Var {
                return unsafe { ffi::f64_to_var(**self as f64) };
            }
        }
    )
)
impl_to_var_float!(f32)
impl_to_var_float!(f64)

impl<'s> ToVar for &'s bool {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(**self as i32)
        }
    }
}
impl ToVar for bool {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(*self as i32)
        }
    }
}
impl ToVar for Box<bool> {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(**self as i32)
        }
    }
}

impl NullVar {
    #[inline]
    pub fn new() -> NullVar {
        NullVar
    }
}
impl UndefinedVar {
    #[inline]
    pub fn new() -> UndefinedVar {
        UndefinedVar
    }
}
impl AnyVar {
    #[inline]
    fn new(var: ffi::PP_Var) -> AnyVar {
        if var.is_null() {
            Null
        } else if var.is_undefined() {
            Undefined
        } else if var.is_a_bool() {
            Bool(unsafe { ffi::bool_from_var(var) != 0 })
        } else if var.is_an_i32() {
            I32(unsafe { ffi::i32_from_var(var) })
        } else if var.is_a_f64() {
            F64(unsafe { ffi::f64_from_var(var) })
        } else if var.is_a_string() {
            String(StringVar::new_from_var(var))
        } else if var.is_an_object() {
            Object(ObjectVar::new_from_var(var))
        } else if var.is_an_array() {
            Array(ArrayVar::new_from_var(var))
        } else if var.is_a_dictionary() {
            Dictionary(DictionaryVar::new_from_var(var))
        } else if var.is_an_array_buffer() {
            ArrayBuffer(ArrayBufferVar::new_from_var(var))
        } else if var.is_a_resource() {
            error!("Resource vars aren't implemented");
            Undefined
        } else {
            error!("Var doesn't have a known type");
            Undefined
        }
    }
    #[inline]
    fn new_bumped(var: ffi::PP_Var) -> AnyVar {
        let v = AnyVar::new(var);
        // bump the ref count:
        unsafe { mem::forget(v.clone()) };
        v
    }
    #[inline] #[allow(dead_code)]
    fn is_ref_counted(&self) -> bool {
        self.is_a_string() ||
            self.is_an_object() ||
            self.is_an_array() ||
            self.is_a_dictionary() ||
            self.is_an_array_buffer() ||
            self.is_a_resource()
    }
}

impl fmt::Show for StringVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = unsafe {
            let mut len: u32 = intrinsics::uninit();
            let buf = (ppb::get_var().VarToUtf8.unwrap())
                (self.to_var(), 
                 &mut len as *mut u32);
            let len = len;
            str::raw::from_buf_len(buf as *const u8, len as uint)
        };
        f.pad(str.as_slice())
    }
}
impl StringVar {
    pub fn new<T: Str>(v: &T) -> StringVar {
        let string = v.as_slice();
        StringVar::new_from_str(string)
    }
    pub fn new_from_str(v: &str) -> StringVar {
        let len = v.len();
        let var = v.with_c_str(|p| {
            (ppb::get_var().VarFromUtf8.unwrap())
                (p,
                 len as u32)
        });
        return StringVar(unsafe { ffi::id_from_var(var) } );
    }
    pub fn new_from_var(v: ffi::PP_Var) -> StringVar {
        StringVar(unsafe { ffi::id_from_var(v) })
    }
}
impl ToVar for String {
    fn to_var(&self) -> ffi::PP_Var {
        (ppb::get_var().VarFromUtf8.unwrap())
            (self.as_slice().as_ptr() as *const i8,
             self.len() as u32)
    }
}
impl<'a> ToVar for &'a str {
    fn to_var(&self) -> ffi::PP_Var {
        (ppb::get_var().VarFromUtf8.unwrap())
            (self.as_ptr() as *const i8,
             self.len() as u32)
    }
}
impl ObjectVar {
    fn new_from_var(v: ffi::PP_Var) -> ObjectVar {
        ObjectVar(unsafe { ffi::id_from_var(v) })
    }
}
impl ArrayVar {
    fn new_from_var(v: ffi::PP_Var) -> ArrayVar {
        ArrayVar(unsafe { ffi::id_from_var(v) })
    }
}
impl DictionaryVar {
    fn new_from_var(v: ffi::PP_Var) -> DictionaryVar {
        DictionaryVar(unsafe { ffi::id_from_var(v) })
    }
}
impl ArrayBufferVar {
    fn new_from_var(v: ffi::PP_Var) -> ArrayBufferVar {
        ArrayBufferVar(unsafe { ffi::id_from_var(v) })
    }
}

pub struct Console(ffi::PP_Instance);
impl Console {
    fn unwrap(&self) -> ffi::PP_Instance {
        let &Console(inst) = self;
        inst
    }
}

fn parse_args(argc: u32,
              argk: *mut *const libc::c_char,
              argv: *mut *const libc::c_char) -> HashMap<String, String> {
    let mut args: HashMap<String, String> = HashMap::new();
    for i in iter::range(0, argc as int) {
        unsafe {
            args.swap(str::raw::from_c_str(*argk.offset(i) as *const i8),
                      str::raw::from_c_str(*argv.offset(i) as *const i8));
        }
    }
    return args;
}
pub type OptionalName = Option<MaybeOwned<'static>>;
trait Callback<TData: Send> {
    fn to_ffi_callback(self,
                       name: OptionalName,
                       take: TData) -> ffi::Struct_PP_CompletionCallback;
    fn sync_call(self,
                 instance: Instance,
                 name: OptionalName,
                 take: Option<TData>,
                 code: Code);
}
trait CompletionCallback {
    fn call(self, code: Code);
}
struct CompletionCallbackWithCode<TData> {
    // a name for debugging; otherwise unused.
    name: OptionalName,
    instance: Instance, 
    data: TData,
    callback: proc(Instance, Code, Option<TData>),
}
struct CompletionCallbackWithoutCode<TData> {
    // a name for debugging; otherwise unused.
    name: OptionalName,
    instance: Instance,
    data: TData,
    callback: proc(Instance, TData),
}
impl<TData: Send> CompletionCallback for Box<CompletionCallbackWithoutCode<TData>> {
    fn call(self, code: Code) {
        let box CompletionCallbackWithoutCode {
            name: name,
            instance: instance,
            data: data,
            callback: callback,
        } = self;
        instance.set_current();
        if code != Ok {
            warn!("callback `{}` called with code: `{}`", name, code);
        } else {
            info!("entering callback: `{}`", name);
            let mut callback = Some(callback);
            let mut data = Some(data);
            let _ = entry::try_block(|| {
                let callback = callback.take_unwrap();
                let data = data.take_unwrap();
                callback(instance, data)
            });
        }
    }
}
impl<TData: Send> CompletionCallback for Box<CompletionCallbackWithCode<TData>> {
    fn call(self, code: Code) {
        let box CompletionCallbackWithCode {
            name: name,
            instance: instance,
            data: data,
            callback: callback,
        } = self;
        instance.set_current();
        info!("entering callback: `{}` with code: `{}`", name, code);
        let mut callback = Some(callback);
        let mut data = Some(data);
        let _ = entry::try_block(|| {
            let callback = callback.take_unwrap();
            callback(instance, code, code.map(data.take_unwrap()))
        });
    }
}
impl<TData: Send> Callback<TData> for proc(Instance, TData) {
    fn to_ffi_callback(self,
                       name: OptionalName,
                       take: TData) -> ffi::Struct_PP_CompletionCallback {
        let callback = box CompletionCallbackWithoutCode {
            instance: Instance::current(),
            name: name,
            data: take,
            callback: self,
        };
        new_ffi_callback(box callback)
    }
    fn sync_call(self,
                 instance: Instance,
                 name: OptionalName,
                 take: Option<TData>,
                 code: Code) {
        if code != Ok {
            warn!("callback `{}` called with code: `{}`", name, code);
        } else if take.is_none() {
            warn!("callback `{}` called with a success code but no data", name);
        } else {
            info!("entering callback: `{}`", name);
            self(instance, take.unwrap())
        }
    }
}
impl<TData: Send> Callback<TData> for proc(Instance, Code, Option<TData>) {
    fn to_ffi_callback(self,
                       name: OptionalName,
                       take: TData) -> ffi::Struct_PP_CompletionCallback {
        let callback = box CompletionCallbackWithCode {
            instance: Instance::current(),
            name: name,
            data: take,
            callback: self,
        };
        new_ffi_callback(box callback)
    }
    fn sync_call(self,
                 instance: Instance,
                 name: OptionalName,
                 take: Option<TData>,
                 code: Code) {
        info!("entering callback: `{}` with code: `{}`", name, code);
        self(instance, code, take.and_then(|take| code.map(take) ))
    }
}
type CallbackBox = Box<CompletionCallback>;
fn new_ffi_callback(callback: CallbackBox) -> ffi::Struct_PP_CompletionCallback {
    extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
        let callback: Box<CallbackBox> =
            unsafe { mem::transmute(user) };
        callback.call(Code::from_i32(status))
    }
    unsafe {
        ffi::make_completion_callback(work_callback,
                                      mem::transmute(box callback)) // :(
    }
}
impl ffi::Struct_PP_CompletionCallback {
    pub unsafe fn sync_call(self, code: Code) {
        (self.func)(self.user_data, code.to_i32())
    }
}

pub struct CurrentInstanceLogger;
impl log::Logger for CurrentInstanceLogger {
    fn log(&mut self, record: &LogRecord) {
        use self::ppb::ConsoleInterface;
        let console = Instance::current().console();
        let level = match record.level {
            log::LogLevel(log::ERROR) => ffi::PP_LOGLEVEL_ERROR,
            log::LogLevel(log::WARN)  => ffi::PP_LOGLEVEL_WARNING,
            log::LogLevel(log::INFO)  => ffi::PP_LOGLEVEL_TIP,
            log::LogLevel(_)          => ffi::PP_LOGLEVEL_LOG,
        };

        let str = format!("{} ({}:{}): {}",
                          record.module_path,
                          record.file,
                          record.line,
                          record.args);
        console.log_to_browser(level,
                               str.to_var());
    }
}
struct CurrentInstanceStdIo {
    level: ffi::PP_LogLevel,
    fd:    i32,
    buffer: Vec<u8>,
}
impl Writer for CurrentInstanceStdIo {
    fn write(&mut self, mut buf: &[u8]) -> io::IoResult<()> {
        // Don't write newlines to the console. Also, don't write anything to the console
        // until we get a newline.
        loop {
            let newline_pos_opt = buf.iter().position(|&c| c == '\n' as u8 );
            let newline_pos = match newline_pos_opt {
                Some(pos) => pos,
                None => {
                    self.buffer.push_all(buf);
                    return result::Ok(());
                }
            };
            let rest = buf.slice(0, newline_pos);
            self.buffer.push_all(rest);
            let result = send_to_console_or_terminal(current_instance.get(),
                                                     self.buffer.as_slice(),
                                                     self.level,
                                                     self.fd);
            self.buffer.truncate(0);
            if buf.len() < newline_pos + 2 {
                return result;
            }
            buf = buf.slice_from(newline_pos + 2);
            if result.is_err() || buf.len() == 0 {
                return result;
            }
        }
    }
}
#[no_mangle] #[inline(never)]
fn send_to_console_or_terminal(instance: Option<local_data::Ref<Instance>>,
                               buf: &[u8],
                               lvl: ffi::PP_LogLevel,
                               fd: i32) -> io::IoResult<()> {
    use self::ppb::ConsoleInterface;
    use libc::{fdopen, fwrite, c_void};

    fn last_ditch_effort(buf: &[u8], fd: i32) -> io::IoResult<()> {
        // fallback to good ol' stderr
        unsafe {
            let fs = "w".with_c_str_unchecked(|mode| {
                fdopen(fd, mode)
            });
            if fs.is_null() {
                result::Err(io::IoError {
                    kind: io::ResourceUnavailable,
                    desc: "couldn't open stderr for writing",
                    detail: None,
                })
            } else {
                fwrite(buf.as_ptr() as *const c_void, buf.len() as libc::size_t, 1, fs);
                result::Ok(())
            }
        }
    }

    match instance {
        Some(instance) => {
            let console = (*instance).console();
            match str::from_utf8(buf) {
                Some(s) => {
                    console.log(lvl, s);
                    result::Ok(())
                }
                None => last_ditch_effort(buf, fd),
            }
        }
        None => last_ditch_effort(buf, fd),
    }
}

#[deriving(Clone, Hash, Eq, PartialEq)]
pub struct Instance {
    instance: ffi::PP_Instance,
}
impl ops::Deref<ffi::PP_Instance> for Instance {
    fn deref<'a>(&'a self) -> &'a ffi::PP_Instance {
        &self.instance
    }
}
impl Instance {
    // Note to devs: don't let this fail.
    fn new(inst: ffi::PP_Instance) -> Instance {
        Instance {
            instance: inst,
        }
    }
    pub fn current() -> Instance {
        Instance::opt_current().expect("instance not set in task local storage!")
    }
    pub fn opt_current() -> Option<Instance> {
        current_instance.get().map(|instance| {
            (*instance).clone()
        })
    }
    fn set_current(&self) {
        current_instance.replace(Some(self.clone()));
    }
    fn unset_current() {
        current_instance.replace(None);
    }
    fn unwrap(&self) -> ffi::PP_Instance {
        self.instance
    }

    fn initialize_nacl_io(&self) {
        unsafe {
            ffi::nacl_io_init_ppapi(self.instance,
                                    ppb::get_actual_browser());
        }
    }

    pub fn console(&self) -> Console {
        return Console(self.instance);
    }

    pub fn create_3d_context<AT: slice::Vector<(i32, i32)> + collections::Collection>
        (&self,
         share_with: Option<Context3d>,
         attribs: AT)
         -> result::Result<Context3d, Code> {
             let attribs = attribs.as_slice();
             let mut a = Vec::with_capacity(attribs.len() * 2 + 1);
             for &(k, v) in attribs.iter() {
                 a.push(k);
                 a.push(v);
             }
             a.push(ffi::PP_GRAPHICS3DATTRIB_NONE as i32);
             let a = a;
             let share_with = share_with.unwrap_or_else(|| {
                 unsafe {
                     Context3d(mem::transmute(0i32))
                 }
             });
             let graphics = ppb::get_graphics_3d();
             
             let raw_cxt  = (graphics.Create.unwrap())(self.instance,
                                                       share_with.unwrap(),
                                                       a.as_ptr());

             if raw_cxt == unsafe { mem::transmute(0i32) } {
                 result::Err(Failed)
             } else {
                 result::Ok(Context3d(raw_cxt))
             }
         }
    pub fn bind_context<T: ContextResource>(&self, cxt: &T) -> Code {
        match (ppb::get_instance().BindGraphics.unwrap())
            (self.instance,
             cxt.get_device()) {
            ffi::PP_TRUE => Ok,
            ffi::PP_FALSE => Failed,
            other => {
                format!("unknown truthy value: {:}", other);
                Failed
            }
        }
    }

    pub fn request_input_events(&self, classes: ffi::PP_InputEvent_Class) -> Code {
        Code::from_i32((ppb::get_input_event().RequestInputEvents.unwrap())
                       (self.instance,
                        classes))
    }
    pub fn request_filtering_input_events(&self, classes: ffi::PP_InputEvent_Class) -> Code {
        Code::from_i32((ppb::get_input_event().RequestFilteringInputEvents.unwrap())
                       (self.instance,
                        classes))
    }
    pub fn clear_input_event_request(&self, classes: ffi::PP_InputEvent_Class) {
        (ppb::get_input_event().ClearInputEventRequest.unwrap())
            (self.instance,
             classes);
    }

    pub fn create_image(&self,
                        format: Option<imagedata::Format>, // uses native format if None
                        size: Size,
                        init_to_zero: bool) -> Option<ImageData> {
        use core::mem::transmute;
        let interface = ppb::get_image_data();
        let format = format.unwrap_or_else(|| {
            imagedata::native_image_data_format()
        }).to_ffi();
        let res = unsafe {
            interface.create(self.instance,
                             format,
                             transmute(size),
                             init_to_zero)
        };
        res.map(|res| ImageData::new(res) )
    }

    pub fn create_font(&self,
                       desc: &font::Description) -> Option<Font> {
        let f = ppb::get_font().Create.unwrap();
        let desc = unsafe { desc.to_ffi() };
        let res = f(self.instance, &desc as *const ffi::Struct_PP_FontDescription_Dev);
        if res != 0 {
            Some(Font::new(res))
        } else {
            None
        }
    }
}
pub trait InstanceCallbacks {
    fn on_destroy(&mut self) {}

    // You need to impl this function if you want to use OpenGL.
    // swap_buffers uses this to get back your callback's actual type.
    // This is an artifact of Rust's two pointer traits.
    fn on_buffers_swapped(&mut self, _code: Code) {
        fail!("you need to override/implement on_buffers_swapped");
    }

    fn on_change_view(&mut self, _view: View) {}
    fn on_change_focus(&mut self, _has_focus: bool) {}
    fn on_document_load(&mut self, _loader: UrlLoader) -> bool { false }
    fn on_message(&mut self, _message: AnyVar) {}
    fn on_kb_input(&mut self, _event: KeyboardInputEvent) -> bool { false }
    fn on_mouse_input(&mut self, _event: MouseInputEvent) -> bool { false }
    fn on_wheel_input(&mut self, _event: WheelInputEvent) -> bool { false }
    fn on_touch_input(&mut self, _event: TouchInputEvent) -> bool { false }
    fn on_ime_input(&mut self, _event: IMEInputEvent)     -> bool { false }
    fn on_graphics_context_lost(&mut self) {}
    fn on_mouse_lock_lost(&mut self) {}
}

struct InstanceStore {
    instance: Instance,
    mxt: Mutex,
    callbacks: Box<InstanceCallbacks>,
}
impl InstanceStore {
    fn new(inst: Instance, callbacks: Box<InstanceCallbacks>) -> InstanceStore {
        InstanceStore {
            instance: inst,
            mxt: Mutex::new(),
            callbacks: callbacks,
        }
    }

    fn on_buffers_swapped(&mut self, code: Code) {
        let _ = self.mxt.lock();
        self.callbacks.on_buffers_swapped(code)
    }

    fn on_destroy(&mut self) {
        let _ = self.mxt.lock();

        self.callbacks.on_destroy();
    }

    fn on_change_view(&mut self, view: View) {
        let _ = self.mxt.lock();

        self.callbacks.on_change_view(view.clone())
    }
    fn on_change_focus(&mut self, has_focus: bool) {
        let _ = self.mxt.lock();

        self.callbacks.on_change_focus(has_focus)
    }
    fn on_document_load(&mut self, loader: UrlLoader) -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_document_load(loader.clone())
    }
    fn on_message(&mut self, message: AnyVar) {
        let _ = self.mxt.lock();

        self.callbacks.on_message(message.clone())
    }
    fn on_kb_input(&mut self, event: KeyboardInputEvent) -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_kb_input(event.clone())
    }
    fn on_mouse_input(&mut self, event: MouseInputEvent) -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_mouse_input(event.clone())
    }
    fn on_wheel_input(&mut self, event: WheelInputEvent) -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_wheel_input(event.clone())
    }
    fn on_touch_input(&mut self, event: TouchInputEvent) -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_touch_input(event.clone())
    }
    fn on_ime_input(&mut self, event: IMEInputEvent)     -> bool {
        let _ = self.mxt.lock();

        self.callbacks.on_ime_input(event.clone())
    }
    fn on_graphics_context_lost(&mut self) {
        let _ = self.mxt.lock();

        self.callbacks.on_graphics_context_lost()
    }
    fn on_mouse_lock_lost(&mut self) {
        let _ = self.mxt.lock();

        self.callbacks.on_mouse_lock_lost()
    }
}
local_data_key!(current_instance: Instance)

type InstancesType = HashMap<Instance,
                             InstanceStore>;
static mut INSTANCES: *mut InstancesType = 0 as *mut InstancesType;

fn deinitialize_instances() {
    unsafe {
        if !INSTANCES.is_null() {
            let instances = ptr::read_and_zero(INSTANCES);
            drop(instances);
        }
    }
}

fn expect_instances() -> &'static mut InstancesType {
    use std::hash::RandomSipHasher;
    use core::mem;
    use alloc::libc_heap::malloc_raw;
    unsafe {
        if INSTANCES.is_null() {
            //let crypto = ppb::get_crypto();
            //let rand_bytes = crypto.GetRandomBytes.unwrap();
            //let mut rand_buf: [u64, ..2] = [0u64, 0u64];
            //rand_bytes(rand_buf.as_mut_ptr() as *mut i8, 16);
            //let hasher = SipHasher::new_with_keys(rand_buf[0], rand_buf[1]);
            let hasher = RandomSipHasher::new();
            let instances: InstancesType = HashMap::with_hasher(hasher);
            INSTANCES = malloc_raw(mem::size_of::<InstancesType>())
                as *mut InstancesType;
            if INSTANCES.is_null() {
                // PANIC!
                fail!("couldn't allocate instances map!");
            }
            ptr::write(mem::transmute(INSTANCES),
                       instances);
            expect_instances()
        } else {
            mem::transmute(INSTANCES)
        }
    }
}

fn find_instance<U, Take>(instance: Instance,
                          take: Take,
                          f: |&mut InstanceStore, Take| -> U) -> Option<U> {
    match expect_instances().find_mut(&instance) {
        Some(inst) => Some(f(inst, take)),
        None => {
            // TODO: better message/moar infos.
            error!("Instance not found");
            None
        },
    }
}

pub mod entry {
    use super::{expect_instances, find_instance};
    use super::{InstanceCallbacks, InstanceStore, Instance};
    use super::AnyVar;
    use super::{View, UrlLoader};
    use super::ToFFIBool;
    use libc::c_char;
    use std::any::Any;
    use std::result;
    use TaskResult = std::rt::task::Result;
    use std::rt::local::{Local};

    use super::ffi;

    // We need to catch all failures in our callbacks,
    // lest an exception (failure) in one instance terminates all
    // instances and crashes the whole plugin.
    pub fn try_block(f: ||) -> TaskResult {
        use rustrt::unwind::try;
        let result = unsafe {
            try(f)
        };
        // if we're unwinding, the instance had a failure, and we need
        // to destory the instance.
        // Note that this can be called before an instance is ever inserted
        // into the global store.
        if result.is_err() {
            match Instance::opt_current() {
                Some(inst) => { expect_instances().pop(&inst); }
                _ => {}
            }
        }
        result
    }
    pub fn try_block_with_ret<U>(f: || -> U) -> result::Result<U, Box<Any + Send>> {
        let mut ret: Option<U> = None;
        try_block(|| {
            ret = Some(f());
        }).map(|()| ret.take().unwrap() )
    }
    
    pub extern "C" fn did_create(inst: ffi::PP_Instance,
                                 argc: u32,
                                 argk: *mut *const c_char,
                                 argv: *mut *const c_char) -> ffi::PP_Bool {
        let instance = Instance::new(inst);
        instance.set_current();
        try_block(|| {
            instance.initialize_nacl_io();

            let callbacks = unsafe {
                super::ppapi_instance_created(instance.clone(),
                                              || super::parse_args(argc, argk, argv) )
            };

            if !expect_instances().insert(instance, InstanceStore::new(instance, callbacks)) {
                fail!("instance already created?");
            }
        }).is_ok().to_ffi_bool()
    }
    pub extern "C" fn did_destroy(inst: ffi::PP_Instance) {
        let instance = Instance::new(inst);
        instance.set_current();
        let _ = try_block(|| {
            debug!("did_destroy");
            
            find_instance(instance, (), |store, ()| store.on_destroy() );
            
            expect_instances().pop(&instance);
        });
    }
    pub extern "C" fn did_change_view(inst: ffi::PP_Instance, view: ffi::PP_Resource) {
        let instance = Instance::new(inst);
        instance.set_current();
        let _ = try_block(|| {
            debug!("did_change_view");

            find_instance(instance, (), |store, ()| store.on_change_view(View::new(view)));
        });
    }
    pub extern "C" fn did_change_focus(inst: ffi::PP_Instance, has_focus: ffi::PP_Bool) {
        let instance = Instance::new(inst);
        instance.set_current();
        let _ = try_block(|| {
            debug!("did_change_focus");
            find_instance(instance,
                          (),
                          |store, ()| store.on_change_focus(has_focus != ffi::PP_FALSE) );
        });
    }
    pub extern "C" fn handle_document_load(inst: ffi::PP_Instance, 
                                           url_loader: ffi::PP_Resource) -> ffi::PP_Bool {
        let instance = Instance::new(inst);
        instance.set_current();
        let handled = try_block_with_ret(|| {
            debug!("handle_document_load");

            find_instance(instance, (), |store, ()| {
                store.on_document_load(UrlLoader::new(url_loader))
            }).unwrap_or(false)
        }).ok().unwrap_or(false);
        return handled as ffi::PP_Bool;
    }

    pub extern "C" fn handle_message(inst: ffi::PP_Instance, msg: ffi::PP_Var) {
        let instance = Instance::new(inst);
        instance.set_current();
        let _ = try_block(|| {
            debug!("handle_message");

            find_instance(instance, (), |store, ()| {
                store.on_message(AnyVar::new_bumped(msg))
            });
        });
    }
    pub extern "C" fn handle_input_event(inst: ffi::PP_Instance,
                                         event: ffi::PP_Resource) -> ffi::PP_Bool {
        use super::ppb;
        use super::{MouseInputEvent, KeyboardInputEvent, WheelInputEvent,
                    TouchInputEvent, IMEInputEvent};
        let instance = Instance::new(inst);
        instance.set_current();
        let handled = try_block_with_ret(|| {
            let kbe = ppb::get_keyboard_event().IsKeyboardInputEvent.unwrap();
            let me  = ppb::get_mouse_event().IsMouseInputEvent.unwrap();
            let we  = ppb::get_wheel_event().IsWheelInputEvent.unwrap();
            let te  = ppb::get_touch_event().IsTouchInputEvent.unwrap();
            let ime = ppb::get_ime_event().IsIMEInputEvent.unwrap();

            let f = if me(event) != 0 {
                |inst: &mut InstanceStore, event: ffi::PP_Resource| -> bool {
                    let e = MouseInputEvent(event);
                    inst.on_mouse_input(e)
                }
            } else if kbe(event) != 0 {
                |inst: &mut InstanceStore, event: ffi::PP_Resource| -> bool {
                    let e = KeyboardInputEvent(event);
                    inst.on_kb_input(e)
                }
            } else if we(event) != 0 {
                |inst: &mut InstanceStore, event: ffi::PP_Resource| -> bool {
                    let e = WheelInputEvent(event);
                    inst.on_wheel_input(e)
                }
            } else if te(event) != 0 {
                |inst: &mut InstanceStore, event: ffi::PP_Resource| -> bool {
                    let e = TouchInputEvent(event);
                    inst.on_touch_input(e)
                }
            } else if ime(event) != 0 {
                |inst: &mut InstanceStore, event: ffi::PP_Resource| -> bool {
                    let e = IMEInputEvent::new(event);
                    inst.on_ime_input(e)
                }
            } else {
                error!("unknown input event");
                return false;
            };
            find_instance(instance, event, f).unwrap_or(false)
        }).ok().unwrap_or(false);
        handled.to_ffi_bool()
    }
    pub extern "C" fn graphics_context_lost(inst: ffi::PP_Instance) {
        let instance = Instance::new(inst);
        instance.set_current();
        let _ = try_block(|| {
            debug!("graphics_context_lost");
            find_instance(instance, (), |store, ()| {
                store.on_graphics_context_lost()
            });
        });
    }
}

extern {
    #[no_mangle]
    fn ppapi_instance_created(instance: Instance,
                              args: || -> HashMap<String, String>)
                              -> Box<InstanceCallbacks>;
}

// The true entry point of any module.
#[no_mangle]
#[inline(never)]
#[allow(non_snake_case_functions)]
pub extern "C" fn PPP_InitializeModule(modu: ffi::PP_Module,
                                       gbi: ffi::PPB_GetInterface) -> libc::int32_t {
    use log::set_logger;
    use std::rt;
    use std::str::Slice;
    use std::io::{LineBufferedWriter, Writer};
    use std::io::stdio::{set_stderr, set_stdout};
    use std::rt::local::{Local};
    use self::entry::try_block;
    use libc::{STDOUT_FILENO, STDERR_FILENO};

    static MAIN_TASK_NAME: &'static str = "main module task";

    rt::init(0, ptr::null());
    {
        // for now, stack bounds don't matter.
        let mut task = native::task::new((0, 0));
        task.name = Some(Slice(MAIN_TASK_NAME));
        Local::put(task);
    }

    let stdout = CurrentInstanceStdIo {
        level: ffi::PP_LOGLEVEL_LOG,
        fd:    STDOUT_FILENO,
        buffer: Vec::new(),
    };
    let stderr = CurrentInstanceStdIo {
        level: ffi::PP_LOGLEVEL_ERROR,
        fd:    STDERR_FILENO,
        buffer: Vec::new(),
    };
    set_stdout(box stdout as Box<Writer + Send>);
    set_stderr(box stderr as Box<Writer + Send>);
    set_logger(box CurrentInstanceLogger);

    // We can't fail! before this point!
    let initialized = try_block(|| {
        pp::initialize_globals(modu);
        ppb::initialize_globals(gbi);
    }).is_ok();

    if initialized {
        ffi::PP_OK
    } else {
        1i32
    }
}
#[no_mangle]
#[allow(non_snake_case_functions)]
pub extern "C" fn PPP_ShutdownModule() {
    use std::rt::local::{Local};
    use self::entry::try_block;
    use std::rt::task::Task;
    // FIXME
    let _ = try_block(|| {
        deinitialize_instances();
    });
    let _: Box<Task> = Local::take();
}
