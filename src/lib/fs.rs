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
///
/// This interface is currently evolving and therefore fairly unstable.

pub use self::common::*;
pub use self::impl_::*;

pub struct ParentsIter<T>
    where T: Path,
{
    p: T,
}
impl<T: Path> Iterator for ParentsIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let parent = self.p.parent();
        if parent == self.p {
            None
        } else {
            self.p = parent.clone();
            Some(parent)
        }
    }
}

pub mod common {
    use std::borrow::Cow;
    use std::fmt::{Display, Debug};

    use ffi;
    use super::super::{CallbackArgs, Code, Time, Resource,
                       StringVar};

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
        fn async_read<'a, F>(&mut self, offset: u64, size: usize,
                             callback: CallbackArgs<F, Cow<'a, [u8]>>) ->
            Code<Cow<'a, [u8]>> where F: FnOnce(Code<Cow<'a, [u8]>>);
    }
    pub trait AsyncWrite {
        fn async_write<'a, F>(&mut self, offset: u64,
                              buffer: Cow<'a, [u8]>,
                              callback: CallbackArgs<F, (usize, Cow<'a, [u8]>)>) ->
            Code<(usize, Cow<'a, [u8]>)> where F: FnOnce(Code<(usize, Cow<'a, [u8]>)>);
        fn async_flush<F>(&mut self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>);
    }
    pub trait AsyncStream: AsyncRead + AsyncWrite { }


    // file/dir releated stuffs
    pub trait AsyncCommon {
        fn async_touch<F>(&self, atime: Time, mtime: Time,
                          callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>);

        fn async_query<F>(&self, callback: CallbackArgs<F, Info>) -> Code<Info>
            where F: FnOnce(Code<Info>);
    }
    pub trait AsyncFile: AsyncCommon {
        fn async_set_len<F>(&mut self, len: u64,
                            callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>);
    }
    pub trait AsyncPath: AsyncCommon {
        fn async_mkdir<F>(&self, flags: MkDirFlags, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>);
        fn async_delete<F>(&self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>);
        fn async_rename<F>(&self, to: Self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>);
        fn async_read_directory_entries<F>(&self, callback: CallbackArgs<F, Vec<DirectoryEntry>>) ->
            Code<Vec<DirectoryEntry>> where F: FnOnce(Code<Vec<DirectoryEntry>>);

        fn async_open_io<F>(&self, instance: super::super::Instance,
                            flags: OpenFlags,
                            callback: CallbackArgs<F, super::FileIo>) -> Code<super::FileIo>
            where F: FnOnce(Code<super::FileIo>);
    }

    pub trait SyncCommon {
        fn sync_touch(&self, atime: Time, mtime: Time) -> Code;
        fn sync_query(&self) -> Code<Info>;
    }
    pub trait SyncFile: SyncCommon {
        fn sync_set_len(&mut self, len: u64) -> Code;
    }
    pub trait SyncPath: SyncCommon {
        fn sync_mkdir(&self, flags: MkDirFlags) -> Code;
        fn sync_delete(&self) -> Code;
        fn sync_rename(&self, to: Self) -> Code;
        fn sync_read_directory_entires(&self) -> Code<Vec<DirectoryEntry>>;

        fn sync_open_io(&self, instance: super::super::Instance,
                        flags: OpenFlags) -> Code<super::FileIo>;
    }

    pub trait Path: AsyncPath + SyncPath + Display + Debug + Eq + Sized + Clone {
        fn name(&self) -> StringVar;
        fn path(&self) -> StringVar;
        fn parent(&self) -> Self;

        /// The first element will be Self's parent.
        fn parents_iter(self) -> super::ParentsIter<Self> {
            super::ParentsIter {
                p: self,
            }
        }
    }

    pub trait FileView: AsyncStream + AsyncCommon + Resource {
        type View;
        type Io;

        fn io(&self) -> &Self::Io;

        fn view(&self, from: u64, to: Option<u64>) -> Self::View;
        fn view_full(&self) -> Self::View { self.view(0, None) }

        fn view_start(&self) -> u64;
        fn view_stop (&self) -> Option<u64>;
        fn view_len  (&self) -> Option<u64> {
            self.view_stop()
                .map(|stop| {
                    let start = self.view_start();
                    if stop < start {
                        0
                    } else {
                        stop - start
                    }
                })
        }

        fn view_absolute_start(&self) -> u64;
    }
}


