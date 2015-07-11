// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A WIP async file IO implementation. This works both in Pepper plugins and in
/// regular unsandboxed applications. For non-pepper mode, this probably isn't
/// the most speedy implementation. However, the API is designed to allow
/// incredibly fast implementations in the future.

use super::Result;

pub use self::impl_::*;

pub trait Read {
    fn read<F>(&mut self, size: usize, callback: F) -> Result<()>
        where F: FnOnce(Result<&[u8]>);
}
pub trait Write {
    fn write<F>(&mut self, buffer: Vec<u8>, callback: F) -> Result<()>
        where F: FnOnce(Result<()>);
    fn flush<F>(&mut self, callback: F) -> Result<()>
        where F: FnOnce(Result<()>);
}


#[cfg(feature = "pepper")]
mod impl_ {
    use ffi;
    use ppb;
    use ppb::FileSystemIf;
    use super::super::{Result, Callback, Code, Resource};

    #[derive(Hash, Eq, PartialEq, Debug)] pub struct FileSystem(ffi::PP_Resource);
    #[derive(Hash, Eq, PartialEq, Debug)] pub struct FileRef(ffi::PP_Resource);

    #[derive(Clone, Hash, Eq, PartialEq, Debug)]
    pub struct FileSliceRef(pub FileRef,
                            pub Option<i64>,
                            pub Option<i64>);
    #[derive(Hash, Eq, PartialEq, Debug)]
    pub struct FileIo(ffi::PP_Resource);

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

        /// This must be called before doing any operation on the FS.
        pub fn open<F>(&self, expected_size: usize, callback: F) -> Result<()>
            where F: Callback
        {
            let code = ppb::get_file_system().open(self.unwrap(), expected_size as i64, callback.to_ffi_callback());
            let code = Code::from_i32(code);
            code.to_empty_result()
        }
    }
}

#[cfg(not(feature = "pepper"))]
mod impl_ {
    //! This is implementation based on std's synchronous IO facilities.
    //! Every FileSystem has its own IO thread.
    use std::io;
    use std::fs::File;
    use std::sync::mpsc;

    fn to_code_result<T>(res: io::Result<T>) -> super::super::Result<T> {
        res.map_err(|err| {
            match err.kind() {
                ErrorKind::FileNotFound => Code::FileNotFound,
                ErrorKind::PermissionDenied => Code::NoAccess,
                ErrorKind::ConnectionRefused => Code::ConnectionRefused,
                ErrorKind::ConnectionReset => Code::ConnectionReset,
                ErrorKind::ConnectionAborted => Code::ConnectionAborted,
                ErrorKind::NotConnected => Code::ConnectionClosed,
                ErrorKind::BrokenPipe => Code::Failed,
                ErrorKind::PathAlreadyExists => Code::FileNotFound,
                ErrorKind::PathDoesntExist => Code::FileExists,
                ErrorKind::MismatchedFileTypeForOperation => Code::FileNotFound,
                ErrorKind::ResourceUnavailable => Code::BadResource,
                ErrorKind::InvalidInput => Code::BadArgument,
                ErrorKind::TimedOut => Code::TimedOut,
                ErrorKind::WriteZero => Code::BadArgument,
                ErrorKind::Interrupted => Code::FileNotFound,
                ErrorKind::Other => Code::Failed,
            }
        })
    }

    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Kind {
        External,
        LocalPersistent,
        LocalTemp,
        Isolated,
    }

    type FId = u64;

    enum Message {
        OpenFileRef(Sender<io::Result<FId>>),

        RefFS,
        RefFileRef(FId),
        RefFileIo(FId),
        DerefFS,
    }

    fn fs_thread(recv: mpsc::Receiver<Message>) {

    }

    struct FSThreadFileRef {
        /// Despite the use of Path, this must be in valid utf8.
        path: Path,
        refs: usize,
    }
    impl FSThreadFileRef {
        fn add_ref(&mut self) {
            self.refs += 1;
        }
        /// Returns true if this instance should be dropped. Doesn't modify the
        /// ref count when it returns true.
        fn sub_ref(&mut self) -> bool {
            if self.refs == 1 {
                true
            } else {
                self.refs -= 1;
                false
            }
        }
    }

    struct FSThreadFileIo {
        f: File,
        refs: usize,
    }
    impl FSThreadFileIo {
        fn add_ref(&mut self) {
            self.refs += 1;
        }
        /// Returns true if this instance should be dropped. Doesn't modify the
        /// ref count when it returns true.
        fn sub_ref(&mut self) -> bool {
            if self.refs == 1 {
                true
            } else {
                self.refs -= 1;
                false
            }
        }

    }

    struct FSThread {
        refs: usize,
        root: Path,
        recv: mpsc::Receiver<Message>,

        /// These paths must be valid utf8:
        file_refs: HashMap<FId, FSThreadFileRef>,

        file_ios:  HashMap<FId, File>,
    }

    impl FSThread {
        fn spawn(recv: mpsc::Receiver<Message>, root: Path) {
            use std::thread;

            let mut t = FSThread {
                refs: 1,
                root: Path,
                recv: recv,

                file_refs: HashMap::new(),
                file_ios:  HashMap::new(),
            };
            thread::spawn(move || {
                let mut t = t;
                t.run();
            })
        }

        fn run(&mut self) {

        }
    }

    pub struct FileSystem {
        kind: Kind,
        chan: mpsc::Sender<Message>,
    }

    impl FileSystem {
        pub fn new(instance: &super::super::Instance, kind: Kind) ->
            Result<FileSystem>
        {

        }
        pub fn create_file_ref(&self, path: &Path) -> Result<FileRef> {

        }
        pub fn filesystem_kind(&self) -> Kind {
            self.kind
        }
    }

    pub struct FileRef {
        fs: FileSystem,
        id: FId,
    }

    pub struct FileRef;
}
