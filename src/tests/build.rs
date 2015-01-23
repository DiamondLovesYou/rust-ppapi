// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(unstable)]

extern crate "pnacl-build-helper" as helper;
extern crate libc;

use std::io::{Command};
use std::io::process::InheritFd;
use std::os::{change_dir, getcwd, getenv};


pub fn main() {
    let ppapi_root = getcwd().unwrap()
        .join_many(["..", ".."].as_slice());
    change_dir(&ppapi_root).unwrap();

    let target = getenv("PPAPI_TESTS_TARGET")
        .unwrap_or_else(move |:| "le32-unknown-nacl".to_string() );
    let env_target = target.replace("-", "_");

    let tools: helper::NativeTools = helper::NativeTools::new(target.as_slice());
    let envs = [(format!("CC_{}", env_target),
                 tools.cc.display().to_string()),
                (format!("CXX_{}", env_target),
                 tools.cxx.display().to_string()),
                (format!("AR_{}", env_target),
                 tools.ar.display().to_string()),
                (format!("RANLIB_{}", env_target),
                 tools.ranlib.display().to_string()),
                ];

    let mut cargo = Command::new("cargo");
    cargo.args(["build", "--verbose", "--target"].as_slice());
    cargo.arg(target);
    for &(ref k, ref v) in envs.iter() {
        cargo.env(k, v);
    }
    cargo.stdout(InheritFd(libc::STDOUT_FILENO));
    cargo.stderr(InheritFd(libc::STDERR_FILENO));
    println!("spawning `{:?}`:", cargo);
    let mut cargo = cargo.spawn().unwrap();
    assert!(cargo.wait().unwrap().success());
}