#[cfg(feature = "pepper")]
mod impl_ {
    use std::borrow::Cow;
    use std::path;

    use ffi;
    use ppb::{FileSystemIf, FileRefIf, FileIoIf};
    use ppb::{get_file_system, get_file_ref, get_file_io};
    use super::common::{AsyncRead, AsyncWrite, AsyncFile, AsyncPath,
                        AsyncCommon, AsyncStream, Info, OpenFlags, MkDirFlags,
                        DirectoryEntry, FileView, SyncCommon, SyncFile,
                        SyncPath};
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
    /// Was `SliceIo<T>`, but rustc has a recursion bug :(
    pub struct SliceIo(FileIo, u64, Option<u64>);
    impl Resource for SliceIo {
        fn unwrap(&self) -> ffi::PP_Resource { self.io().unwrap() }
        fn type_of(&self) -> Option<ResourceType> { self.io().type_of() }
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

        pub fn create<T: AsRef<path::Path>>(&self, path: T) -> Option<FileRef> {
            let cstr = format!("{}\0", path.as_ref().display());
            get_file_ref().create(self.unwrap(), cstr.as_ptr() as *const _)
                .map(|r| FileRef(r) )
        }
    }

    impl FileView for FileIo {
        type View = SliceIo;
        type Io = FileIo;
        fn io(&self) -> &FileIo { self }

        fn view(&self, from: u64, to: Option<u64>) -> SliceIo {
            match (from, to) {
                (from, Some(to)) => assert!(from <= to),
                _ => (),
            }
            SliceIo(self.io().clone(), from, to)
        }

        fn view_start(&self) -> u64         { 0    }
        fn view_stop (&self) -> Option<u64> { None }

        fn view_absolute_start(&self) -> u64 { 0 }
    }
    impl FileView for SliceIo {
        type View = SliceIo;
        type Io = FileIo;

        fn io(&self) -> &FileIo { &self.0 }

        fn view(&self, from: u64, to: Option<u64>) -> SliceIo {
            match (from, to) {
                (from, Some(to)) => assert!(from <= to),
                _ => (),
            }
            let to = to.map(|to| self.view_start() + to );
            SliceIo(self.io().clone(), self.view_start() + from, to)
        }

        fn view_start(&self) -> u64         { self.1 }
        fn view_stop (&self) -> Option<u64> { self.2 }

        fn view_absolute_start(&self) -> u64 {
            self.0.view_absolute_start() + self.view_start()
        }
    }

