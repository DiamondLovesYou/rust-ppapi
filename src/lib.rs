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

/*! Rust idiomatic wrapper for the Pepper API.

To use, you will need to implement at least these two functions:

```rust
#![no_main]
#[no_managle]
pub extern fn ppapi_instance_created(instance: Instance,
                                     args: HashMap<String, String>) {
}
#[no_managle]
pub extern fn ppapi_instance_destroyed() {
}
```

All instances will be created from a new task/thread. The task takes its
name from the id attribute on the embed object used to create the
instance. Failing will cause rust-ppapi to cleanup the task, though the
plugin will continue to run. Currently there is no way to automatically
restart an instance if it fails.

The other callbacks consist of:

```rust
#[no_mangle]
pub extern fn ppapi_on_document_loaded(loader: ppapi::UrlLoader) -> bool {
}
#[no_mangle]
pub extern fn ppapi_on_change_view(view: ppapi::View) {
}
#[no_mangle]
pub extern fn ppapi_on_change_focus(has_focus: bool) {
}
#[no_mangle]
pub extern fn ppapi_on_message(msg: ppapi::AnyVar) {
}
#[no_mangle]
pub extern fn ppapi_on_input(event: ppapi::input::Class) -> bool {
}
#[no_mangle]
pub extern fn ppapi_on_graphics_context_lost() {
}
```

These are all optional. If implemented, they will be called from the instance's task.

More info:

 * [ppapi_instance_created](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___instance__1__1.html#aff2dd72f7aab6335cacf8bc3a66ccbba)
 * [ppapi_instance_destroyed](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___instance__1__1.html#a99edbb91abde255fec3bc3e1f9c8ba82)
 * [ppapi_on_document_loaded](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___instance__1__1.html#a2fba2c9d06044a48e73a649b04398e1d)
 * [ppapi_on_change_view](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___instance__1__1.html#aa028a7b17d62242ac56b6ab4b55dc047)
 * [ppapi_on_change_focus](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___instance__1__1.html#abf4a481156b605938416bf873bd2c782)
 * [ppapi_on_message](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___messaging__1__0.html#a558ca784cf11eaba479ff8621ae2c507)
 * [ppapi_on_input](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___input_event__0__1.html#ae684a39a2bf6b58aee0f7420aab43150)
 * [ppapi_on_graphics_context_lost](https://developer.chrome.com/native-client/pepper_stable/c/struct_p_p_p___graphics3_d__1__0.html#ae7aba86d10d1b8c4c7a41bac3af64b0a)

*/

#![crate_name = "ppapi"]
#![crate_type = "rlib"]
#![experimental]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(default_type_params, struct_variant)]
#![feature(linkage)]
#![feature(thread_local)]
//#![warn(missing_doc)]

#![allow(dead_code)]

extern crate native;
#[phase(plugin, link)]
extern crate log;
extern crate collections;
extern crate sync;
extern crate rand;
extern crate serialize;
extern crate http;
extern crate "url" as iurl;
extern crate libc;
extern crate core;
extern crate alloc;
extern crate rustrt;

use std::{mem, cmp, io, hash, num};
use std::mem::transmute;
use std::intrinsics;
use std::ptr;
use std::ops;
use std::rt::task;
use std::iter;
use std::clone;
use std::{str, string};
use std::result;
use std::collections::hashmap::HashMap;
use std::fmt;

use log::LogRecord;

use ppb::{get_url_loader, get_url_request, get_url_response};
use ppb::{ViewIf, MessageLoopIf, VarIf, ImageDataIf, URLLoaderIf,
          URLRequestInfoIf};

use font::Font;
use gles::Context3d;
use input::{KeyboardInputEvent,
            MouseInputEvent,
            WheelInputEvent,
            TouchInputEvent,
            IMEInputEvent};
use imagedata::ImageData;
use url::{UrlLoader, UrlRequestInfo, UrlResponseInfo};

