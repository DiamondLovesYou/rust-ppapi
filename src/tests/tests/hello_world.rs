// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(plugin)]

#[macro_use] #[plugin] #[no_link]
extern crate "ppapi-tester" as _ppapi_tester;

#[ppapi_test]
fn hello_world(instance: ppapi::Instance, args: HashMap<String, String>) {
    println!("Hello, world!");
}
