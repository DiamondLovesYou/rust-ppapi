// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::ffi;

mod globals {
    use super::super::ffi;

    pub static mut MODULE: ffi::PP_Module = 0 as ffi::PP_Module;
}
// Initialize the global module handle.
pub fn initialize_globals(module: ffi::PP_Module) {
    unsafe {
        globals::MODULE = module;
    }
}

// Return a clone of the module handle.
pub fn get_module() -> ffi::PP_Module {
    unsafe {
        globals::MODULE
    }
}