macro_rules! impl_resource_for(
    ($ty:ty $type_:ident) => (
        impl Resource for $ty {
            #[inline]
            fn unwrap(&self) -> ::ffi::PP_Resource {
                unsafe { ::std::mem::transmute_copy(self) }
            }
            #[inline]
            fn type_of(&self) -> ::ResourceType {
                ::$type_
            }
        }
        impl $ty {
            pub fn new(res: ::ffi::PP_Resource) -> $ty {
                unsafe {
                    ::std::mem::transmute_copy(&res)
                }
            }
            pub fn new_bumped(res: ::ffi::PP_Resource) -> $ty {
                let v: $ty = unsafe { ::std::mem::transmute_copy(&res) };
                // bump the ref count:
                unsafe { ::std::mem::forget(v.clone()) };
                v
            }
        }
        impl ::ToOption<::ffi::PP_Resource> for $ty {
            fn to_option(from: &::ffi::PP_Resource) -> Option<$ty> {
                if *from == 0 {
                    None
                } else {
                    Some(unsafe {
                        ::std::mem::transmute_copy(from)
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
                println!("add ref: `{}`", self);
                (ppb::get_core().AddRefResource.unwrap())(self.unwrap());
                unsafe {
                    mem::transmute_copy(self)
                }
            }
        }
        impl Drop for $ty {
            fn drop(&mut self) {
                println!("drop ref: `{}`", self);
                (ppb::get_core().ReleaseResource.unwrap())(self.unwrap());
            }
        }
    )
)

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
            fail!("Code: `{code:s}`, Message: `{msg:s}`",
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

impl fmt::Show for ffi::Struct_PP_FloatPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
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
#[deriving(Hash, Eq, PartialEq, Show)] pub struct Context2d(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct View(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct MessageLoop(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct FileRef(ffi::PP_Resource);
#[deriving(Clone, Hash, Eq, PartialEq, Show)]
pub struct FileSliceRef(FileRef,
                        Option<i64>,
                        Option<i64>);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct FileIo(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct Filesystem(ffi::PP_Resource);

impl_clone_drop_for!(Context3d)
impl_resource_for!(Context2d Graphics2DRes)
impl_clone_drop_for!(Context2d)
impl_resource_for!(View ViewRes)
impl_clone_drop_for!(View)
impl_resource_for!(MessageLoop MessageLoopRes)
impl_clone_drop_for!(MessageLoop)
impl_clone_drop_for!(KeyboardInputEvent)
impl_clone_drop_for!(MouseInputEvent)
impl_clone_drop_for!(WheelInputEvent)
impl_clone_drop_for!(TouchInputEvent)
impl_clone_drop_for!(Font)
impl_clone_drop_for!(ImageData)
impl_resource_for!(FileRef FileRefRes)
impl_clone_drop_for!(FileRef)
impl_resource_for!(FileIo FileIoRes)
impl_clone_drop_for!(FileIo)
impl_resource_for!(Filesystem FilesystemRes)
impl_clone_drop_for!(Filesystem)
impl_clone_drop_for!(IMEInputEvent)
impl_clone_drop_for!(UrlLoader)
impl_clone_drop_for!(UrlRequestInfo)
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
impl Messaging {
    pub fn post_message<T: ToVar>(&self, message: T) {
        use ppb::MessagingIf;
        ppb::get_messaging().post_message(self.unwrap(), message.to_var())
    }
}
impl MessageLoop {
    pub fn get_main_loop() -> MessageLoop {
        MessageLoop::new((ppb::get_message_loop().GetForMainThread.unwrap())())
    }
    pub fn is_attached() -> bool {
        unsafe {
            (ppb::get_message_loop().GetCurrent.unwrap())() != mem::transmute(0i32)
        }
    }
    pub fn current() -> Option<MessageLoop> {
        ppb::get_message_loop()
            .get_current()
            .map(|current| MessageLoop::new(current) )
    }
    pub fn attach_to_current_thread(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().AttachToCurrentThread.unwrap())(self.unwrap()))
    }
    /// Blocking
    pub fn run_loop(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().Run.unwrap())(self.unwrap()))
    }
    pub fn post_work<T: Callback>(&self, work: T, delay: i64) -> Code {
        let comp_cb = work.to_ffi_callback();
        match ppb::get_message_loop().post_work(&self.unwrap(), comp_cb, delay) {
            ffi::PP_ERROR_BADARGUMENT => fail!("internal error: completion callback was null?"),
            c => Code::from_i32(c),
        }
    }
    pub fn post_to_self<T: Callback>(work: T, delay: i64) -> Code {
        MessageLoop::current()
            .expect("can't post work to self: no message loop attached to the current thread")
            .post_work(work, delay)
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
        ppb::get_var().add_ref(self);
        unsafe {
            mem::transmute_copy(self)
        }
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
            string::raw::from_buf_len(buf as *const u8, len as uint)
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

#[deriving(Clone, Eq, PartialEq)]
pub struct Messaging(ffi::PP_Instance);
impl Messaging {
    fn unwrap(&self) -> ffi::PP_Instance {
        let &Messaging(inst) = self;
        inst
    }
}

#[deriving(Clone, Eq, PartialEq)]
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
            args.swap(string::raw::from_buf(*argk.offset(i) as *const u8),
                      string::raw::from_buf(*argv.offset(i) as *const u8));
        }
    }
    return args;
}