    impl AsyncRead for FileIo {
        fn async_read<'a, F>(&mut self, offset: u64, size: usize,
                             callback: CallbackArgs<F, Cow<'a, [u8]>>) ->
            Code<Cow<'a, [u8]>> where F: FnOnce(Code<Cow<'a, [u8]>>)
        {
            fn map_arg<'a>(raw: InPlaceArrayOutputStorage<u8>, _status: Code) -> Cow<'a, [u8]> {
                let v: Vec<_> = raw.into();
                Cow::Owned(v)
            }

            let raw_args: InPlaceArrayOutputStorage<u8> = Default::default();
            let mapper = StorageToArgsMapper(map_arg);
            let mut cc = callback.to_ffi_callback(raw_args, mapper);
            let fficc = cc.cc;
            let code = get_file_io()
                .read_to_array(self.unwrap(), offset, size as u32,
                               cc.as_mut(), fficc);
            cc.drop_with_code(code)
        }
    }

    impl AsyncWrite for FileIo {
        fn async_write<'a, F>(&mut self, offset: u64,
                              buffer: Cow<'a, [u8]>,
                              callback: CallbackArgs<F, (usize, Cow<'a, [u8]>)>) ->
            Code<(usize, Cow<'a, [u8]>)> where F: FnOnce(Code<(usize, Cow<'a, [u8]>)>)
        {
            impl<'a> super::super::InPlaceInit for (bool, usize, Cow<'a, [u8]>) { }

            fn map_arg<'a>((sync, written, buf): (bool, usize, Cow<'a, [u8]>),
                           status: Code) -> (usize, Cow<'a, [u8]>) {
                if sync {
                    (written, buf)
                } else {
                    let written = match status {
                        Code::Ok(written) => written,
                        _ => unreachable!(),
                    };
                    (written, buf)
                }
            }
            let mapper = StorageToArgsMapper(map_arg);
            let mut cc = callback.to_ffi_callback((false, 0, buffer), mapper);
            let written = get_file_io()
                .write(self.unwrap(), offset,
                       cc.2.as_ptr() as *const _,
                       cc.2.len() as usize,
                       cc.cc);
            match written {
                Code::Ok(written) => {
                    cc.0 = true;
                    cc.1 = written as usize;
                    cc.drop_with_code(Code::Ok(written))
                },
                code => cc.drop_with_code(code),
            }
        }
        fn async_flush<F>(&mut self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_io()
                .flush(self.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncStream for FileIo { }
    impl AsyncCommon for FileIo {
        fn async_touch<F>(&self, atime: Time, mtime: Time,
                          callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_io()
                .touch(self.unwrap(), atime, mtime, cc.cc);
            cc.drop_with_code(code)
        }
        fn async_query<F>(&self, callback: CallbackArgs<F, Info>) -> Code<Info>
            where F: FnOnce(Code<Info>)
        {
            impl super::super::InPlaceInit for ffi::Struct_PP_FileInfo { }
            fn map_arg(arg: Info, _status: Code) -> Info { arg }
            let mapper = StorageToArgsMapper(map_arg);
            let mut cc = callback.to_ffi_callback(Default::default(),
                                                  mapper);
            let fficc = cc.cc;
            let code = get_file_io()
                .query(self.unwrap(), &mut *cc, fficc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncFile for FileIo {
        fn async_set_len<F>(&mut self, len: u64,
                            callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_io()
                .set_length(self.unwrap(), len, cc.cc);
            cc.drop_with_code(code)
        }
    }

    impl SyncCommon for FileIo {
        fn sync_touch(&self, atime: Time, mtime: Time) -> Code {
            get_file_io()
                .touch(self.unwrap(), atime, mtime,
                       BlockUntilComplete::new())
        }
        fn sync_query(&self) -> Code<Info> {
            let mut dest: Info = Default::default();
            get_file_io()
                .query(self.unwrap(), &mut dest,
                       BlockUntilComplete::new())
                .map_ok(move |_| dest )
        }
    }
    impl SyncFile for FileIo {
        fn sync_set_len(&mut self, len: u64) -> Code {
            get_file_io()
                .set_length(self.unwrap(), len,
                            BlockUntilComplete::new())
        }
    }

    impl AsyncRead for SliceIo {
        fn async_read<'a, F>(&mut self, offset: u64, size: usize,
                             callback: CallbackArgs<F, Cow<'a, [u8]>>) ->
            Code<Cow<'a, [u8]>> where F: FnOnce(Code<Cow<'a, [u8]>>)
        {
            let slice_start = self.view_start();
            let slice_stop  = self.view_stop().unwrap_or(u64::max_value());

            let offset = slice_start + offset;
            let size = ::std::cmp::min((slice_stop - slice_start) as usize,
                                       size);

            self.0.async_read(offset, size, callback)
        }
    }
    impl AsyncWrite for SliceIo {
        fn async_write<'a, F>(&mut self, offset: u64,
                              buffer: Cow<'a, [u8]>,
                              callback: CallbackArgs<F, (usize, Cow<'a, [u8]>)>) ->
            Code<(usize, Cow<'a, [u8]>)> where F: FnOnce(Code<(usize, Cow<'a, [u8]>)>)
        {
            let offset = self.view_start() + offset;
            self.0.async_write(offset, buffer, callback)
        }
        fn async_flush<F>(&mut self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>)
        {
            self.0.async_flush(callback)
        }
    }
    impl AsyncStream for SliceIo { }
    impl AsyncCommon for SliceIo {
        fn async_touch<F>(&self, atime: Time, mtime: Time,
                          callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>)
        {
            self.0.async_touch(atime, mtime, callback)
        }
        fn async_query<F>(&self, callback: CallbackArgs<F, Info>) -> Code<Info>
            where F: FnOnce(Code<Info>)
        {
            let slice_start = self.view_start();
            let slice_stop  = self.view_stop().unwrap_or(u64::max_value());

            fn map(info: Info, slice_start: u64, slice_stop: u64) -> Info {
                // Alter the file size to the slice's actual size.
                ffi::Struct_PP_FileInfo {
                    size: (::std::cmp::min(info.size as u64, slice_stop) - slice_start) as i64,
                    .. info
                }
            }
            let optional = callback.optional;
            let mut local = CallbackArgs::new(move |info: Code<Info>| {
                let info = info.map_ok(|info| map(info, slice_start, slice_stop) );
                callback.call_directly(info)
            });
            local.set_optional(optional);

            let code = self.0.async_query(local);
            if let Code::Ok(info) = code {
                Code::Ok(map(info, slice_start, slice_stop))
            } else {
                code
            }
        }
    }
    impl SyncCommon for SliceIo {
        fn sync_touch(&self, atime: Time, mtime: Time) -> Code {
            self.io().sync_touch(atime, mtime)
        }
        fn sync_query(&self) -> Code<Info> {
            self.io()
                .sync_query()
                .map_ok(|info| {
                    let slice_start = self.view_start();
                    let slice_stop  = self.view_stop().unwrap_or(u64::max_value());
                    // Alter the file size to the slice's actual size.
                    ffi::Struct_PP_FileInfo {
                        size: (::std::cmp::min(info.size as u64, slice_stop) - slice_start) as i64,
                        .. info
                    }
                })
        }
    }


    // FileRef
    impl AsyncCommon for FileRef {
        fn async_touch<F>(&self, atime: Time, mtime: Time,
                          callback: CallbackArgs<F, ()>) ->
            Code<()> where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_ref()
                .touch(self.unwrap(), atime, mtime, cc.cc);
            cc.drop_with_code(code)
        }
        fn async_query<F>(&self, callback: CallbackArgs<F, Info>) -> Code<Info>
            where F: FnOnce(Code<Info>)
        {
            fn map_arg(arg: Info, _status: Code) -> Info { arg }
            let mapper = StorageToArgsMapper(map_arg);
            let mut cc = callback.to_ffi_callback(Default::default(),
                                                  mapper);
            let fficc = cc.cc;
            let code = get_file_ref()
                .query(self.unwrap(), &mut *cc, fficc);
            cc.drop_with_code(code)
        }
    }
    impl AsyncPath for FileRef {
        fn async_mkdir<F>(&self, flags: MkDirFlags, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_ref()
                .mkdir(self.unwrap(), flags.into(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_delete<F>(&self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_ref()
                .delete(self.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_rename<F>(&self, to: Self, callback: CallbackArgs<F, ()>) -> Code<()>
            where F: FnOnce(Code<()>)
        {
            let cc = callback.to_ffi_callback((), Default::default());
            let code = get_file_ref()
                .rename(self.unwrap(), to.unwrap(), cc.cc);
            cc.drop_with_code(code)
        }
        fn async_read_directory_entries<F>(&self, callback: CallbackArgs<F, Vec<DirectoryEntry>>) ->
            Code<Vec<DirectoryEntry>> where F: FnOnce(Code<Vec<DirectoryEntry>>)
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
                            flags: OpenFlags,
                            callback: CallbackArgs<F, super::FileIo>) -> Code<super::FileIo>
            where F: FnOnce(Code<super::FileIo>)
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
    impl SyncCommon for FileRef {
        fn sync_touch(&self, atime: Time, mtime: Time) -> Code {
            get_file_ref()
                .touch(self.unwrap(), atime, mtime,
                       BlockUntilComplete::new())
        }
        fn sync_query(&self) -> Code<Info> {
            let mut dest: Info = Default::default();
            get_file_ref()
                .query(self.unwrap(), &mut dest,
                       BlockUntilComplete::new())
                .map_ok(move |_| dest )
        }
    }
    impl SyncPath for FileRef {
        fn sync_mkdir(&self, flags: MkDirFlags) -> Code {
            get_file_ref()
                .mkdir(self.unwrap(), flags.into(),
                       BlockUntilComplete::new())
        }
        fn sync_delete(&self) -> Code {
            get_file_ref()
                .delete(self.unwrap(),
                        BlockUntilComplete::new())
        }
        fn sync_rename(&self, to: FileRef) -> Code {
            get_file_ref()
                .rename(self.unwrap(), to.unwrap(),
                        BlockUntilComplete::new())
        }
        fn sync_read_directory_entires(&self) -> Code<Vec<DirectoryEntry>> {
            use super::super::InPlaceInit;
            let mut dest: InPlaceArrayOutputStorage<DirectoryEntry> =
                Default::default();
            dest.inplace_init();

            get_file_ref()
                .read_directory_entries(self.unwrap(), *dest.as_ref(),
                                        BlockUntilComplete::new())
                .map_ok(move |_| dest.into() )
        }

        fn sync_open_io(&self, instance: super::super::Instance,
                        flags: OpenFlags) -> Code<FileIo> {
            let file_io = get_file_io().create(instance.unwrap());
            if file_io.is_none() { return Code::BadArgument; }
            let file_io = file_io.unwrap();

            get_file_io()
                .open(self.unwrap(), file_io, flags.into(),
                      BlockUntilComplete::new())
                .map_ok(move |_| FileIo(file_io) )
        }
    }

    impl Read for SliceIo {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let cc = BlockUntilComplete;
            let cc = cc.to_ffi_callback();

            let to_read = ::std::cmp::min(buf.len() as u64,
                                          self.view_len().unwrap_or(u64::max_value()));
            let to_read = to_read as usize;

            let offset = self.view_absolute_start();
            let read = get_file_io()
                .read(self.unwrap(), offset, buf.as_mut_ptr() as *mut _,
                      to_read, cc.cc());

            let read = match read {
                Code::CompletionPending => unreachable!(),
                Code::Ok(read) => read,
                read => { return Err(read.into()); },
            };
            self.1 += read as u64;
            Ok(read as usize)
        }
    }
    impl Seek for SliceIo {
        fn seek(&mut self, from: io::SeekFrom) -> io::Result<u64> {
            match from {
                io::SeekFrom::Start(v) => {
                    self.1 = v;
                },
                io::SeekFrom::End(_) => {
                    // TODO
                    return Err({Code::NotSupported}.into());
                },
                io::SeekFrom::Current(v) => {
                    if v < 0 && -v as u64 > self.1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                                  "can't seek before 0"));
                    }
                    if v < 0 {
                        self.1 -= (-v) as u64;
                    } else {
                        self.1 += v as u64;
                    }
                },
            }

            Ok(self.1)
        }
    }
    impl Write for SliceIo {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let cc = BlockUntilComplete;
            let cc = cc.to_ffi_callback();

            let to_write = ::std::cmp::min(buf.len() as u64,
                                           self.view_len().unwrap_or(u64::max_value()));
            let to_write = to_write as usize;

            let offset = self.view_absolute_start();
            let write = get_file_io()
                .write(self.unwrap(), offset, buf.as_ptr() as *const _,
                       to_write, cc.cc());

            let write = match write {
                Code::CompletionPending => unreachable!(),
                Code::Ok(write) => write,
                write => { return Err(write.into()); },
            };
            self.1 += write as u64;
            Ok(write as usize)
        }
        fn flush(&mut self) -> io::Result<()> {
            let cc = BlockUntilComplete::new();
            get_file_io()
                .flush(self.unwrap(), cc)
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
