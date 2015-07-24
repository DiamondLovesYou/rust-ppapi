// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A WIP async file IO implementation. This works/will work both in Pepper plugins and
/// regular unsandboxed applications. For non-pepper mode, this probably isn't
/// the most speedy implementation. However, the API is designed to allow
/// incredibly fast implementations in the future.

pub use self::common::*;
pub use self::impl_::*;

pub mod common {
    use std::borrow::Cow;

    use ffi;
    use super::super::{Callback, CallbackArgs, Code, Time, Resource};

    pub use ffi::Struct_PP_FileInfo as Info;

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
    #[repr(u32)]
    pub enum Type {
        Regular = ffi::PP_FILETYPE_REGULAR,
        Directory = ffi::PP_FILETYPE_DIRECTORY,
        Other = ffi::PP_FILETYPE_OTHER,
    }
    #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    pub struct DirectoryEntry {
        pub file: super::FileRef,
        pub ty: Type,
    }
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
    pub struct OpenFlags {
        read: bool,
        write: bool,
        create: bool,
        truncate: bool,
        exclusive: bool,
        append: bool,
    }
    impl OpenFlags {
        pub fn new() -> OpenFlags { Default::default() }

        pub fn read(mut self, read: bool) -> OpenFlags {
            self.read = read;
            self
        }
        pub fn write(mut self, write: bool) -> OpenFlags {
            self.append = false;
            self.write = write;
            self
        }
        pub fn create(mut self, create: bool) -> OpenFlags {
            self.create = create;
            self
        }
        pub fn truncate(mut self, trunc: bool) -> OpenFlags {
            self.truncate = trunc;
            self
        }
        pub fn exclusive(mut self, exclusive: bool) -> OpenFlags {
            self.exclusive = exclusive;
            self
        }
        pub fn append(mut self, append: bool) -> OpenFlags {
            if append {
                self.write = false;
            }
            self.append = append;
            self
        }
    }
    impl Default for OpenFlags {
        fn default() -> OpenFlags {
            OpenFlags {
                read: true,
                write: false,
                create: false,
                truncate: false,
                exclusive: false,
                append: false,
            }
        }
    }
    impl Into<i32> for OpenFlags {
        fn into(self) -> i32 {
            let mut flags = 0;
            if self.read { flags |= ffi::PP_FILEOPENFLAG_READ; }
            if self.write { flags |= ffi::PP_FILEOPENFLAG_WRITE; }
            if self.create { flags |= ffi::PP_FILEOPENFLAG_CREATE; }
            if self.truncate { flags |= ffi::PP_FILEOPENFLAG_TRUNCATE; }
            if self.exclusive { flags |= ffi::PP_FILEOPENFLAG_EXCLUSIVE; }
            if self.append { flags |= ffi::PP_FILEOPENFLAG_APPEND; }
            flags as i32
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
    pub struct MkDirFlags {
        ancestors: bool,
        exclusive: bool,
    }
    impl Default for MkDirFlags {
        fn default() -> MkDirFlags {
            MkDirFlags {
                // I realize this isn't normally the default, but it's never made
                // sence to why it isn't.
                ancestors: true,
                exclusive: false,
            }
        }
    }
    impl MkDirFlags {
        pub fn new() -> MkDirFlags { Default::default() }

        pub fn ancestors(mut self, ancestors: bool) -> MkDirFlags {
            self.ancestors = ancestors;
            self
        }
        pub fn exclusive(mut self, exclusive: bool) -> MkDirFlags {
            self.exclusive = exclusive;
            self
        }
    }
    impl Into<i32> for MkDirFlags {
        fn into(self) -> i32 {
            let mut flags = ffi::PP_MAKEDIRECTORYFLAG_NONE;
            if self.ancestors {
                flags |= ffi::PP_MAKEDIRECTORYFLAG_WITH_ANCESTORS;
            }
            if self.exclusive {
                flags |= ffi::PP_MAKEDIRECTORYFLAG_EXCLUSIVE;
            }
            flags as i32
        }
    }

    pub trait AsyncRead {
        fn async_read<'a, F>(&mut self, offset: u64, size: usize, callback: F) -> Code
            where F: CallbackArgs<Cow<'a, [u8]>>;
    }
    pub trait AsyncWrite {
        fn async_write<F>(&mut self, offset: u64, buffer: Vec<u8>, callback: F) -> Code
            where F: CallbackArgs<(usize, Vec<u8>)>;
        fn async_flush<F>(&mut self, callback: F) -> Code
            where F: Callback;
    }
    pub trait AsyncStream: AsyncRead + AsyncWrite { }


    // file/dir releated stuffs
    pub trait AsyncCommon {
        fn async_touch<F>(&self, atime: Time, mtime: Time, callback: F) -> Code
            where F: Callback;
        fn async_query<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Info>;
    }
    pub trait AsyncFile: AsyncCommon {
        fn async_set_len<F>(&mut self, len: u64, callback: F) -> Code
            where F: Callback;
    }
    pub trait AsyncPath: AsyncCommon {
        fn async_mkdir<F>(&self, flags: MkDirFlags, callback: F) -> Code
            where F: Callback;
        fn async_delete<F>(&self, callback: F) -> Code
            where F: Callback;
        fn async_rename<F>(&self, to: Self, callback: F) -> Code
            where F: Callback;
        fn async_read_directory_entries<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Vec<DirectoryEntry>>;

        fn async_open_io<F>(&self, instance: super::super::Instance,
                            flags: OpenFlags, callback: F) -> Code
            where F: CallbackArgs<super::FileIo>;
    }

    pub trait FileView: AsyncStream + AsyncCommon + Resource {
        type Target;

        fn file_io(&self) -> &super::FileIo;

        fn view(&self, from: Option<u64>, to: Option<u64>) -> Self::Target;
        fn view_full(&self) -> Self::Target { self.view(None, None) }

        fn view_start(&self) -> u64;
        fn view_stop (&self) -> Option<u64>;
        fn view_len  (&self) -> Option<u64> {
            self.view_stop()
                .map(|stop| stop - self.view_start() )
        }
    }
}


#[cfg(feature = "pepper")]
mod impl_ {
    use std::borrow::Cow;

    use libc;

    use ffi;
    use ppb::{FileSystemIf, FileRefIf, FileIoIf};
    use ppb::{get_file_system, get_file_ref, get_file_io};
    use super::common::{AsyncRead, AsyncWrite, AsyncFile, AsyncPath,
                        AsyncCommon, AsyncStream, Info, OpenFlags, MkDirFlags,
                        DirectoryEntry, FileView};
    use super::super::{Result, Callback, CallbackArgs, Code,
                       Resource, StorageToArgsMapper,
                       InPlaceArrayOutputStorage, Time,
                       BlockUntilComplete, ResourceType};

    use std::io::{self, Seek, Read, Write};

    #[derive(Hash, Eq, PartialEq, Debug)] pub struct FileSystem(ffi::PP_Resource);
    #[derive(Hash, Eq, PartialEq, Debug)] pub struct FileRef(ffi::PP_Resource);
    #[derive(Hash, Eq, PartialEq, Debug)] pub struct FileIo(ffi::PP_Resource);

    impl_resource_for!(FileSystem, ResourceType::FileSystem);
    impl_clone_drop_for!(FileSystem);
    impl_resource_for!(FileRef, ResourceType::FileRef);
    impl_clone_drop_for!(FileRef);
    impl_resource_for!(FileIo, ResourceType::FileIo);
    impl_clone_drop_for!(FileIo);

    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    pub struct SliceIo<T = FileIo>(T, u64, Option<u64>) where T: FileView;
    impl<T: FileView> Resource for SliceIo<T> {
        fn unwrap(&self) -> ffi::PP_Resource { self.file_io().unwrap() }
        fn type_of(&self) -> Option<ResourceType> { Some(ResourceType::FileIo) }
    }

    #[repr(u32)]
    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Kind {
        External = ffi::PP_FILESYSTEMTYPE_EXTERNAL,
        LocalPersistent = ffi::PP_FILESYSTEMTYPE_LOCALPERSISTENT,
        LocalTemp = ffi::PP_FILESYSTEMTYPE_LOCALTEMPORARY,
        Isolated = ffi::PP_FILESYSTEMTYPE_ISOLATED,
    }

    impl FileSystem {
        pub fn kind(&self) -> Kind {
            use std::mem::transmute;
            unsafe { transmute(get_file_system().get_type(self.unwrap())) }
        }

        /// This must be called before doing any operation on the FS.
        pub fn open<F>(&self, expected_size: usize, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_system().open(self.unwrap(), expected_size as i64,
                                              cc.cc);
            cc.drop_with_code(code)
        }

        pub fn create<T: ::std::fmt::Display>(&self, path: T) -> Option<FileRef> {
            let cstr = format!("{}\0", path);
            get_file_ref().create(self.unwrap(), cstr.as_ptr() as *const _)
                .map(|r| FileRef(r) )
        }
    }

    impl FileView for FileIo {
        type Target = SliceIo<FileIo>;
        fn file_io(&self) -> &FileIo { self }

        fn view(&self, from: Option<u64>, to: Option<u64>) -> SliceIo<FileIo> {
            match (from, to) {
                (Some(from), Some(to)) => assert!(from <= to),
                _ => (),
            }
            SliceIo(self.clone(), from.unwrap_or_default(), to)
        }

        fn view_start(&self) -> u64         { 0    }
        fn view_stop (&self) -> Option<u64> { None }
    }
    impl<T: FileView> FileView for SliceIo<T> {
        type Target = SliceIo<Self>;
        fn file_io(&self) -> &FileIo { self.0.file_io() }

        fn view(&self, from: Option<u64>, to: Option<u64>) -> SliceIo<Self> {
            match (from, to) {
                (Some(from), Some(to)) => assert!(from <= to),
                _ => (),
            }
            SliceIo(self.clone(), from.unwrap_or_default(), to)
        }

        fn view_start(&self) -> u64         { self.1 }
        fn view_stop (&self) -> Option<u64> { self.2 }
    }

    impl AsyncRead for FileIo {
        fn async_read<'a, F>(&mut self, offset: u64, size: usize, callback: F) -> Code
            where F: CallbackArgs<Cow<'a, [u8]>>
        {
            fn map_arg<'a>(raw: InPlaceArrayOutputStorage<u8>) -> Cow<'a, [u8]> {
                let v: Vec<_> = raw.into();
                Cow::Owned(v)
            }

            let raw_args: InPlaceArrayOutputStorage<u8> = Default::default();
            let mapper = StorageToArgsMapper::Take(map_arg);
            let mut cc = callback.to_ffi_callback(raw_args, mapper);
            let fficc = cc.cc;
            let code = get_file_io()
                .read_to_array(self.unwrap(), offset, size as u32,
                               cc.as_mut(), fficc);
            cc.drop_with_code(code)
        }
    }

    impl AsyncWrite for FileIo {
        fn async_write<F>(&mut self, offset: u64, buffer: Vec<u8>, callback: F) -> Code
            where F: CallbackArgs<(usize, Vec<u8>)>
        {
            impl super::super::InPlaceInit for (usize, Vec<u8>) { }

            fn map_arg(arg: (usize, Vec<u8>)) -> (usize, Vec<u8>) { arg }
            let mapper = StorageToArgsMapper::Take(map_arg);
            let mut cc = callback.to_ffi_callback((0, buffer), mapper);
            let written = get_file_io()
                .write(self.unwrap(), offset,
                       cc.1.as_ptr() as *const _,
                       cc.1.len() as libc::size_t,
                       cc.cc);
            match written {
                Err(code) => cc.drop_with_code(code),
                Ok(written) => {
                    cc.0 = written as usize;
                    cc.drop_with_code(Code::CompletionPending)
                },
            }
        }
        fn async_flush<F>(&mut self, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_io()
                .flush(self.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncStream for FileIo { }
    impl AsyncCommon for FileIo {
        fn async_touch<F>(&self, atime: Time, mtime: Time, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_io()
                .touch(self.unwrap(), atime, mtime, cc.cc);
            cc.drop_with_code(code)
        }
        fn async_query<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Info>
        {
            impl super::super::InPlaceInit for ffi::Struct_PP_FileInfo { }
            fn map_arg(arg: Info) -> Info { arg }
            let mapper = StorageToArgsMapper::Take(map_arg);
            let mut cc = callback.to_ffi_callback(Default::default(),
                                                  mapper);
            let fficc = cc.cc;
            let code = get_file_io()
                .query(self.unwrap(), &mut *cc, fficc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncFile for FileIo {
        fn async_set_len<F>(&mut self, len: u64, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_io()
                .set_length(self.unwrap(), len, cc.cc);
            cc.drop_with_code(code)
        }
    }

    impl<T: FileView> AsyncRead for SliceIo<T> {
        fn async_read<'a, F>(&mut self, offset: u64, size: usize, callback: F) -> Code
            where F: CallbackArgs<Cow<'a, [u8]>>
        {
            let slice_start = self.view_start();
            let slice_stop  = self.view_stop().unwrap_or(u64::max_value());

            let offset = slice_start + offset;
            let size = ::std::cmp::min((slice_stop - slice_start) as usize,
                                       size);

            self.0.async_read(offset, size, callback)
        }
    }
    impl<T: FileView> AsyncWrite for SliceIo<T> {
        fn async_write<F>(&mut self, offset: u64, buffer: Vec<u8>, callback: F) -> Code
            where F: CallbackArgs<(usize, Vec<u8>)>
        {
            let offset = self.view_start() + offset;
            self.0.async_write(offset, buffer, callback)
        }
        fn async_flush<F>(&mut self, callback: F) -> Code
            where F: Callback
        {
            self.0.async_flush(callback)
        }
    }
    impl<T: FileView> AsyncStream for SliceIo<T> { }
    impl<T: FileView> AsyncCommon for SliceIo<T> {
        fn async_touch<F>(&self, atime: Time, mtime: Time, callback: F) -> Code
            where F: Callback
        {
            self.0.async_touch(atime, mtime, callback)
        }
        fn async_query<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Info>
        {
            let slice_start = self.view_start();
            let slice_stop  = self.view_stop().unwrap_or(u64::max_value());
            self.0.async_query(move |info: Result<Info>| {
                let info = info
                    .map(|info| {
                        // Alter the file size to the slice's actual size.
                        ffi::Struct_PP_FileInfo {
                            size: (::std::cmp::min(info.size as u64, slice_stop) - slice_start) as i64,
                            .. info
                        }
                    });
                callback.call_directly(info)
            })
        }
    }


    // FileRef
    impl AsyncCommon for FileRef {
        fn async_touch<F>(&self, atime: Time, mtime: Time, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_ref()
                .touch(self.unwrap(), atime, mtime, cc.cc);
            cc.drop_with_code(code)
        }
        fn async_query<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Info>
        {
            fn map_arg(arg: Info) -> Info { arg }
            let mapper = StorageToArgsMapper::Take(map_arg);
            let mut cc = callback.to_ffi_callback(Default::default(),
                                                  mapper);
            let fficc = cc.cc;
            let code = get_file_ref()
                .query(self.unwrap(), &mut *cc, fficc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncPath for FileRef {
        fn async_mkdir<F>(&self, flags: MkDirFlags, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_ref()
                .mkdir(self.unwrap(), flags.into(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_delete<F>(&self, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_ref()
                .delete(self.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_rename<F>(&self, to: FileRef, callback: F) -> Code
            where F: Callback
        {
            let cc = callback.to_ffi_callback();
            let code = get_file_ref()
                .rename(self.unwrap(), to.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_read_directory_entries<F>(&self, callback: F) -> Code
            where F: CallbackArgs<Vec<DirectoryEntry>>
        {
            type StorageTy = StorageToArgsMapper<InPlaceArrayOutputStorage<DirectoryEntry>,
                                                 Vec<DirectoryEntry>>;
            let raw_args: InPlaceArrayOutputStorage<DirectoryEntry> = Default::default();
            let mapper: StorageTy = Default::default();
            let cc = callback.to_ffi_callback(raw_args, mapper);
            let fficc = cc.cc;
            let code = get_file_ref()
                .read_directory_entries(self.unwrap(), *cc.as_ref(), fficc);
            cc.drop_with_code(code)
        }

        fn async_open_io<F>(&self, instance: super::super::Instance,
                            flags: OpenFlags, callback: F) -> Code
            where F: CallbackArgs<FileIo>
        {
            impl super::super::InPlaceInit for FileIo { }

            let mut cc = callback.to_ffi_callback(FileIo(0), Default::default());

            let file_io = get_file_io().create(instance.unwrap());
            if file_io.is_none() { return cc.drop_with_code(Code::BadArgument); }
            let file_io = file_io.unwrap();
            cc.0 = file_io;

            let code = get_file_io()
                .open(self.unwrap(), file_io, flags.into(), cc.cc);
            cc.drop_with_code(code)
        }
    }

    impl Read for SliceIo {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            unimplemented!()
        }
    }
    impl Seek for SliceIo {
        fn seek(&mut self, from: io::SeekFrom) -> io::Result<u64> {
            unimplemented!()
        }
    }
    impl Write for SliceIo {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            unimplemented!()
        }
        fn flush(&mut self) -> io::Result<()> {
            self.async_flush(BlockUntilComplete)
                .to_valued_result(())
                .map_err(|code| code.into() )
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