trait Callback {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback;
}
trait PostToSelf {
    fn post_to_self(self, code: Code);
}
impl PostToSelf for ffi::Struct_PP_CompletionCallback {
    fn post_to_self(self, code: Code) {
        // Used because we specifically don't want to take the callbacks in local data.
        struct RunCompletionCallback(proc(): 'static);
        impl Callback for RunCompletionCallback {
            fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
                extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
                    let code = Code::from_i32(status);

                    if possibly_warn_code_callback(code) { return; }

                    let work: Box<proc()> = unsafe { mem::transmute(user) };
                    // Nb no try_block here.
                    (*work)()
                }
                let RunCompletionCallback(work) = self;
                unsafe {
                    ffi::make_completion_callback(work_callback,
                                                  mem::transmute(box work))
                }
            }
        }

        MessageLoop::post_to_self(RunCompletionCallback(proc() {
            unsafe {
                ffi::run_completion_callback(self,
                                             code.to_i32())
            }
        }), 0);
    }
}
fn possibly_warn_code_callback(code: Code) -> bool {
    if code != Ok {
        warn!("unhandled code in callback: `{}`", code);
        true
    } else {
        false
    }
}
// TODO: use std::ops::FnOnce.

fn if_error_shutdown_msg_loop(result: task::Result) {
    if result.is_err() {
        let _ = MessageLoop::current().unwrap().shutdown();
    }
}

impl Callback for proc():Send {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let code = Code::from_i32(status);

            if possibly_warn_code_callback(code) { return; }

            let work: Box<proc():Send> = unsafe { mem::transmute(user) };
            (*work)();
        }
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(box self))
        }
    }
}
impl Callback for proc(Code):Send {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let work: Box<proc(Code):Send> = unsafe { mem::transmute(user) };
            let code = Code::from_i32(status);
            (*work)(code);
        }
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(box self))
        }
    }
}
impl Callback for fn() {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let code = Code::from_i32(status);

            if possibly_warn_code_callback(code) { return; }

            let work: fn() = unsafe { mem::transmute(user) };
            work()
        }
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(self))
        }
    }
}
impl Callback for fn(Code) {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let work: fn(Code) = unsafe { mem::transmute(user) };
            let code = Code::from_i32(status);
            work(code);
        }
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(self))
        }
    }
}

struct InternalCallbacksOperatorProc<'a>(proc(): 'a);
impl<'a> Callback for InternalCallbacksOperatorProc<'a> {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let code = Code::from_i32(status);

            if possibly_warn_code_callback(code) { return; }

            let work: Box<proc()> = unsafe {
                mem::transmute(user)
            };
            (*work)();
        }
        let InternalCallbacksOperatorProc(work) = self;
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(box work))
        }
    }
}
struct InternalCallbacksOperatorFn(fn());
impl Callback for InternalCallbacksOperatorFn {
    fn to_ffi_callback(self) -> ffi::Struct_PP_CompletionCallback {
        extern "C" fn work_callback(user: *mut libc::c_void, status: i32) {
            let code = Code::from_i32(status);

            if possibly_warn_code_callback(code) { return; }

            let work: fn() = unsafe {
                mem::transmute(user)
            };
            work();
        }
        let InternalCallbacksOperatorFn(work) = self;
        unsafe {
            ffi::make_completion_callback(work_callback,
                                          mem::transmute(work))
        }
    }
}

