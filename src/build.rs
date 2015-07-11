// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate pnacl_build_helper as helper;

pub fn main() {
    let mut a = helper::Archive::new("helper");
    a.cxx("src/libhelper/helper.cpp", &["-Os".to_string()]);
    a.archive();
    helper::print_lib_paths();
}
