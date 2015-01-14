// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ffi;
use ppb;
use ppb::FileSystemIf;
use super::{Result, Callback, Code, Resource};

#[derive(Hash, Eq, PartialEq, Show)] pub struct FileSystem(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Show)] pub struct FileRef(ffi::PP_Resource);

#[unstable]
#[derive(Clone, Hash, Eq, PartialEq, Show)]
pub struct FileSliceRef(pub FileRef,
                        pub Option<i64>,
                        pub Option<i64>);
#[derive(Hash, Eq, PartialEq, Show)] pub struct FileIo(ffi::PP_Resource);

impl_resource_for!(FileSystem, ResourceType::FileSystemRes);
impl_clone_drop_for!(FileSystem);
impl_resource_for!(FileRef, ResourceType::FileRefRes);
impl_clone_drop_for!(FileRef);
impl_resource_for!(FileIo, ResourceType::FileIoRes);
impl_clone_drop_for!(FileIo);

#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Kind {
    External = ffi::PP_FILESYSTEMTYPE_EXTERNAL,
    LocalPersistent = ffi::PP_FILESYSTEMTYPE_LOCALPERSISTENT,
    LocalTemp = ffi::PP_FILESYSTEMTYPE_LOCALTEMPORARY,
    Isolated = ffi::PP_FILESYSTEMTYPE_ISOLATED,
}

impl FileSystem {
    pub fn filesystem_kind(&self) -> Kind {
        use std::mem::transmute;
        unsafe { transmute(ppb::get_file_system().get_type(self.unwrap())) }
    }
    pub fn open<F>(&self, expected_size: usize, callback: F) -> Result<()>
        where F: Callback
    {
        let code = ppb::get_file_system().open(self.unwrap(), expected_size as i64, callback.to_ffi_callback());
        let code = Code::from_i32(code);
        code.to_empty_result()
    }
}