struct ConsoleLogger {
    console: Console,
}
impl ConsoleLogger {
    fn new(instance: &Instance) -> ConsoleLogger {
        ConsoleLogger {
            console: instance.console(),
        }
    }
}
impl log::Logger for ConsoleLogger {
    fn log(&mut self, record: &LogRecord) {
        use self::ppb::ConsoleInterface;
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
                          record.args)
            .to_string_var();
        self.console.log(level,
                         str);
    }
}
struct StdIo {
    level: ffi::PP_LogLevel,
    raw: io::stdio::StdWriter,
    console: Option<Console>,
    buffer: Vec<u8>,
}
impl Writer for StdIo {
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
            let rest = buf.slice(0, newline_pos + 1);
            self.buffer.push_all(rest);
            let result = (|| {
                use std::result::{Ok, Err};
                use self::ppb::ConsoleInterface;
                let console = self.console.or_else(|| {
                    Instance::opt_current()
                        .map(|i| i.console() )
                });

                try!(self.raw.write(self.buffer.as_slice()));

                str::from_utf8(self.buffer.slice_to(self.buffer.len() - 1))
                    .and_then(|s| console.map(|c| (c, s) ) )
                    .map(|(c, s)| {
                        c.log(self.level, s)
                    });
                Ok(())
            })();
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

local_data_key!(current_instance: Instance)
static mut first_instance: Option<Instance> = None;

pub fn is_main_thread() -> bool {
    Some(MessageLoop::get_main_loop()) == MessageLoop::current()
}

#[deriving(Clone, Hash, Eq, PartialEq)]
pub struct Instance {
    instance: ffi::PP_Instance,
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
    fn check_current(&self) {
        assert!(Instance::current() == *self);
    }
    fn assert_unset_current() {
        assert!(Instance::opt_current().is_none());
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

    pub fn messaging(&self) -> Messaging {
        return Messaging(self.instance);
    }

    pub fn create_3d_context(&self,
                             share_with: Option<Context3d>,
                             attribs: &[(u32, u32)]) -> result::Result<Context3d, Code> {
        let mut a = Vec::with_capacity(attribs.len() * 2 + 1);
        for &(k, v) in attribs.iter() {
            a.push(k);
            a.push(v);
        }
        a.push(ffi::PP_GRAPHICS3DATTRIB_NONE);
        let a = a;
        let share_with = share_with
            .map(|ctxt| {
                ctxt.unwrap()
            })
            .unwrap_or_else(|| 0i32 );

        let graphics = ppb::get_graphics_3d();

        let raw_cxt  = (graphics.Create.unwrap())(self.instance,
                                                  share_with,
                                                  a.as_ptr() as *const i32);

        if raw_cxt == 0i32 {
            result::Err(Failed)
        } else {
            result::Ok(Context3d::new_bumped(raw_cxt))
        }
    }
    pub fn bind_context<T: ContextResource>(&self, cxt: &T) -> Code {
        match (ppb::get_instance().BindGraphics.unwrap())
            (self.instance,
             cxt.get_device()) {
            ffi::PP_TRUE => Ok,
            ffi::PP_FALSE => Failed,
            other => {
                error!("unknown truthy value: {:}", other);
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
        use std::mem::transmute;
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

    pub fn create_msg_loop(&self) -> MessageLoop {
        MessageLoop(ppb::get_message_loop().create(&self.unwrap()))
    }

    pub fn create_url_loader(&self) -> Option<UrlLoader> {
        get_url_loader().create(self.unwrap()).map(|loader| UrlLoader::new(loader) )
    }
    fn create_url_request_info(&self) -> Option<UrlRequestInfo> {
        get_url_request().create(self.unwrap()).map(|info| UrlRequestInfo::new(info) )
    }
}

impl MessageLoop {
    fn get_ref<'a>(&'a self) -> &'a MessageLoop {
        self
    }

    fn on_destroy(&self) {
        fn work() {
            unsafe {
                ppapi_instance_destroyed();
            }
        }
        self.get_ref()
            .post_work(InternalCallbacksOperatorFn(work),
                       0)
            .expect("couldn't tell an instance to shutdown");
        self.get_ref().shutdown().expect("message loop shutdown failed");
    }

    fn on_change_view(&mut self, view: View) {
        self.get_ref()
            .post_work(InternalCallbacksOperatorProc(proc() {
                unsafe {
                    assert!(!ppapi_on_change_view.is_null());
                    let on_change_view: fn(View) =
                        transmute(ppapi_on_change_view);
                    on_change_view(view);
                }
            }),
                       0)
            .expect("couldn't tell an instance about an on_change_view event");
    }
    fn on_change_focus(&mut self, has_focus: bool) {
        self.get_ref()
            .post_work(InternalCallbacksOperatorProc(proc() {
                unsafe {
                    assert!(!ppapi_on_change_focus.is_null());
                    let on_change_focus: fn(bool) =
                        transmute(ppapi_on_change_focus);
                    on_change_focus(has_focus);
                }
            }),
                       0)
            .expect("couldn't tell an instance about an on_change_focus event");
    }
    fn on_document_load(&mut self, loader: UrlLoader) -> bool {

        // TODO: THIS IS MASSIVELY UNSAFE.

        let (tx, rx) = channel();
        self.get_ref()
            .post_work(InternalCallbacksOperatorProc(proc() {
                unsafe {
                    assert!(!ppapi_on_document_loaded.is_null());
                    let on_document_loaded: fn(UrlLoader) -> bool =
                        transmute(ppapi_on_document_loaded);

                    let handled = on_document_loaded(loader);
                    tx.send(handled);
                }
            }),
                       0)
            .expect("couldn't tell an instance about an on_change_view event");
        rx.recv_opt().unwrap_or(false)
    }
}

type InstancesType = HashMap<Instance,
                             MessageLoop>;

// THIS MAY ONLY BE ACCESSED FROM THE MAIN MODULE THREAD.
static mut INSTANCES: *mut InstancesType = 0 as *mut InstancesType;

unsafe fn deinitialize_instances() {
    if !INSTANCES.is_null() {
        let instances = ptr::read_and_zero(INSTANCES);
        drop(instances);
    }
}

fn expect_instances() -> &'static mut InstancesType {
    use std::hash::RandomSipHasher;
    use core::mem;
    use alloc::libc_heap::malloc_raw;
    unsafe {
        if INSTANCES.is_null() {
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
                          f: |&mut MessageLoop, Take| -> U) -> Option<U> {
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
    use super::{Instance, Callback};
    use super::{AnyVar, Ok};
    use super::{View};
    use super::ToFFIBool;
    use super::{ffi};
    use super::url::UrlLoader;

    use libc::c_char;
    use std::any::Any;
    use std::finally::try_finally;
    use std::mem::transmute;
    use std::result;
    use std::rt::local::{Local};
    use std::rt::task::{Task, Result};
    use rustrt::unwind::try;

    // We need to catch all failures in our callbacks,
    // lest an exception (failure) in one instance terminates all
    // instances and crashes the whole plugin.
    pub fn try_block(f: ||) -> Result {
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
        use log::set_logger;
        use std::rt::task::TaskOpts;
        use std::str::Owned;
        use std::io;
        use std::io::{Writer};
        use std::io::stdio::{set_stderr, set_stdout};
        use std::task::Spawner;
        use native;
        use super::{StdIo, ConsoleLogger, MessageLoop};

        let instance = Instance::new(inst);
        Instance::assert_unset_current();
        instance.set_current();

        let logger = ConsoleLogger::new(&instance);
        // TODO: I think this should be set only for the first created instance,
        // not all of them.
        set_logger(box logger);

        let mut success = false;
        let _ = try_block(|| {
            instance.initialize_nacl_io();

            let args = super::parse_args(argc, argk, argv);
            let mut ops = TaskOpts::new();
            ops.name = args.find_copy(&"id".to_string())
                .map(|id| {
                    Owned(id)
                });

            let (tx, rx) = channel();

            let spawner = native::task::NativeSpawner;
            spawner.spawn(ops, proc() {
                instance.set_current();
                let console = instance.console();
                let stdout = StdIo {
                    level:   ffi::PP_LOGLEVEL_LOG,
                    raw:     io::stdio::stdout_raw(),
                    console: Some(console.clone()),
                    buffer:  Vec::new(),
                };
                let stderr = StdIo {
                    level:   ffi::PP_LOGLEVEL_ERROR,
                    raw:     io::stdio::stderr_raw(),
                    console: Some(console.clone()),
                    buffer:  Vec::new(),
                };
                let logger = ConsoleLogger::new(&instance);
                set_stdout(box stdout as Box<Writer + Send>);
                set_stderr(box stderr as Box<Writer + Send>);
                set_logger(box logger);

                let ml = instance.create_msg_loop();
                match ml.attach_to_current_thread() {
                    Ok => {}
                    _ => {
                        error!("failed to attach the new instance's message loop");
                        tx.send(None);
                        return;
                    }
                }

                fn unwinding() -> bool {
                    Local::borrow(None::<Task>).unwinder.unwinding()
                }

                try_finally(&mut (), args,
                            |_, args| unsafe {
                                super::ppapi_instance_created(instance.clone(), args)
                            },
                            |_| {
                                if unwinding() {
                                    error!("failed to initialize instance");
                                    tx.send(None);
                                } else {
                                    tx.send(Some(ml.clone()));
                                }
                            });

                let code = try_finally(&mut (), ml.clone(),
                                       |_, ml| ml.run_loop(),
                                       |_| {
                                           if unwinding() {
                                               let _ = ml.shutdown();
                                           }
                                       });
                if code != Ok {
                    fail!("message loop exited with code: `{}`", code);
                }
                if MessageLoop::is_attached() {
                    fail!("please shutdown the loop; I may add pausing for some sort of pattern later");
                } else {
                    MessageLoop::get_main_loop()
                        .post_work(proc() {
                            super::expect_instances()
                                .pop(&instance);
                        }, 0);
                }
            });

            success = rx.recv()
                .map(|ml| {
                    if !expect_instances().insert(instance, ml.clone()) {
                        error!("instance already exists");
                        ml.on_destroy();
                        false
                    } else {
                        true
                    }
                })
                .unwrap_or(false)
        });
        Instance::unset_current();
        success.to_ffi_bool()
    }
    pub extern "C" fn did_destroy(inst: ffi::PP_Instance) {
        let instance = Instance::new(inst);
        Instance::assert_unset_current();
        instance.set_current();

        let _ = try_block(|| {
            debug!("did_destroy");

            find_instance(instance, (), |store, ()| store.on_destroy() );

            expect_instances().pop(&instance);
        });

        Instance::unset_current();
    }
    pub extern "C" fn did_change_view(inst: ffi::PP_Instance, view: ffi::PP_Resource) {
        let instance = Instance::new(inst);
        Instance::assert_unset_current();
        instance.set_current();

        if !super::ppapi_on_change_view.is_null() {
            let _ = try_block(|| {
                debug!("did_change_view");
                find_instance(instance,
                              view,
                              |store, view| {
                                  let view = View::new_bumped(view);
                                  store.on_change_view(view)
                              });
            });
        } else {
            warn!("plugin is missing 'ppapi_on_change_view'");
        }

        Instance::unset_current();
    }
    pub extern "C" fn did_change_focus(inst: ffi::PP_Instance, has_focus: ffi::PP_Bool) {
        let instance = Instance::new(inst);
        Instance::assert_unset_current();
        instance.set_current();

        if !super::ppapi_on_change_focus.is_null() {
            let _ = try_block(|| {
                debug!("did_change_focus");

                find_instance(instance,
                              (),
                              |store, ()| store.on_change_focus(has_focus != ffi::PP_FALSE) );
            });
        } else {
            warn!("plugin is missing 'ppapi_on_change_focus'");
        }

        Instance::unset_current();
    }
    pub extern "C" fn handle_document_load(inst: ffi::PP_Instance,
                                           url_loader: ffi::PP_Resource) -> ffi::PP_Bool {
        let instance = Instance::new(inst);
        Instance::assert_unset_current();
        instance.set_current();

        if super::ppapi_on_document_loaded.is_null() {
            warn!("plugin is missing 'ppapi_on_document_loaded'");
            return false.to_ffi_bool();
        }

        let handled = try_block_with_ret(|| {
            debug!("handle_document_load");

            find_instance(instance,
                          UrlLoader::new_bumped(url_loader),
                          |store, url_loader| {
                              store.on_document_load(url_loader)
                          }).unwrap_or(false)
        }).ok().unwrap_or(false);

        Instance::unset_current();

        return handled.to_ffi_bool();
    }

    pub extern "C" fn handle_message(inst: ffi::PP_Instance, msg: ffi::PP_Var) {
        let instance = Instance::new(inst);
        instance.check_current();

        if super::ppapi_on_message.is_null() {
            warn!("plugin is missing 'ppapi_on_message'");
            return;
        }

        debug!("handle_message");
        unsafe {
            let on_message: fn(AnyVar) = transmute(super::ppapi_on_message);
            on_message(AnyVar::new_bumped(msg));
        }
    }


    // this is called from the instance's thread, not from main.
    pub extern "C" fn handle_input_event(inst: ffi::PP_Instance,
                                         event: ffi::PP_Resource) -> ffi::PP_Bool {
        use super::{ppb, ppapi_on_input};
        use input::{MouseInputEvent, KeyboardInputEvent, WheelInputEvent,
                    TouchInputEvent, IMEInputEvent};
        use input::Class;
        let instance = Instance::new(inst);
        instance.check_current();

        if ppapi_on_input.is_null() {
            warn!("plugin requested input events, but didn't implement \
                   'ppapi_on_input'");
            return false.to_ffi_bool();
        }

        let mut handled;
        unsafe {
            let kbe = ppb::get_keyboard_event().IsKeyboardInputEvent.unwrap();
            let me  = ppb::get_mouse_event().IsMouseInputEvent.unwrap();
            let we  = ppb::get_wheel_event().IsWheelInputEvent.unwrap();
            let te  = ppb::get_touch_event().IsTouchInputEvent.unwrap();
            let ime = ppb::get_ime_event().IsIMEInputEvent.unwrap();

            let e = if me(event) != 0 {
                Class::new(MouseInputEvent::new(event))
            } else if kbe(event) != 0 {
                Class::new(KeyboardInputEvent::new(event))
            } else if we(event) != 0 {
                Class::new(WheelInputEvent::new(event))
            } else if te(event) != 0 {
                Class::new(TouchInputEvent::new(event))
            } else if ime(event) != 0 {
                Class::new(IMEInputEvent::new(event))
            } else {
                fail!("unknown input event");
            };
            let on_input: fn(Class) -> bool =
                transmute(ppapi_on_input);
            handled = Some(on_input(e));
        }

        handled.unwrap_or(false).to_ffi_bool()
    }
    pub extern "C" fn graphics_context_lost(inst: ffi::PP_Instance) {
        let instance = Instance::new(inst);
        instance.check_current();

        if super::ppapi_on_graphics_context_lost.is_null() {
            warn!("plugin is missing 'ppapi_on_graphics_context_lost'");
            return;
        }

        unsafe {
            let on: fn() = transmute(super::ppapi_on_graphics_context_lost);
            on();
        }
    }
}

#[allow(ctypes)]
extern {
    #[no_mangle]
    fn ppapi_instance_created(instance: Instance,
                              args: HashMap<String, String>);
    #[no_mangle]
    fn ppapi_instance_destroyed();

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_document_loaded: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_change_view: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_change_focus: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_message: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_input: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_graphics_context_lost: *const libc::c_void;
}

#[no_mangle]
#[allow(non_snake_case)]
// The true entry point of any module. DO NOT CALL THIS YOURSELF. It is used by Pepper.
pub extern "C" fn PPP_InitializeModule(modu: ffi::PP_Module,
                                       gbi: ffi::PPB_GetInterface) -> libc::int32_t {
    use std::io::stdio::{set_stderr, set_stdout, stdout_raw, stderr_raw};
    use std::str::Slice;
    use std::rt;
    use std::rt::local::{Local};
    use self::entry::try_block;

    static MAIN_TASK_NAME: &'static str = "main module task";

    rt::init(0, ptr::null());
    {
        // for now, stack bounds don't matter.
        let mut task = native::task::new((0, 0));
        task.name = Some(Slice(MAIN_TASK_NAME));
        Local::put(task);
    }

    let stdout = StdIo {
        level:   ffi::PP_LOGLEVEL_LOG,
        raw:     stdout_raw(),
        console: None,
        buffer:  Vec::new(),
    };
    let stderr = StdIo {
        level:   ffi::PP_LOGLEVEL_ERROR,
        raw:     stderr_raw(),
        console: None,
        buffer:  Vec::new(),
    };
    set_stdout(box stdout);
    set_stderr(box stderr);

    // We can't fail! before this block!
    let result = try_block(|| {
        pp::initialize_globals(modu);
        ppb::initialize_globals(gbi);
    });

    match result {
        result::Ok(()) => ffi::PP_OK,
        result::Err(_) => {
            // Nb: this gets printed to chrome's stdout if it is running on a console.
            // Otherwise it falls into a black hole and is eaten.
            println!("module initialization failed");
            1i32
        }
    }
}
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn PPP_ShutdownModule() {
    use std::rt::local::{Local};
    use self::entry::try_block;
    use std::rt::task::Task;
    // FIXME
    let _ = try_block(|| { unsafe {
        deinitialize_instances();
    }} );
    let _: Box<Task> = Local::take();
}
