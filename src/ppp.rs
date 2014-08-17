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
    pub static INSTANCE: ffi::Struct_PPP_Instance_1_1 = ffi::Struct_PPP_Instance_1_1 {
        DidCreate: Some(entry::did_create),
        DidDestroy: Some(entry::did_destroy),
        DidChangeView: Some(entry::did_change_view),
        DidChangeFocus: Some(entry::did_change_focus),
        HandleDocumentLoad: Some(entry::handle_document_load),
    };
    pub static MESSAGING: ffi::Struct_PPP_Messaging_1_0 = ffi::Struct_PPP_Messaging_1_0 {
        HandleMessage: Some(entry::handle_message),
    };
    pub static INPUTEVENT: ffi::Struct_PPP_InputEvent_0_1 = ffi::Struct_PPP_InputEvent_0_1 {
        HandleInputEvent: Some(entry::handle_input_event),
    };
    pub static GRAPHICS: ffi::Struct_PPP_Graphics3D_1_0 = ffi::Struct_PPP_Graphics3D_1_0 {
        Graphics3DContextLost: Some(entry::graphics_context_lost),
    };
}

#[no_mangle]
#[allow(dead_code)]
#[doc(hidden)]
#[allow(non_snake_case_functions)]
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
