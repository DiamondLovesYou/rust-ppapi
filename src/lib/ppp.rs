// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::c_str;
use std::ptr;
use libc::{c_char, c_void};

mod consts {
    pub static INSTANCE: &'static str = "PPP_Instance;1.1";
    pub static MESSAGING: &'static str = "PPP_Messaging;1.0";
    pub static INPUTEVENT: &'static str = "PPP_InputEvent;0.1";
    pub static GRAPHICS: &'static str = "PPP_Graphics_3D;1.0";
}
mod globals {
    use super::super::entry;
    use super::super::ffi;
    pub const INSTANCE: ffi::Struct_PPP_Instance_1_1 = ffi::Struct_PPP_Instance_1_1 {
        DidCreate: Some(entry::did_create as extern "C" fn(i32, u32, *mut *const i8, *mut *const i8) -> u32),
        DidDestroy: Some(entry::did_destroy as extern "C" fn(i32)),
        DidChangeView: Some(entry::did_change_view as extern "C" fn(i32, i32)),
        DidChangeFocus: Some(entry::did_change_focus as extern "C" fn(i32, u32)),
        HandleDocumentLoad: Some(entry::handle_document_load as extern "C" fn(i32, i32) -> u32),
    };
    pub static MESSAGING: ffi::Struct_PPP_Messaging_1_0 = ffi::Struct_PPP_Messaging_1_0 {
        HandleMessage: Some(entry::handle_message as extern "C" fn(i32, ffi::Struct_PP_Var)),
    };
    pub static INPUTEVENT: ffi::Struct_PPP_InputEvent_0_1 = ffi::Struct_PPP_InputEvent_0_1 {
        HandleInputEvent: Some(entry::handle_input_event as extern "C" fn(i32, i32) -> u32),
    };
    pub static GRAPHICS: ffi::Struct_PPP_Graphics3D_1_0 = ffi::Struct_PPP_Graphics3D_1_0 {
        Graphics3DContextLost: Some(entry::graphics_context_lost as extern "C" fn(i32)),
    };
}

#[no_mangle]
#[allow(dead_code)]
#[doc(hidden)]
#[allow(non_snake_case)]
pub extern "C" fn PPP_GetInterface(interface_name: *const c_char) -> *const c_void {
    use core::mem::transmute;
    unsafe {
        let c_name = c_str::CString::new(interface_name, false);
        let name = c_name.as_str().expect("Naughty browser");
        if name == consts::INSTANCE {
            transmute(&globals::INSTANCE)
        } else if name == consts::MESSAGING {
            transmute(&globals::MESSAGING)
        } else if name == consts::INPUTEVENT {
            transmute(&globals::INPUTEVENT)
        } else if name == consts::GRAPHICS {
            transmute(&globals::GRAPHICS)
        } else {
            warn!("PPAPI requested unknown interface: `{}`",
                  name);
            ptr::null()
        }
    }
}
