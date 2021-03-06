// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*! Rust idiomatic wrapper for the Pepper API.

To use, you will need to implement at least these two functions:

```rust
#![no_main]
#[no_managle]
pub extern fn ppapi_instance_created(instance: Instance,
                                     args: Vec<(String, String)>) {
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
#![feature(linkage)]
#![feature(thread_local)]
#![feature(unboxed_closures)]
#![feature(box_syntax)] #![feature(box_patterns)]
#![feature(collections)]
#![feature(core)]
#![feature(scoped_tls)]
#![feature(map_in_place)]
#![feature(enumset)]
#![feature(iter_idx)]
#![feature(catch_panic)]

#![allow(dead_code)]
#![allow(non_shorthand_field_patterns)]

#[macro_use]
extern crate log;
extern crate collections;
extern crate hyper;
extern crate httparse;
extern crate url as iurl;
extern crate libc;

use std::{cmp};
use std::mem::{self, transmute};
use std::ops;
use std::clone;
use std::result;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::marker::PhantomData;

use log::LogRecord;

use ppb::{ViewIf, MessageLoopIf, VarIf, ImageDataIf, URLLoaderIf,
          URLRequestInfoIf, VarDictionaryIf, VarArrayIf,
          ConsoleInterface};

pub use http as url;

macro_rules! impl_resource_for(
    ($ty:ty, $type_:expr) => (
        unsafe impl Send for $ty {}
        impl ::Resource for $ty {
            fn unwrap(&self) -> ::ffi::PP_Resource {
                let res: &::ffi::PP_Resource = unsafe {
                    ::std::mem::transmute(self)
                };
                assert!(*res != 0);
                *res
            }
            fn type_of(&self) -> Option<::ResourceType> {
                use ::ResourceType;
                Some($type_)
            }
        }
        impl $ty {
            #[doc(hidden)]
            pub fn new(res: ::ffi::PP_Resource) -> $ty {
                unsafe {
                    ::std::mem::transmute_copy(&res)
                }
            }
        }
    )
);
macro_rules! impl_clone_drop_for(
    ($ty:ty) => (
        impl Clone for $ty {
            fn clone(&self) -> $ty {
                use ::Resource;
                (::ppb::get_core().AddRefResource.unwrap())(self.unwrap());
                unsafe {
                    ::std::mem::transmute_copy(self)
                }
            }
        }
        impl Drop for $ty {
            fn drop(&mut self) {
                use ::Resource;
                (::ppb::get_core().ReleaseResource.unwrap())(self.unwrap());
            }
        }
    )
);
#[macro_export]
macro_rules! try_code(
    ($expr:expr) => ({
        let code = $expr;
        if !code.is_ok() { return code.map_err(); }
        code.unwrap()
    });
    ($expr:expr => CC($cc:expr)) => ({
        let code = $expr;
        return $cc.drop_with_code(code);
    })
);

#[allow(missing_docs)] pub mod ffi;
pub mod ppp;
pub mod pp;
pub mod ppb;
pub mod gles;
pub mod font;
pub mod imagedata;
pub mod input;
pub mod http;
pub mod fs;
pub mod media_stream_video_track;
pub mod video_frame;
pub mod video_decoder;
pub mod message_handler;

#[cfg(feature = "pepper")]
#[link(name = "helper", kind = "static")]
#[link(name = "ppapi_stub", kind = "static")]
extern {}

pub type Result<T> = result::Result<T, Code>;

/// You must null terminate all strings provided.
pub fn mount<'s, 't, 'f, 'd>(source: &'s str,
                             target: &'t str,
                             filesystem_type: &'f str,
                             data: &'d str) -> Code {
    match unsafe {
        ffi::mount(source.as_ptr() as *const i8,
                   target.as_ptr() as *const i8,
                   filesystem_type.as_ptr() as *const i8,
                   0,
                   data.as_ptr() as *const libc::c_void)
    } {
        c if c >= 0 => Code::Ok(c as usize),
        -1 => Code::Failed,
        _ => {
            warn!("Unrecognized failure code");
            Code::Failed
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

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
#[must_use]
pub enum Code<T = usize> {
    Ok(T),
    CompletionPending, // = ffi::PP_OK_COMPLETIONPENDING,

    /// eventually, these will be split from Code.
    BadResource,       // = ffi::PP_ERROR_BADRESOURCE,
    BadArgument,       // = ffi::PP_ERROR_BADARGUMENT,
    WrongThread,       // = ffi::PP_ERROR_WRONG_THREAD,
    InProgress,        // = ffi::PP_ERROR_INPROGRESS,
    Failed,            // = ffi::PP_ERROR_FAILED,
    NotSupported,      // = ffi::PP_ERROR_NOTSUPPORTED,
    NoMemory,          // = ffi::PP_ERROR_NOMEMORY,
    NoSpace,           // = ffi::PP_ERROR_NOSPACE,
    NoQuota,           // = ffi::PP_ERROR_NOQUOTA,
    ContextLost,       // = ffi::PP_ERROR_CONTEXT_LOST,
    FileNotFound,      // = ffi::PP_ERROR_FILENOTFOUND,
    FileExists,        // = ffi::PP_ERROR_FILEEXISTS,
    NoAccess,          // = ffi::PP_ERROR_NOACCESS,
    ConnectionRefused, // = ffi::PP_ERROR_CONNECTION_REFUSED,
    ConnectionReset,   // = ffi::PP_ERROR_CONNECTION_RESET,
    ConnectionAborted, // = ffi::PP_ERROR_CONNECTION_ABORTED,
    ConnectionClosed,  // = ffi::PP_ERROR_CONNECTION_CLOSED,
    TimedOut,          // = ffi::PP_ERROR_TIMEDOUT,
    NoMessageLoop,     // = ffi::PP_ERROR_NO_MESSAGE_LOOP,

    /// See PP_ERROR_ABORTED.
    Aborted,

    /// See PP_ERROR_NOINTERFACE.
    NoInterface,
    /// The instance handle is no longer valid. This will happen after the
    /// instance is destroyed.
    BadInstance,
}
impl<T: fmt::Display> fmt::Display for Code<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match self {
            &Code::Ok(ref v)   => {
                return write!(f, "ok({})", v);
            },
            &Code::BadResource => "bad resource",
            &Code::BadArgument => "bad argument",
            &Code::WrongThread => "wrong thread",
            &Code::InProgress  => "in-progress",
            &Code::Failed      => "failed",
            &Code::NoMemory    => "no memory",
            &Code::ContextLost => "context lost",
            &Code::CompletionPending => "completion callback pending",
            &Code::NoSpace     => "no space left",
            &Code::NoQuota     => "no quota left",
            &Code::FileNotFound => "file not found",
            &Code::FileExists  => "file exists",
            &Code::NoAccess    => "insufficient privileges",
            &Code::ConnectionRefused => "connection attempt refused",
            &Code::ConnectionReset => "connection reset",
            &Code::ConnectionAborted => "connection aborted",
            &Code::ConnectionClosed => "connection closed",
            &Code::TimedOut    => "operation timed out",
            &Code::NotSupported => "operation not supported/implemented",
            &Code::NoMessageLoop =>
                "this thread doesn't have an attached message loop",
            &Code::NoInterface => "missing PPAPI interface",
            &Code::BadInstance => "instance destroyed",
            &Code::Aborted => "callback aborted",
        };
        write!(f, "{}", desc)
    }
}
impl From<i32> for Code {
    fn from(v: i32) -> Code {
        match v {
            v if v >= 0 => Code::Ok(v as usize),
            ffi::PP_OK_COMPLETIONPENDING => Code::CompletionPending,
            ffi::PP_ERROR_BADRESOURCE => Code::BadResource,
            ffi::PP_ERROR_BADARGUMENT => Code::BadArgument,
            ffi::PP_ERROR_WRONG_THREAD => Code::WrongThread,
            ffi::PP_ERROR_INPROGRESS => Code::InProgress,
            ffi::PP_ERROR_FAILED => Code::Failed,
            ffi::PP_ERROR_NOTSUPPORTED => Code::NotSupported,
            ffi::PP_ERROR_NOMEMORY => Code::NoMemory,
            ffi::PP_ERROR_CONTEXT_LOST => Code::ContextLost,
            ffi::PP_ERROR_FILENOTFOUND => Code::FileNotFound,
            ffi::PP_ERROR_FILEEXISTS => Code::FileExists,
            ffi::PP_ERROR_NOACCESS => Code::NoAccess,
            ffi::PP_ERROR_CONNECTION_REFUSED => Code::ConnectionRefused,
            ffi::PP_ERROR_CONNECTION_RESET => Code::ConnectionReset,
            ffi::PP_ERROR_CONNECTION_ABORTED => Code::ConnectionAborted,
            ffi::PP_ERROR_CONNECTION_CLOSED => Code::ConnectionClosed,
            ffi::PP_ERROR_TIMEDOUT | ffi::PP_ERROR_CONNECTION_TIMEDOUT =>
                Code::TimedOut,
            ffi::PP_ERROR_NO_MESSAGE_LOOP => Code::NoMessageLoop,
            ffi::PP_ERROR_NOINTERFACE => Code::NoInterface,
            ffi::PP_ERROR_ABORTED => Code::Aborted,

            _ => unreachable!("unexpected invalid or unknown code: `{}`", v),
        }
    }
}
impl<'a, T> From<&'a Result<T>> for Code {
    fn from(v: &'a Result<T>) -> Code {
        match v {
            &Ok(_) => Code::Ok(0),
            &Err(code) => code,
        }
    }
}
impl ::std::error::Error for Code {
    fn description(&self) -> &str {
        match self {
            &Code::Ok(_)       => "ok",
            &Code::BadResource => "bad resource",
            &Code::BadArgument => "bad argument",
            &Code::WrongThread => "wrong thread",
            &Code::InProgress  => "in-progress",
            &Code::Failed      => "failed",
            &Code::NoMemory    => "no memory",
            &Code::ContextLost => "context lost",
            &Code::CompletionPending => "completion callback pending",
            &Code::NoSpace     => "no space left",
            &Code::NoQuota     => "no quota left",
            &Code::FileNotFound => "file not found",
            &Code::FileExists  => "file exists",
            &Code::NoAccess    => "insufficient privileges",
            &Code::ConnectionRefused => "connection attempt refused",
            &Code::ConnectionReset => "connection reset",
            &Code::ConnectionAborted => "connection aborted",
            &Code::ConnectionClosed => "connection closed",
            &Code::TimedOut    => "operation timed out",
            &Code::NotSupported => "operation not supported/implemented",
            &Code::NoMessageLoop =>
                "this thread doesn't have an attached message loop",
            &Code::NoInterface => "missing PPAPI interface",
            &Code::BadInstance => "instance destroyed",
            &Code::Aborted => "callback aborted",
        }
    }
}
impl Into<::std::io::Error> for Code {
    fn into(self) -> ::std::io::Error {
        use std::io::{ErrorKind, Error};
        assert!(!self.is_ok());
        let kind = match self {
            Code::TimedOut => ErrorKind::TimedOut,
            Code::ConnectionClosed => ErrorKind::NotConnected,
            Code::ConnectionAborted => ErrorKind::ConnectionAborted,
            Code::ConnectionReset => ErrorKind::ConnectionReset,
            Code::ConnectionRefused => ErrorKind::ConnectionRefused,
            _ => ErrorKind::Other, // TODO
        };

        Error::new(kind, self)
    }
}
impl<T> Into<Result<T>> for Code<T> {
    fn into(self) -> Result<T> {
        if let Code::Ok(v) = self {
            Ok(v)
        } else {
            Err(self.map_err())
        }
    }
}
impl Code {
    pub fn from_i32(code: i32) -> Code {
        From::from(code)
    }
    pub fn to_i32(self) -> i32 {
        match self {
            Code::Ok(v)             => v as i32,
            Code::CompletionPending => ffi::PP_OK_COMPLETIONPENDING,
            Code::BadResource => ffi::PP_ERROR_BADRESOURCE,
            Code::BadArgument => ffi::PP_ERROR_BADARGUMENT,
            Code::WrongThread => ffi::PP_ERROR_WRONG_THREAD,
            Code::InProgress  => ffi::PP_ERROR_INPROGRESS,
            Code::Failed      => ffi::PP_ERROR_FAILED,
            Code::NotSupported=> ffi::PP_ERROR_NOTSUPPORTED,
            Code::NoMemory    => ffi::PP_ERROR_NOMEMORY,
            Code::ContextLost => ffi::PP_ERROR_CONTEXT_LOST,
            Code::NoSpace     => ffi::PP_ERROR_NOSPACE,
            Code::NoQuota     => ffi::PP_ERROR_NOQUOTA,
            Code::FileNotFound => ffi::PP_ERROR_FILENOTFOUND,
            Code::FileExists  => ffi::PP_ERROR_FILEEXISTS,
            Code::NoAccess    => ffi::PP_ERROR_NOACCESS,
            Code::ConnectionRefused => ffi::PP_ERROR_CONNECTION_REFUSED,
            Code::ConnectionReset => ffi::PP_ERROR_CONNECTION_RESET,
            Code::ConnectionAborted => ffi::PP_ERROR_CONNECTION_ABORTED,
            Code::ConnectionClosed => ffi::PP_ERROR_CONNECTION_CLOSED,
            Code::TimedOut    => ffi::PP_ERROR_TIMEDOUT,
            Code::NoMessageLoop => ffi::PP_ERROR_NO_MESSAGE_LOOP,
            Code::NoInterface => ffi::PP_ERROR_NOINTERFACE,
            Code::Aborted     => ffi::PP_ERROR_ABORTED,

            Code::BadInstance => ffi::PP_ERROR_RESOURCE_FAILED,
        }
    }
    pub fn to_empty_result(self) -> Result<()> {
        if self.is_ok() {
            result::Result::Ok(())
        } else {
            result::Result::Err(self)
        }
    }
    pub fn to_result<T, F>(self, ok: F) -> Result<T> where F: FnOnce(Code) -> T {
        if self.is_ok() {
            result::Result::Ok(ok(self))
        } else {
            result::Result::Err(self)
        }
    }
    pub fn to_valued_result<T>(self, val: T) -> Result<T> {
        if self.is_ok() {
            result::Result::Ok(val)
        } else {
            result::Result::Err(self)
        }
    }
    pub fn expect(self, msg: &str) {
        if !self.is_ok() {
            panic!("Code: `{code:}`, Message: `{msg:}`",
                  code=self, msg=msg)
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
impl<T> Code<T> {
    pub fn map_ok<F, U>(self, f: F) -> Code<U>
        where F: FnOnce(T) -> U,
    {
        match self {
            Code::Ok(v) => Code::Ok(f(v)),

            // Well, this is nice.
            Code::CompletionPending => Code::CompletionPending,
            Code::BadResource => Code::BadResource,
            Code::BadArgument => Code::BadArgument,
            Code::WrongThread => Code::WrongThread,
            Code::InProgress  => Code::InProgress,
            Code::Failed      => Code::Failed,
            Code::NotSupported=> Code::NotSupported,
            Code::NoMemory    => Code::NoMemory,
            Code::ContextLost => Code::ContextLost,
            Code::NoSpace     => Code::NoSpace,
            Code::NoQuota     => Code::NoQuota,
            Code::FileNotFound => Code::FileNotFound,
            Code::FileExists  => Code::FileExists,
            Code::NoAccess    => Code::NoAccess,
            Code::ConnectionRefused => Code::ConnectionRefused,
            Code::ConnectionReset => Code::ConnectionReset,
            Code::ConnectionAborted => Code::ConnectionAborted,
            Code::ConnectionClosed => Code::ConnectionClosed,
            Code::TimedOut    => Code::TimedOut,
            Code::NoMessageLoop => Code::NoMessageLoop,
            Code::NoInterface => Code::NoInterface,
            Code::BadInstance => Code::BadInstance,
            Code::Aborted => Code::Aborted,
        }
    }
    pub fn map_err<U>(&self) -> Code<U> {
        match self {
            &Code::BadResource => Code::BadResource,
            &Code::BadArgument => Code::BadArgument,
            &Code::WrongThread => Code::WrongThread,
            &Code::InProgress  => Code::InProgress,
            &Code::Failed      => Code::Failed,
            &Code::NotSupported=> Code::NotSupported,
            &Code::NoMemory    => Code::NoMemory,
            &Code::ContextLost => Code::ContextLost,
            &Code::NoSpace     => Code::NoSpace,
            &Code::NoQuota     => Code::NoQuota,
            &Code::FileNotFound => Code::FileNotFound,
            &Code::FileExists  => Code::FileExists,
            &Code::NoAccess    => Code::NoAccess,
            &Code::ConnectionRefused => Code::ConnectionRefused,
            &Code::ConnectionReset => Code::ConnectionReset,
            &Code::ConnectionAborted => Code::ConnectionAborted,
            &Code::ConnectionClosed => Code::ConnectionClosed,
            &Code::TimedOut    => Code::TimedOut,
            &Code::NoMessageLoop => Code::NoMessageLoop,
            &Code::NoInterface => Code::NoInterface,
            &Code::BadInstance => Code::BadInstance,
            &Code::Aborted => Code::Aborted,
            &Code::CompletionPending => Code::CompletionPending,

            &Code::Ok(_) => unreachable!(),
        }
    }
    pub fn ok(self) -> Option<T> {
        match self {
            Code::Ok(v) => Some(v),
            _ => None,
        }
    }
    pub fn is_ok(&self) -> bool {
        match self {
            &Code::Ok(_) => true,
            _ => false,
        }
    }
    pub fn completion_pending(&self) -> bool {
        match self {
            &Code::CompletionPending => true,
            _ => false,
        }
    }
    pub fn unwrap_completion_pending(self) {
        if !self.completion_pending() {
            let code: Code = Code::CompletionPending;
            panic!("unexpected error code `{:?}`; expected `{:?}`",
                   self.map_err::<usize>(), code)
        }
    }
    pub fn aborted(&self) -> bool {
        match self {
            &Code::Aborted => true,
            _ => false,
        }
    }
    pub fn is_err(&self) -> bool {
        !self.is_ok() && !self.completion_pending()
    }
    pub fn unwrap(self) -> T {
        if let Code::Ok(v) = self {
            v
        } else {
            let code: Code<usize> = self.map_err();
            panic!("unexpected error code `{:?}`", code)
        }
    }
}

impl ops::Add for ffi::Struct_PP_Point {
    type Output = ffi::Struct_PP_Point;
    fn add(self, rhs: ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl ops::Sub for ffi::Struct_PP_Point {
    type Output = ffi::Struct_PP_Point;
    fn sub(self, rhs: ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl ops::Mul for ffi::Struct_PP_Point {
    type Output = ffi::Struct_PP_Point;
    fn mul(self, rhs: ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}
impl ops::Div for ffi::Struct_PP_Point {
    type Output = ffi::Struct_PP_Point;
    fn div(self, rhs: ffi::Struct_PP_Point) -> ffi::Struct_PP_Point {
        ffi::Struct_PP_Point {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
impl ops::Add for ffi::Struct_PP_Size {
    type Output = ffi::Struct_PP_Size;
    fn add(self, rhs: ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}
impl ops::Sub for ffi::Struct_PP_Size {
    type Output = ffi::Struct_PP_Size;
    fn sub(self, rhs: ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}
impl ops::Mul for ffi::Struct_PP_Size {
    type Output = ffi::Struct_PP_Size;
    fn mul(self, rhs: ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}
impl ops::Div for ffi::Struct_PP_Size {
    type Output = ffi::Struct_PP_Size;
    fn div(self, rhs: ffi::Struct_PP_Size) -> ffi::Struct_PP_Size {
        ffi::Struct_PP_Size {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
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

impl fmt::Debug for ffi::Struct_PP_FloatPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FloatPoint({}, {})", self.x, self.y)
    }
}

pub type FloatPoint = ffi::PP_FloatPoint;
pub type TouchPoint = ffi::PP_TouchPoint;
pub type Ticks = ffi::PP_TimeTicks;
pub type Time = ffi::PP_Time;
pub type TimeDelta = ffi::PP_TimeDelta;

// duplicated here so we don't have such a long name for this.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
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
}
impl From<ffi::PP_Size> for Size {
    fn from(v: ffi::PP_Size) -> Size {
        use std::mem::transmute;
        unsafe { transmute(v) }
    }
}
impl Into<ffi::PP_Size> for Size {
    fn into(self) -> ffi::PP_Size {
        use std::mem::transmute;
        unsafe { transmute(self) }
    }
}

// duplicated here so we don't have such a long name for this.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct Rect {
    pub point: Point,
    pub size:  Size,
}
impl Rect {
    pub fn new(point: Point, size: Size) -> Rect {
        Rect {
            point: point,
            size: size,
        }
    }
}
impl From<ffi::PP_Rect> for Rect {
    fn from(v: ffi::PP_Rect) -> Rect {
        use std::mem::transmute;
        unsafe { transmute(v) }
    }
}
impl Into<ffi::PP_Rect> for Rect {
    fn into(self) -> ffi::PP_Rect {
        use std::mem::transmute;
        unsafe { transmute(self) }
    }
}
// duplicated here so we don't have such a long name for this.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}
impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point {
            x: x,
            y: y,
        }
    }
}
impl From<ffi::PP_Point> for Point {
    fn from(v: ffi::PP_Point) -> Point {
        use std::mem::transmute;
        unsafe { transmute(v) }
    }
}
impl Into<ffi::PP_Point> for Point {
    fn into(self) -> ffi::PP_Point {
        use std::mem::transmute;
        unsafe { transmute(self) }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ResourceType {
    WheelInputEvent,
    WebSocket,
    View,
    UrlResponseInfo,
    UrlRequestInfo,
    UrlLoader,
    UdpSocket,
    TrueTypeFont,
    TouchInputEvent,
    TcpSocket,
    NetworkMonitor,
    NetworkList,
    NetworkAddress,
    MouseInputEvent,
    MessageLoop,
    KeyboardInputEvent,
    ImageData,
    IMEInputEvent,
    HostResolver,
    Graphics3D,
    Graphics2D,
    Font,
    FileSystem,
    FileRef,
    FileIo,
    AudioConfig,
    Audio,
    VideoTrack,
    VideoFrame,
    VideoDecoder,
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub struct GenericResource(ffi::PP_Resource);
unsafe impl Send for GenericResource { }
impl GenericResource {
    pub fn is_graphics3d(&self) -> bool {
        use ppb::*;
        get_graphics_3d_opt().is(self.unwrap())
    }
    pub fn media_stream_video_track(&self) ->
        Option<media_stream_video_track::VideoTrack>
    {
        use ppb::*;
        if get_media_stream_video_track_opt().is(self.unwrap()) {
            Some(From::from(self.unwrap()))
        } else {
            None
        }
    }
}
impl_clone_drop_for!(GenericResource);
impl Resource for GenericResource {
    fn unwrap(&self) -> ffi::PP_Resource { self.0 }

    fn type_of(&self) -> Option<ResourceType> {
        use ppb::*;
        let t;

        if get_graphics_3d_opt().is(self.0) {
            t = Some(ResourceType::Graphics3D);
        } else if get_keyboard_event_opt().is(self.0) {
            t = Some(ResourceType::KeyboardInputEvent);
        } else if get_mouse_event_opt().is(self.0) {
            t = Some(ResourceType::MouseInputEvent);
        } else if get_wheel_event_opt().is(self.0) {
            t = Some(ResourceType::WheelInputEvent);
        } else if get_touch_event_opt().is(self.0) {
            t = Some(ResourceType::TouchInputEvent);
        //} else if get_ime_event_opt().is(self.0) {
        //    t = Some(ResourceType::IMEInputEvent);
        } else if get_url_loader_opt().is(self.0) {
            t = Some(ResourceType::UrlLoader);
        } else if get_url_request_opt().is(self.0) {
            t = Some(ResourceType::UrlRequestInfo);
        } else if get_url_response_opt().is(self.0) {
            t = Some(ResourceType::UrlResponseInfo);
        } else if get_view_opt().is(self.0) {
            t = Some(ResourceType::View);
        } else if get_file_system_opt().is(self.0) {
            t = Some(ResourceType::FileSystem);
        } else if get_media_stream_video_track_opt().is(self.0) {
            t = Some(ResourceType::VideoTrack);
        } else if get_video_frame_opt().is(self.0) {
            t = Some(ResourceType::VideoFrame);
        } else {
            t = None;
        }

        t
    }
}
impl ToVar for GenericResource {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe { ffi::resource_id_to_var(self.unwrap()) }
    }
}
#[doc(hidden)]
impl From<ffi::PP_Resource> for GenericResource {
    fn from(v: ffi::PP_Resource) -> GenericResource {
        GenericResource(v)
    }
}

pub trait Resource: Clone + Send {
    #[doc(hidden)] fn unwrap(&self) -> ffi::PP_Resource;

    fn type_of(&self) -> Option<ResourceType>;
}
pub trait ContextResource: Resource {
    fn get_device(&self) -> ffi::PP_Resource;
}
#[derive(Hash, Eq, PartialEq, Debug)] pub struct Context2d(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Debug)] pub struct View(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Debug)] pub struct MessageLoop(ffi::PP_Resource);

impl_resource_for!(Context2d, ResourceType::Graphics2D);
impl_clone_drop_for!(Context2d);
impl_resource_for!(View, ResourceType::View);
impl_clone_drop_for!(View);
impl_resource_for!(MessageLoop, ResourceType::MessageLoop);
impl_clone_drop_for!(MessageLoop);

impl ContextResource for Context2d {
    fn get_device(&self) -> ffi::PP_Resource {
        self.unwrap()
    }
}
impl View {
    pub fn rect(&self) -> Option<Rect> {
        ppb::get_view()
            .rect(self.unwrap())
            .map(|rect| From::from(rect) )
    }
    pub fn is_fullscreen(&self) -> bool {
        ppb::get_view().is_fullscreen(self.unwrap())
    }
    pub fn is_visible(&self) -> bool {
        ppb::get_view().is_visible(self.unwrap())
    }
    pub fn is_page_visible(&self) -> bool {
        ppb::get_view().is_page_visible(self.unwrap())
    }
    pub fn clip_rect(&self) -> Option<Rect> {
        ppb::get_view()
            .clip_rect(self.unwrap())
            .map(|rect| From::from(rect) )
    }
    pub fn device_scale(&self) -> f32 {
        ppb::get_view().device_scale(self.unwrap())
    }
    pub fn css_scale(&self) -> f32 {
        ppb::get_view().css_scale(self.unwrap())
    }
}
impl Messaging {
    pub fn post<T: ToVar>(&self, message: T) {
        use ppb::MessagingIf;
        ppb::get_messaging().post_message(self.unwrap(), message.to_var())
    }
}
impl MessageLoop {
    fn get_main_loop() -> MessageLoop {
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
    pub fn post_work<F>(&self, work: F, delay: u64) -> Code
        where F: FnOnce(Code<()>) + Send
    {
        let work = CallbackArgs::new(work);
        let cc = work.to_ffi_callback((), Default::default());
        match ppb::get_message_loop().post_work(&self.unwrap(), cc.cc(), delay as i64) {
            ffi::PP_ERROR_BADARGUMENT => panic!("internal error: completion callback was null?"),
            c => Code::from_i32(c),
        }
    }
    pub fn post_to_self<F>(work: F, delay: u64) -> Code
        where F: FnOnce(Code<()>)
    {
        MessageLoop::current()
            .map(move |m| {
                let work = CallbackArgs::new(work);
                let cc = work.to_ffi_callback((), Default::default());
                let code = ppb::get_message_loop().post_work(&m.unwrap(), cc.cc(),
                                                             delay as i64);
                From::from(code)
            })
            .unwrap_or(Code::NoMessageLoop)
    }

    /// Work posted beforehand will still run.
    pub fn queue_pause(&self) -> Code {
        Code::from_i32((ppb::get_message_loop().PostQuit.unwrap())(self.unwrap(), ffi::PP_FALSE))
    }

    /// Work posted beforehand will still run.
    pub fn queue_shutdown(&self) -> Code {
        message_handler::unregister_handlers(self);
        Code::from_i32((ppb::get_message_loop().PostQuit.unwrap())(self.unwrap(), ffi::PP_TRUE))
    }
}

#[derive(Clone, Debug)]
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
    Resource(GenericResource),
}
#[derive(Clone, Eq, PartialEq, Copy)]
pub struct NullVar;
#[derive(Clone, Eq, PartialEq, Copy)]
pub struct UndefinedVar;
#[derive(Eq, PartialEq, Hash)]
pub struct StringVar     (i64);
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ObjectVar     (i64);
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ArrayVar      (i64);
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct DictionaryVar (i64);
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ArrayBufferVar(i64);

pub trait UnwrapOr {
    fn unwrap_or_null(&self) -> AnyVar;
    fn unwrap_or_undefined(&self) -> AnyVar;
}
impl<T: ToVar> UnwrapOr for Option<T> {
    fn unwrap_or_null(&self) -> AnyVar {
        self.as_ref().map(|v| v.to_any() ).unwrap_or(AnyVar::Null)
    }
    fn unwrap_or_undefined(&self) -> AnyVar {
        self.as_ref().map(|v| v.to_any() ).unwrap_or(AnyVar::Undefined)
    }
}

pub trait ByRefVar {
    fn get_id(&self) -> i64;
}
pub trait ToVar {
    fn to_var(&self) -> ffi::PP_Var;
    #[inline]
    fn to_any(&self) -> AnyVar {
        AnyVar::new_bumped(self.to_var())
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
pub trait Var {
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
impl Clone for ffi::PP_Var {
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
    ($ty:ty, $is_true_name:ident) => (
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
                    let v: $ty = VarCtor::ctor(var);
                    mem::forget(v.clone());
                    Some(v)
                } else {
                    None
                }
            }
        }
    )
);
impl_clone_drop_for!(StringVar, is_a_string);
impl_clone_drop_for!(ObjectVar, is_an_object);
impl_clone_drop_for!(ArrayVar, is_an_array);
impl_clone_drop_for!(DictionaryVar, is_a_dictionary);
impl_clone_drop_for!(ArrayBufferVar, is_an_array_buffer);

macro_rules! impl_var_for(
    ($ty:ty, $is_true_name:ident) => (
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
                    let v: $ty = VarCtor::ctor(var);
                    mem::forget(v.clone());
                    Some(v)
                } else {
                    None
                }
            }
        }
    )
);
impl_var_for!(NullVar, is_null);
impl_var_for!(UndefinedVar, is_undefined);
impl_var_for!(bool, is_a_bool);
impl_var_for!(i32, is_an_i32);
impl_var_for!(f64, is_a_f64);

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
impl<'a> ToStringVar for &'a StringVar {
    fn to_string_var(&self) -> StringVar {
        (*self).clone()
    }
}
impl Var for ::std::string::String {
    #[inline] fn is_a_string(&self) -> bool { true }
}
impl ToStringVar for ::std::string::String {
    #[inline] fn to_string_var(&self) -> StringVar {
        StringVar::new_from_str(self.as_ref())
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
            &AnyVar::Null => {NullVar}.to_var(),
            &AnyVar::Undefined => {UndefinedVar}.to_var(),
            &AnyVar::Bool(b) => b.to_var(),
            &AnyVar::I32(i) => i.to_var(),
            &AnyVar::F64(f) => f.to_var(),
            &AnyVar::String(ref v) => v.to_var(),
            &AnyVar::Object(ref v) => v.to_var(),
            &AnyVar::Array(ref v) => v.to_var(),
            &AnyVar::Dictionary(ref v) => v.to_var(),
            &AnyVar::ArrayBuffer(ref v) => v.to_var(),
            &AnyVar::Resource(ref r) => r.to_var(),
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
);
impl_by_ref_var!(StringVar);
impl_by_ref_var!(ObjectVar);
impl_by_ref_var!(ArrayVar);
impl_by_ref_var!(DictionaryVar);
impl_by_ref_var!(ArrayBufferVar);

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
);
impl_to_var_int!(i8);
impl_to_var_int!(i16);
impl_to_var_int!(i32);

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
);
impl_to_var_float!(f32);
impl_to_var_float!(f64);

impl<'s> ToVar for &'s bool {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(**self as u8)
        }
    }
}
impl ToVar for bool {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(*self as u8)
        }
    }
}
impl ToVar for Box<bool> {
    fn to_var(&self) -> ffi::PP_Var {
        unsafe {
            ffi::bool_to_var(**self as u8)
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
    fn new(var: ffi::PP_Var) -> AnyVar {
        if var.is_null() {
            AnyVar::Null
        } else if var.is_undefined() {
            AnyVar::Undefined
        } else if var.is_a_bool() {
            AnyVar::Bool(unsafe { ffi::bool_from_var(var) != 0 })
        } else if var.is_an_i32() {
            AnyVar::I32(unsafe { ffi::i32_from_var(var) })
        } else if var.is_a_f64() {
            AnyVar::F64(unsafe { ffi::f64_from_var(var) })
        } else if var.is_a_string() {
            AnyVar::String(StringVar(unsafe { ffi::id_from_var(var) }))
        } else if var.is_an_object() {
            AnyVar::Object(ObjectVar(unsafe { ffi::id_from_var(var) }))
        } else if var.is_an_array() {
            AnyVar::Array(ArrayVar(unsafe { ffi::id_from_var(var) }))
        } else if var.is_a_dictionary() {
            AnyVar::Dictionary(DictionaryVar(unsafe { ffi::id_from_var(var) }))
        } else if var.is_an_array_buffer() {
            AnyVar::ArrayBuffer(ArrayBufferVar(unsafe { ffi::id_from_var(var) }))
        } else if var.is_a_resource() {
            AnyVar::Undefined
        } else {
            error!("Var doesn't have a known type");
            AnyVar::Undefined
        }
    }
    fn new_bumped(var: ffi::PP_Var) -> AnyVar {
        let v = AnyVar::new(var);
        // bump the ref count:
        mem::forget(v.clone());
        v
    }
    pub fn is_ref_counted(&self) -> bool {
        self.is_a_string() ||
            self.is_an_object() ||
            self.is_an_array() ||
            self.is_a_dictionary() ||
            self.is_an_array_buffer() ||
            self.is_a_resource()
    }

    pub fn get_string_ref(&self) -> Option<&StringVar> {
        match self {
            &AnyVar::String(ref s) => Some(s),
            _ => None,
        }
    }
    pub fn get_string(&self) -> Option<StringVar> {
        match self {
            &AnyVar::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn get_dict_ref(&self) -> Option<&DictionaryVar> {
        match self {
            &AnyVar::Dictionary(ref s) => Some(s),
            _ => None,
        }
    }
    pub fn get_dict(&self) -> Option<DictionaryVar> {
        match self {
            &AnyVar::Dictionary(ref s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn get_resource_ref(&self) -> Option<&GenericResource> {
        if let &AnyVar::Resource(ref r) = self {
            Some(r)
        } else {
            None
        }
    }
    pub fn get_resource(&self) -> Option<GenericResource> {
        if let &AnyVar::Resource(ref r) = self {
            Some(r.clone())
        } else {
            None
        }
    }
    pub fn get_i32(&self) -> Option<i32> {
        if let &AnyVar::I32(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl fmt::Debug for StringVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StringVar({}) = \"{}\"",
               self.get_id(),
               self)
    }
}
impl fmt::Display for StringVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.as_ref())
    }
}
impl cmp::PartialEq<str> for StringVar {
    fn eq(&self, rhs: &str) -> bool {
        &**self == rhs
    }
}
impl<'a> cmp::PartialEq<&'a str> for StringVar {
    fn eq(&self, rhs: &&'a str) -> bool {
        &**self == *rhs
    }
}
impl ops::Deref for StringVar {
    type Target = str;
    fn deref<'a>(&'a self) -> &'a str {
        use std::str::from_utf8_unchecked;
        use std::slice::from_raw_parts;
        use std::mem::transmute;

        let f = ppb::get_var().VarToUtf8.unwrap();

        unsafe {
            let mut len: u32 = mem::uninitialized();
            let buf = f(self.to_var(), &mut len as *mut u32);
            let len = len as usize;
            let slice = from_raw_parts(transmute(buf), len);
            transmute(from_utf8_unchecked(slice))
        }
    }
}
impl StringVar {
    pub fn new<T: AsRef<str>>(v: T) -> StringVar {
        StringVar::new_from_str(v.as_ref())
    }
    pub fn new_from_str(v: &str) -> StringVar {
        let len = v.len();
        let var = (ppb::get_var().VarFromUtf8.unwrap())
                (v.as_ptr() as *const i8,
                 len as u32);
        return StringVar(unsafe { ffi::id_from_var(var) } );
    }
    fn new_from_var(v: ffi::PP_Var) -> StringVar {
        let v: StringVar = From::from(v);
        mem::forget(v.clone());
        v
    }
    pub fn as_str(&self) -> &str { &*self }
}
#[doc(hidden)]
impl From<ffi::PP_Var> for StringVar {
    fn from(v: ffi::PP_Var) -> StringVar {
        let v: StringVar = StringVar(unsafe { ffi::id_from_var(v) });
        mem::forget(v.clone());
        v
    }
}
impl ObjectVar {
    fn new_from_var(v: ffi::PP_Var) -> ObjectVar {
        ObjectVar(unsafe { ffi::id_from_var(v) })
    }
}
pub struct ArrayVarIter<'a> {
    var: &'a ArrayVar,
    index: usize,
    len: usize,
}
impl<'a> Iterator for ArrayVarIter<'a> {
    type Item = AnyVar;
    fn next(&mut self) -> Option<AnyVar> {
        if self.index >= self.len { None }
        else {
            let v = self.var.get(self.index);
            self.index += 1;
            Some(v)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl ArrayVar {
    fn new_from_var(v: ffi::PP_Var) -> ArrayVar {
        ArrayVar(unsafe { ffi::id_from_var(v) })
    }
    pub fn new() -> ArrayVar {
        ArrayVar::new_from_var(ppb::get_array().create())
    }
    pub fn from<T>(v: &[T]) -> ArrayVar
        where T: ToVar,
    {
        let mut this = ArrayVar::new();
        assert!(this.resize(v.len()));
        for (index, value) in v.iter().enumerate() {
            this.set(index, value);
        }
        this
    }
    pub fn len(&self) -> usize {
        ppb::get_array().get_len(&self.to_var()) as usize
    }
    pub fn resize(&mut self, new_len: usize) -> bool {
        ppb::get_array().set_len(&self.to_var(), new_len as libc::uint32_t)
    }
    pub fn get(&self, index: usize) -> AnyVar {
        AnyVar::new_bumped(ppb::get_array().get(&self.to_var(), index as libc::uint32_t))
    }
    pub fn set<T: ToVar>(&mut self, index: usize, value: &T) -> bool {
        ppb::get_array().set(&self.to_var(), index as u32, &value.to_var())
    }

    pub fn iter<'a>(&'a self) -> ArrayVarIter<'a> {
        ArrayVarIter {
            var: self,
            index: 0,
            len: self.len(),
        }
    }
}
pub struct DictEntries<'a> {
    dict: &'a DictionaryVar,
    keys: ArrayVar,
    key_index: usize,
    len: usize,
}
impl<'a> Iterator for DictEntries<'a> {
    type Item = (StringVar, AnyVar);
    fn next(&mut self) -> Option<(StringVar, AnyVar)> {
        if self.key_index >= self.len { None }
        else {
            let k = self.keys.get(self.key_index);
            let k = match k {
                AnyVar::String(k) => k,
                k => unreachable!("dictionary keys should always be stored as strings: `{:?}` was not.", k),
            };
            let v = self.dict.get(&k);
            self.key_index = self.key_index + 1;
            Some((k, v))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl DictionaryVar {
    fn new_from_var(v: ffi::PP_Var) -> DictionaryVar {
        DictionaryVar(unsafe { ffi::id_from_var(v) })
    }

    pub fn new() -> DictionaryVar {
        DictionaryVar::new_from_var(ppb::get_dictionary().create())
    }
    pub fn len(&self) -> usize {
        self.keys().len()
    }
    pub fn has_key<T: ToStringVar>(&self, key: T) -> bool {
        let key = key.to_string_var();
        ppb::get_dictionary().has_key(&self.to_var(), &key.to_var())
    }
    pub fn get<T: ToStringVar>(&self, key: T) -> AnyVar {
        let v = ppb::get_dictionary().get(&self.to_var(), &key.to_string_var().to_var());
        AnyVar::new_bumped(v)
    }
    pub fn set<T: ToStringVar, V: ToVar>(&self, key: T, value: V) -> bool {
        let key = key.to_string_var();
        ppb::get_dictionary().set(&self.to_var(), &key.to_var(), &value.to_var())
    }
    pub fn keys(&self) -> ArrayVar {
        let v = ppb::get_dictionary().get_keys(&self.to_var());
        let v = ArrayVar::new_from_var(v);
        mem::forget(v.clone());
        v
    }
    pub fn entries<'a>(&'a self) -> DictEntries<'a> {
        let keys = self.keys();
        let keys_len = keys.len();
        DictEntries {
            dict: self,
            keys: keys,
            key_index: 0,
            len: keys_len,
        }
    }
}
impl ArrayBufferVar {
    fn new_from_var(v: ffi::PP_Var) -> ArrayBufferVar {
        ArrayBufferVar(unsafe { ffi::id_from_var(v) })
    }
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub struct Messaging(ffi::PP_Instance);
impl Messaging {
    fn unwrap(&self) -> ffi::PP_Instance {
        let &Messaging(inst) = self;
        inst
    }
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub struct Console(ffi::PP_Instance);
impl Console {
    fn unwrap(&self) -> ffi::PP_Instance {
        let &Console(inst) = self;
        inst
    }

    pub fn print<T: ToVar>(&self, msg: T) {
        self.log(ffi::PP_LOGLEVEL_TIP, msg)
    }


    pub fn err<T: ToVar>(&self, msg: T) {
        self.log(ffi::PP_LOGLEVEL_ERROR, msg)
    }
    pub fn err_with_file<T: ToVar, U: Display, V: Display>(&self, msg: T, file: U, line: V) {
        let source = format!("{}:{}", file, line)
            .to_string_var();
        self.log_with_source(ffi::PP_LOGLEVEL_ERROR, source, msg)
    }
    pub fn warn<T: ToVar>(&self, msg: T) {
        self.log(ffi::PP_LOGLEVEL_WARNING, msg)
    }
    pub fn warn_with_file<T: ToVar, U: Display, V: Display>(&self, msg: T, file: U, line: V) {
        let source = format!("{}:{}", file, line)
            .to_string_var();
        self.log_with_source(ffi::PP_LOGLEVEL_WARNING, source, msg)
    }
}

/// Update any internal self referential pointers/refs. This is only called
/// after the object has been placed into it's final callback-storage location.
pub trait InPlaceInit {
    fn inplace_init(&mut self) { }
}
impl<T> InPlaceInit for Vec<T> { }
impl InPlaceInit for () { }

/// The storage must be in-place (read: can't be moved), or the pointer to
/// storage will be invalid!
struct InPlaceArrayOutputStorage<T> {
    storage: Vec<T>,
    ffi: ffi::Struct_PP_ArrayOutput,
}
impl<T> Into<Vec<T>> for InPlaceArrayOutputStorage<T> {
    fn into(self) -> Vec<T> {
        let InPlaceArrayOutputStorage {
            storage: storage, ..
        } = self;
        storage
    }
}
unsafe impl<T: Send> Send for InPlaceArrayOutputStorage<T> { }
impl<T> AsRef<ffi::Struct_PP_ArrayOutput> for InPlaceArrayOutputStorage<T> {
    fn as_ref(&self) -> &ffi::Struct_PP_ArrayOutput { &self.ffi }
}
impl<T> AsMut<ffi::Struct_PP_ArrayOutput> for InPlaceArrayOutputStorage<T> {
    fn as_mut(&mut self) -> &mut ffi::Struct_PP_ArrayOutput { &mut self.ffi }
}
impl<T> AsRef<Vec<T>> for InPlaceArrayOutputStorage<T> {
    fn as_ref(&self) -> &Vec<T> { &self.storage }
}
impl<T> AsMut<Vec<T>> for InPlaceArrayOutputStorage<T> {
    fn as_mut(&mut self) -> &mut Vec<T> { &mut self.storage }
}
impl<T> Default for InPlaceArrayOutputStorage<T> {
    fn default() -> InPlaceArrayOutputStorage<T> {
        extern "C" fn get_data_buffer<T>(vec: *mut libc::c_void,
                                         count: libc::uint32_t,
                                         size: libc::uint32_t) -> *mut libc::c_void {
            use std::mem::size_of;
            assert!(size_of::<T>() == size as usize,
                    "PPAPI is wanting to allocate an unexpected element type");
            assert!(vec != 0 as *mut _);

            if count == 0 { return 0 as *mut _; }

            let v: &mut Vec<T> = unsafe { transmute(vec) };
            v.reserve_exact(count as usize);
            unsafe { v.set_len(count as usize); }
            v.as_mut_ptr() as *mut _
        }

        let mut this = InPlaceArrayOutputStorage {
            storage: Vec::new(),
            ffi: Default::default(),
        };
        this.ffi.GetDataBuffer = Some(get_data_buffer::<T>);
        this
    }
}
impl<T> InPlaceInit for InPlaceArrayOutputStorage<T> {
    fn inplace_init(&mut self) {
        let storage_ptr: *mut Vec<T> = unsafe {
            transmute(&mut self.storage)
        };
        self.ffi.user_data = storage_ptr as *mut libc::c_void;
    }
}

#[must_use]
pub struct CallbackCompletion<F> {
    pub cc: ffi::Struct_PP_CompletionCallback,
    _1: PhantomData<F>,
}
impl<F> CallbackCompletion<F> {
    pub fn cc(&self) -> ffi::Struct_PP_CompletionCallback { self.cc }

    pub fn drop_with_code(self, code: Code) -> Code {
        if !code.completion_pending() {
            let optional = self.cc.flags as u32 & ffi::PP_COMPLETIONCALLBACK_FLAG_OPTIONAL != 0;
            match (optional, self.cc.user_data as usize) {
                (true, 0) => { return code; }
                (true, _) => {
                    // clean up.
                    let _: Box<F> = unsafe { mem::transmute(self.cc.user_data) };

                    return code;
                },
                _ => (),
            }

            if self.cc.func.is_some() &&
                !self.cc.post_to_self(code).is_ok()
            {
                unsafe {
                    ffi::run_completion_callback(self.cc,
                                                 code.to_i32())
                }
            }
        }
        code
    }
}

pub trait Callback {
    type Fun;
    fn to_ffi_callback(self) -> CallbackCompletion<<Self as Callback>::Fun>;

    fn is_blocking(&self) -> bool { false }
}
trait PostToSelf: Send {
    fn post_to_self(self, code: Code) -> Code;
}
unsafe impl Send for ffi::Struct_PP_CompletionCallback {}
impl PostToSelf for ffi::Struct_PP_CompletionCallback {
    fn post_to_self(self, code: Code) -> Code {
        MessageLoop::post_to_self(move |_| {
            unsafe {
                ffi::run_completion_callback(self,
                                             code.to_i32())
            }
        }, 0)
    }
}

/// TODO `F` does NOT need to be `Send` unless the use case requires it, ie MessageLoop.
impl<F: Sized> Callback for F
    where F: FnOnce(Result<()>) + Send,
{
    type Fun = F;
    fn to_ffi_callback(self) -> CallbackCompletion<F> {
        extern "C" fn work_callback<F: Sized>(user: *mut libc::c_void, status: i32)
            where F : FnOnce(Result<()>) + Send
        {
            let work: Box<F> = unsafe { mem::transmute(user) };
            let code = Code::from_i32(status)
                .to_valued_result(());
            work.call_once((code,));
        }
        let cc = unsafe {
            ffi::make_completion_callback(Some(work_callback::<F>),
                                          mem::transmute(box self))
        };
        CallbackCompletion {
            cc: cc,
            _1: PhantomData,
        }
    }
}

pub struct StorageToArgsMapper<RawArgs, Args>(fn(RawArgs, Code) -> Args);
impl<RawArgs, Args> Default for StorageToArgsMapper<RawArgs, Args>
    where RawArgs: Into<Args>,
{
    fn default() -> StorageToArgsMapper<RawArgs, Args> {
        fn identity<RawArgs, Args>(i: RawArgs, _status: Code) -> Args
            where RawArgs: Into<Args>,
        {
            i.into()
        }
        StorageToArgsMapper(identity)
    }
}

struct CallbackArgsStorage<RawArgs, Args, F> {
    args: RawArgs,
    mapper: StorageToArgsMapper<RawArgs, Args>,
    f: F,
}

#[must_use]
pub struct CallbackArgsCompletion<F, Args, RawArgs> {
    raw: *mut RawArgs,
    pub cc: ffi::Struct_PP_CompletionCallback,
    _1:  PhantomData<(F, Args)>,
}
impl<F, Args, RawArgs> CallbackArgsCompletion<F, Args, RawArgs> {
    fn raw_args(&self) -> *mut RawArgs { self.raw }

    pub fn cc(&self) -> ffi::Struct_PP_CompletionCallback { self.cc }

    pub fn drop_with_code(self, code: Code) -> Code<Args> {
        if code.completion_pending() { return code.map_err(); }

        let optional = self.cc.flags as u32 & ffi::PP_COMPLETIONCALLBACK_FLAG_OPTIONAL != 0;
        let blocking = self.cc.func.is_none();
        if code.is_ok() && (optional || blocking) {
            // the call completed synchronously.

            // CallbackArgsCompletion always has user_data
            debug_assert!(self.cc.user_data as usize != 0);

            let box CallbackArgsStorage {
                args, mapper: StorageToArgsMapper(mapper), f: _f,
            }: Box<CallbackArgsStorage<RawArgs, Args, F>> = unsafe {
                mem::transmute(self.cc.user_data)
            };
            Code::Ok(mapper(args, code))
        } else {
            // if this is taken, `code` is an error.
            // TODO: return provided data back to caller on errors

            if self.cc.user_data as usize != 0 {
                // clean up.
                let _: Box<CallbackArgsStorage<RawArgs, Args, F>> =
                    unsafe { mem::transmute(self.cc.user_data) };
            }
            code.map_err()
        }
    }
}

impl<F, Args, RawArgs> ops::Deref for CallbackArgsCompletion<F, Args, RawArgs> {
    type Target = RawArgs;
    fn deref(&self) -> &RawArgs { unsafe { mem::transmute(self.raw) } }
}
impl<F, Args, RawArgs> ops::DerefMut for CallbackArgsCompletion<F, Args, RawArgs> {
    fn deref_mut(&mut self) -> &mut RawArgs { unsafe { mem::transmute(self.raw) } }
}

/// A completion callback that has arguments which PPAPI writes to before
/// calling.
pub struct CallbackArgs<F: FnOnce(Code<OutArgs>), OutArgs>
{
    pub optional: bool,
    f: F,
    _1: PhantomData<OutArgs>,
}
impl<F, OutArgs> CallbackArgs<F, OutArgs>
    where F: FnOnce(Code<OutArgs>),
{
    pub fn new(f: F) -> CallbackArgs<F, OutArgs> {
        CallbackArgs {
            optional: false,
            f: f,
            _1: PhantomData,
        }
    }
    pub fn optional(f: F) -> CallbackArgs<F, OutArgs> {
        CallbackArgs {
            optional: true,
            f: f,
            _1: PhantomData,
        }
    }

    pub fn set_optional(&mut self, opt: bool) {
        self.optional = opt;
    }

    pub fn to_ffi_callback<RawArgs>(self, args: RawArgs,
                                    args_mapper: StorageToArgsMapper<RawArgs, OutArgs>) ->
        CallbackArgsCompletion<F, OutArgs, RawArgs>
        where RawArgs: InPlaceInit
    {
        extern "C" fn work_callback<F, RawArgs, OutArgs>(user: *mut libc::c_void,
                                                         status: i32)
            where F: FnOnce(Code<OutArgs>),
        {
            let box CallbackArgsStorage {
                args, mapper: StorageToArgsMapper(mapper), f,
            }: Box<CallbackArgsStorage<RawArgs, OutArgs, F>> = unsafe {
                mem::transmute(user)
            };

            let code = Code::from_i32(status);
            let code = code
                .map_ok(|status| {
                    mapper(args, Code::Ok(status))
                });
            f.call_once((code, ));
        }
        let optional = self.optional;
        let mut store = box CallbackArgsStorage {
            args: args, mapper: args_mapper, f: self.f,
        };
        store.args.inplace_init();

        let args: *mut RawArgs = &mut store.args as *mut _;

        let mut cc = unsafe {
            ffi::make_completion_callback(Some(work_callback::<F, RawArgs, OutArgs>),
                                          mem::transmute(store))
        };

        if optional {
            cc.flags |= ffi::PP_COMPLETIONCALLBACK_FLAG_OPTIONAL as i32;
        }

        CallbackArgsCompletion {
            raw: args,
            cc: cc,
            _1: PhantomData,
        }
    }

    pub fn call_directly(self, args: Code<OutArgs>) {
        self.f.call_once((args,))
    }
}

// This avoids an allocation.
struct InternalCallbacksOperatorFn(fn());
impl Callback for InternalCallbacksOperatorFn {
    type Fun = fn();
    fn to_ffi_callback(self) -> CallbackCompletion<fn()> {
        extern "C" fn work_callback(user: *mut libc::c_void, _status: i32) {
            let work: fn() = unsafe {
                mem::transmute(user)
            };
            work();
        }
        let InternalCallbacksOperatorFn(work) = self;
        let cc = unsafe {
            ffi::make_completion_callback(Some(work_callback),
                                          mem::transmute(work))
        };
        CallbackCompletion {
            cc: cc,
            _1: PhantomData,
        }
    }
}

struct BlockUntilComplete;
impl Callback for BlockUntilComplete {
    type Fun = ();
    fn to_ffi_callback(self) -> CallbackCompletion<()> {
        let cc = unsafe {
            ffi::block_until_complete()
        };
        CallbackCompletion {
            cc: cc,
            _1: PhantomData,
        }
    }
}
impl BlockUntilComplete {
    fn new() -> ffi::Struct_PP_CompletionCallback {
        unsafe {
            ffi::block_until_complete()
        }
    }
}

struct ConsoleLogger {
    filter_levels: Option<HashMap<Instance, log::LogLevelFilter>>,
    current_filter: log::MaxLogLevelFilter,
}
impl ConsoleLogger {
    fn new(filter: log::MaxLogLevelFilter) -> ConsoleLogger {
        ConsoleLogger {
            filter_levels: None,
            current_filter: filter,
        }
    }
    fn console(&self) -> Option<Console> {
        Instance::opt_current()
            .map(|c| c.console() )
    }
    fn current_instance_filter_level(&self) -> log::LogLevelFilter {
        Instance::opt_current()
            .and_then(|instance| {
                self.filter_levels
                    .as_ref()
                    .map(move |l| (l, instance) )
            })
            .and_then(|(levels, instance)| {
                levels.get(&instance)
            })
            .map(|&filter| filter )
            .unwrap_or(log::LogLevelFilter::Info)
    }
}
impl log::Log for ConsoleLogger {
    fn enabled(&self, md: &log::LogMetadata) -> bool {
        let level = self.current_instance_filter_level()
            .to_log_level();
        if let Some(level) = level {
            md.level() >= level
        } else {
            false
        }
    }
    fn log(&self, record: &LogRecord) {
        use self::ppb::ConsoleInterface;
        use log::LogLevel;
        let level = match record.level() {
            LogLevel::Error => ffi::PP_LOGLEVEL_ERROR,
            LogLevel::Warn  => ffi::PP_LOGLEVEL_WARNING,
            LogLevel::Info  => ffi::PP_LOGLEVEL_TIP,
            _               => ffi::PP_LOGLEVEL_LOG,
        };

        let loc = record.location();

        let str = format!("{} ({}:{}): {}",
                          loc.module_path(),
                          loc.file(),
                          loc.line(),
                          record.args())
            .to_string_var();
        match self.console() {
            Some(console) => console.log(level, str),
            None => {},
        }
    }
}

scoped_thread_local!(static CURRENT_INSTANCE: Instance);
static mut FIRST_INSTANCE: Option<Instance> = None;

pub fn is_main_thread() -> bool {
    Some(MessageLoop::get_main_loop()) == MessageLoop::current()
}

#[derive(Clone, Hash, Eq, PartialEq, Copy)]
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
        if CURRENT_INSTANCE.is_set() {
            Some(CURRENT_INSTANCE.with(|i| i.clone() ))
        } else { None }
    }
    fn check_current(&self) {
        assert!(Instance::current() == *self);
    }
    fn unwrap(&self) -> ffi::PP_Instance {
        self.instance
    }

    pub fn initialize_nacl_io(&self) {
        unsafe {
            ffi::nacl_io_init_ppapi(self.instance,
                                    Some(ppb::get_actual_browser()));
        }
    }

    pub fn console(&self) -> Console {
        return Console(self.instance);
    }

    pub fn messaging(&self) -> Messaging {
        return Messaging(self.instance);
    }

    pub fn create_3d_context(&self,
                             share_with: Option<gles::Context3d>,
                             attribs: &[gles::Context3dAttrib]) -> result::Result<gles::Context3d, Code> {
        let mut a = Vec::with_capacity(attribs.len() + 1);
        let attrs_to_ffi = attribs
            .iter()
            .map(|attr| attr.to_ffi() );
        a.extend(attrs_to_ffi);

        // only one is needed; le shurg.
        a.push((ffi::PP_GRAPHICS3DATTRIB_NONE,
                ffi::PP_GRAPHICS3DATTRIB_NONE));
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
            result::Result::Err(Code::Failed)
        } else {
            result::Result::Ok(gles::Context3d::new(raw_cxt))
        }
    }
    pub fn bind_context<T: ContextResource>(&self, cxt: &T) -> Code {
        match (ppb::get_instance().BindGraphics.unwrap())
            (self.instance,
             cxt.get_device()) {
            ffi::PP_TRUE => Code::Ok(0),
            ffi::PP_FALSE => Code::Failed,
            other => {
                error!("unknown truthy value: {:}", other);
                Code::Failed
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
                        init_to_zero: bool) -> Option<imagedata::ImageData> {
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
        res.map(|res| imagedata::ImageData::new(res) )
    }

    pub fn create_font(&self,
                       desc: &font::Description) -> Option<font::Font> {
        let f = ppb::get_font().Create.unwrap();
        let desc = unsafe { desc.to_ffi() };
        let res = f(self.instance, &desc as *const ffi::Struct_PP_FontDescription_Dev);
        if res != 0 {
            Some(font::Font::new(res))
        } else {
            None
        }
    }

    pub fn create_message_loop(&self) -> MessageLoop {
        MessageLoop(ppb::get_message_loop().create(&self.unwrap()))
    }

    /// Creates a new message loop and runs it inside a new thread. Once
    /// instance local data is added, this will ensure the new thread has
    /// access. This does not require the message loop to be shutdown until you
    /// return from `thread_local_setup`. FYI, `MessageLoop::is_attached()` will
    /// return false only after the queue is shutdown.
    pub fn spawn_message_loop<F>(&self,
                                 thread_local_setup: F) -> (MessageLoop, ::std::thread::JoinHandle<()>)
        where F: FnOnce(fn() -> Code) + Send + 'static,
    {
        fn run_loop() -> Code {
            MessageLoop::current()
                .unwrap()
                .run_loop()
        }
        let msg_loop = self.create_message_loop();
        let msg_loop2 = msg_loop.clone();
        let instance = self.clone();
        let join = ::std::thread::spawn(move || {
            msg_loop.attach_to_current_thread()
                .unwrap();
            CURRENT_INSTANCE.set(&instance, || {
                thread_local_setup(run_loop)
            });

            assert!(!MessageLoop::is_attached(),
                    "please stop (or shutdown) loop");
        });
        (msg_loop2, join)
    }

    pub fn create_file_system(&self, kind: fs::Kind) -> Option<fs::FileSystem> {
        use ppb::FileSystemIf;
        ppb::get_file_system().create(self.unwrap(),
                                      kind as ffi::PP_FileSystemType)
            .map(|fs| fs::FileSystem::new(fs) )
    }

    pub fn create_video_decoder(&self) -> Option<video_decoder::VideoDecoder> {
        use ppb::{get_video_decoder_opt, VideoDecoderIf};
        get_video_decoder_opt()
            .and_then(|i| i.create(self.unwrap()) )
            .map(|r| From::from(r) )
    }

    pub fn post_message<T: ToVar>(&self, msg: T) {
        self.messaging().post(msg);
    }
}

impl MessageLoop {
    fn get_ref<'a>(&'a self) -> &'a MessageLoop {
        self
    }

    fn on_destroy(&self) {
        fn work(_: Code<()>) {
            unsafe {
                ppapi_instance_destroyed();
            }
        }
        self.get_ref()
            .post_work(work, 0)
            .expect("couldn't tell an instance to shutdown");
        self.get_ref().queue_shutdown().expect("message loop shutdown failed");
    }

    fn on_change_view(&self, view: View) {
        self.get_ref()
            .post_work(move |_| {
                unsafe {
                    assert!(!ppapi_on_change_view.is_null());
                    let on_change_view: fn(View) =
                        transmute(ppapi_on_change_view);
                    on_change_view(view);
                }
            },
                       0)
            .expect("couldn't tell an instance about an on_change_view event");
    }
    fn on_change_focus(&self, has_focus: bool) {
        self.get_ref()
            .post_work(move |_| {
                unsafe {
                    assert!(!ppapi_on_change_focus.is_null());
                    let on_change_focus: fn(bool) =
                        transmute(ppapi_on_change_focus);
                    on_change_focus(has_focus);
                }
            },
                       0)
            .expect("couldn't tell an instance about an on_change_focus event");
    }
    fn on_document_load(&self, loader: http::Loader) -> bool {
        use std::sync::mpsc::channel;
        let (tx, rx) = channel();
        self.get_ref()
            .post_work(move |_| {
                unsafe {
                    assert!(!ppapi_on_document_loaded.is_null());
                    let on_document_loaded: fn(http::Loader) -> bool =
                        transmute(ppapi_on_document_loaded);

                    let handled = on_document_loaded(loader);
                    let _ = tx.send(handled);
                }
            },
                       0)
            .expect("couldn't tell an instance about an on_change_view event");

        // This will block forever if the recieving instance isn't responding to new messages.
        rx.try_recv().unwrap_or(false)
    }
}

type InstancesType = Vec<(Instance, MessageLoop)>;

// THIS MAY ONLY BE ACCESSED FROM THE MAIN MODULE THREAD.
static mut INSTANCES: *mut InstancesType = 0 as *mut InstancesType;

#[allow(deprecated)]
unsafe fn deinitialize_instances() {
    if !INSTANCES.is_null() {
        let _: Box<InstancesType> = mem::transmute(INSTANCES);
        INSTANCES = 0 as *mut _;
    }
}

fn expect_instances() -> &'static mut InstancesType {
    unsafe {
        if INSTANCES.is_null() {
            let instances: InstancesType = Default::default();
            let instances = Box::new(instances);
            INSTANCES = mem::transmute(instances);
            expect_instances()
        } else {
            mem::transmute(INSTANCES)
        }
    }
}

fn find_instance<U, Take, F>(instance: Instance,
                             take: Take,
                             f: F) -> Option<U>
    where F: FnOnce(&MessageLoop, Take) -> U
{
    let mut v = None;
    for &(ref inst, ref msg) in expect_instances().iter() {
        if *inst == instance {
            v = Some(msg);
            break;
        }
    }
    match v {
        Some(msg) => Some(f(msg, take)),
        None => {
            // TODO: better message/moar infos.
            error!("Instance not found");
            None
        },
    }
}

fn remove_instance(instance: Instance) -> Option<MessageLoop> {
    let mut index = 0;
    for &(ref inst, _) in expect_instances().iter() {
        if *inst == instance {
            let (_, msg) = expect_instances()
                .remove(index);
            return Some(msg);
        }
        index += 1;
    }
    None
}
fn insert_instance(instance: Instance, msg: MessageLoop) {
    debug_assert!({
        let mut found = false;
        for &(ref inst, _) in expect_instances().iter() {
            if *inst == instance {
                found = true;
                break;
            }
        }
        !found
    });

    expect_instances()
        .push((instance, msg));
}
#[doc(hidden)]
pub mod entry {
    use super::{find_instance, insert_instance,
                remove_instance, CURRENT_INSTANCE};
    use super::{Code, Instance, View, ToFFIBool, GenericResource,
                Resource};
    use super::{ffi};

    use libc::c_char;
    use std::any::Any;
    use std::mem::transmute;

    // We need to catch all failures in our callbacks,
    // lest an exception (failure) in one instance terminates all
    // instances and crashes the whole plugin.
    pub fn try_block<U: Send, F: FnOnce() -> U + Send + 'static>(f: F) ->
        Result<U, Box<Any + Send>>
    {
        use std::thread::catch_panic;

        let result = catch_panic(f);
        // if we're unwinding, the instance had a failure, and we need
        // to destory the instance.
        // Note that this can be called before an instance is ever inserted
        // into the global store.
        if result.is_err() {
            match Instance::opt_current() {
                Some(inst) => {
                    let _ = remove_instance(inst)
                        .map(|msg| msg.queue_shutdown() );
                }
                _ => {}
            }
        }
        result
    }

    pub extern "C" fn did_create(inst: ffi::PP_Instance,
                                 argc: u32,
                                 argk: *mut *const c_char,
                                 argv: *mut *const c_char) -> ffi::PP_Bool {
        use std::thread::{Builder, catch_panic};
        use std::sync::mpsc::channel;
        use super::{MessageLoop};

        fn parse_args(args: Args, id: &mut Option<String>) -> Vec<(String, String)> {
            use libc::strlen;
            use std::slice::from_raw_parts;
            use std::str::from_utf8_unchecked;

            let Args(argc, argk, argv) = args;

            let mut dest = Vec::with_capacity(argc);

            for i in (0..argc) {
                unsafe {
                    let ak = *argk.offset(i as isize);
                    let av = *argv.offset(i as isize);

                    assert!(ak as usize != 0); assert!(av as usize != 0);

                    let ak_len = strlen(ak);
                    let av_len = strlen(av);
                    let ak_slice = from_raw_parts(ak as *const u8, ak_len as usize);
                    let av_slice = from_raw_parts(av as *const u8, av_len as usize);
                    let ak_str = from_utf8_unchecked(ak_slice);
                    let av_str = from_utf8_unchecked(av_slice);

                    if ak_str == "id" {
                        *id = Some(ak_str.to_string());
                    }

                    dest.push((ak_str.to_string(), av_str.to_string()));
                }
            }

            dest
        }

        struct Args(usize, *mut *const c_char, *mut *const c_char);
        unsafe impl Send for Args { }

        let args = Args(argc as usize, argk, argv);

        let instance = Instance::new(inst);
        // Dat nesting.
        let success = try_block(move || {
            let mut id = None;
            let args = parse_args(args, &mut id);

            let builder = Builder::new()
                .stack_size(0);
            let builder = if let Some(id) = id {
                builder.name(id)
            } else {
                builder.name(format!("instance id `{}`", inst))
            };

            let (tx, rx) = channel();

            let _ = builder.spawn(move || {
                let mut args = Some(args.clone());
                CURRENT_INSTANCE.set
                    (&instance,
                     || {
                         let ml = instance.create_message_loop();
                         match ml.attach_to_current_thread() {
                             Code::Ok(_) => {}
                             _ => {
                                 error!("failed to attach the new instance's message loop");
                                 let _ = tx.send(None);
                                 return;
                             }
                         }

                         fn unwinding() -> bool {
                             use std::thread;
                             thread::panicking()
                         }


                         let res = {
                             let i = instance.clone();
                             let a = args.take().unwrap();
                             catch_panic(move || unsafe {
                                 super::ppapi_instance_created(i, a)
                             })
                         };

                         match res {
                             Ok(()) => {
                                 tx.send(Some(ml.clone())).unwrap();
                             },
                             Err(..) => {
                                 error!("failed to initialize instance");
                                 tx.send(None).unwrap();
                             },
                         }

                         // TODO log errors.
                         let _ = catch_panic(move || ml.run_loop() );

                         if MessageLoop::is_attached() {
                             panic!("please shutdown the loop; I may add pausing \
                                     for some sort of pattern later");
                         } else {
                             let cb = move |_| {
                                 remove_instance(instance);
                             };
                             MessageLoop::get_main_loop()
                                 .post_work(cb, 0)
                                 .unwrap();
                         }
                     });
            });

            rx.recv()
                .ok()
                .and_then(|ml| ml )
                .map(|ml: MessageLoop| {
                    insert_instance(instance, ml);
                    true
                })
                .unwrap_or(false)
        }) // try_block
            .unwrap();
        success.to_ffi_bool()
    }
    pub extern "C" fn did_destroy(inst: ffi::PP_Instance) {
        let instance = Instance::new(inst);

        CURRENT_INSTANCE.set
            (&instance,
             || {
                 let _ = try_block(move || {
                     debug!("did_destroy");

                     let store = remove_instance(instance);
                     if let Some(store) = store {
                         store.on_destroy();
                     }
                 });
             });

    }
    pub extern "C" fn did_change_view(inst: ffi::PP_Instance, view: ffi::PP_Resource) {
        let instance = Instance::new(inst);

        CURRENT_INSTANCE.set
            (&instance,
             || {
                 if !super::ppapi_on_change_view.is_null() {
                     let _ = try_block(move || {
                         debug!("did_change_view");
                         find_instance(instance,
                                       view,
                                       |store, view| {
                                           ::std::mem::forget(View::new(view));
                                           store.on_change_view(View::new(view))
                                       });
                     });
                 } else {
                     warn!("plugin is missing 'ppapi_on_change_view'");
                 }
             })
    }
    pub extern "C" fn did_change_focus(inst: ffi::PP_Instance, has_focus: ffi::PP_Bool) {
        let instance = Instance::new(inst);

        CURRENT_INSTANCE.set
            (&instance,
             || {
                 if !super::ppapi_on_change_focus.is_null() {
                     let _ = try_block(move || {
                         debug!("did_change_focus");

                         find_instance(instance,
                                       (),
                                       |store, ()| store.on_change_focus(has_focus != ffi::PP_FALSE) );
                     });
                 } else {
                     warn!("plugin is missing 'ppapi_on_change_focus'");
                 }
             });
    }
    pub extern "C" fn handle_document_load(inst: ffi::PP_Instance,
                                           url_loader: ffi::PP_Resource) -> ffi::PP_Bool {
        let instance = Instance::new(inst);

        let handled = CURRENT_INSTANCE.set
            (&instance,
             move || {
                 if super::ppapi_on_document_loaded.is_null() {
                     warn!("plugin is missing 'ppapi_on_document_loaded'");
                     return false;
                 }

                 let handled = try_block(move || {
                     debug!("handle_document_load");

                     find_instance(instance,
                                   From::from(url_loader),
                                   |store, url_loader: GenericResource| {
                                       ::std::mem::forget(url_loader.clone());
                                       let loader = From::from(url_loader.unwrap());
                                       store.on_document_load(loader)
                                   }).unwrap_or(false)
                 }).ok().unwrap_or(false);
                 handled
             });
        return handled.to_ffi_bool();
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

        let handled;
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
                panic!("unknown input event");
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

#[allow(improper_ctypes)]
extern {
    #[no_mangle]
    fn ppapi_instance_created(instance: Instance,
                              args: Vec<(String, String)>);
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
    static ppapi_on_input: *const libc::c_void;

    #[no_mangle]
    #[linkage = "extern_weak"]
    static ppapi_on_graphics_context_lost: *const libc::c_void;
}

#[cfg(test)]
mod test {
    #![allow(private_no_mangle_fns)]
    use super::Instance;
    use std::collections::HashMap;
    #[no_mangle]
    extern fn ppapi_instance_created(_instance: Instance,
                                     _args: Vec<(String, String)>) {
    }
    #[no_mangle]
    extern fn ppapi_instance_destroyed() {
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[doc(hidden)]
pub extern "C" fn PPP_InitializeModule(modu: ffi::PP_Module,
                                       gbi: ffi::PPB_GetInterface) -> libc::int32_t {
    use self::entry::try_block;
    use log::set_logger;

    // We can't fail! before this block!
    let result = try_block(move || {
        pp::initialize_globals(modu);
        ppb::initialize_globals(gbi);
    });

    match result {
        result::Result::Ok(()) => {
            set_logger(move |f| box ConsoleLogger::new(f) )
                .unwrap();

            ffi::PP_OK
        }
        result::Result::Err(_) => {
            // Nb: this gets printed to chrome's stdout if it is running on a console.
            // Otherwise it falls into a black hole and is eaten.
            println!("module initialization failed");
            1i32
        }
    }
}
#[no_mangle]
#[allow(non_snake_case)]
#[doc(hidden)]
pub extern "C" fn PPP_ShutdownModule() {
    use self::entry::try_block;
    let _ = try_block(|| { unsafe {
        deinitialize_instances();
    }} );
}
