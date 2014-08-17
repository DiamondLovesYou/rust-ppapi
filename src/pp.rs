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
