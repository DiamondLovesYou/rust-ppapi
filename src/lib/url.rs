// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Callback, Resource, Instance, ToStringVar, ToVar, Code};
use super::ppb::{get_url_loader, get_url_request};
use ppb::{URLRequestInfoIf, URLResponseInfoIf, URLLoaderIf};
use std::io::Result;
use collections::enum_set::{CLike, EnumSet};
use http;
use http::header::Headers;
use super::ffi;
use super::ffi::bool_to_var;
use iurl::Url;

use fs::{SliceIo, FileIo, FileView};

#[derive(Hash, Eq, PartialEq, Debug)] pub struct UrlLoader(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Debug)] pub struct UrlRequestInfo(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Debug)] pub struct UrlResponseInfo(ffi::PP_Resource);

impl_resource_for!(UrlLoader, ResourceType::UrlLoader);
impl_resource_for!(UrlRequestInfo, ResourceType::UrlRequestInfo);
impl_resource_for!(UrlResponseInfo, ResourceType::UrlResponseInfo);

pub type RequestProperties = EnumSet<RequestProperties_>;
#[derive(Eq, PartialEq, Clone, Hash, Copy)]
pub enum RequestProperties_ {
    AllowCrossOriginRequests,
    AllowCredentials,
    RecordUploadProgress,
    RecordDownloadProgress,
    FollowRedirects,
    StreamToFile,
}
impl CLike for RequestProperties_ {
    fn to_usize(&self) -> usize {
        match self {
            &RequestProperties_::AllowCrossOriginRequests => 0,
            &RequestProperties_::AllowCredentials => 1,
            &RequestProperties_::RecordUploadProgress => 2,
            &RequestProperties_::RecordDownloadProgress => 3,
            &RequestProperties_::FollowRedirects => 4,
            &RequestProperties_::StreamToFile => 5,
        }
    }
    fn from_usize(v: usize) -> RequestProperties_ {
        match v {
            0 => RequestProperties_::AllowCrossOriginRequests,
            1 => RequestProperties_::AllowCredentials,
            2 => RequestProperties_::RecordUploadProgress,
            3 => RequestProperties_::RecordDownloadProgress,
            4 => RequestProperties_::FollowRedirects,
            5 => RequestProperties_::StreamToFile,
            _ => unreachable!(),
        }
    }
}
impl RequestProperties_ {
    fn to_ffi(&self) -> ffi::PP_URLRequestProperty {
        match self {
            &RequestProperties_::AllowCrossOriginRequests =>
                ffi::PP_URLREQUESTPROPERTY_ALLOWCROSSORIGINREQUESTS,
            &RequestProperties_::AllowCredentials =>
                ffi::PP_URLREQUESTPROPERTY_ALLOWCREDENTIALS,
            &RequestProperties_::RecordUploadProgress =>
                ffi::PP_URLREQUESTPROPERTY_RECORDUPLOADPROGRESS,
            &RequestProperties_::RecordDownloadProgress =>
                ffi::PP_URLREQUESTPROPERTY_RECORDDOWNLOADPROGRESS,
            &RequestProperties_::FollowRedirects =>
                ffi::PP_URLREQUESTPROPERTY_FOLLOWREDIRECTS,
            &RequestProperties_::StreamToFile =>
                ffi::PP_URLREQUESTPROPERTY_STREAMTOFILE,
        }
    }
}
#[derive(Clone)]
pub enum Body {
    File(SliceIo<FileIo>, Option<super::Time>),
    Blob(Vec<u8>),
}

pub type Method = http::method::Method;

#[derive(Clone)]
pub struct RequestInfo {
    pub url: Url,

    pub method: Method,

    pub prefetch_buffer: Option<(i32, i32)>,
    properties: RequestProperties,
    set_props:  RequestProperties,

    pub body: Vec<Body>,

    pub headers: Headers,
}
impl RequestInfo {
    fn clear_bit(bitfield: RequestProperties, prop: RequestProperties_) -> RequestProperties {
        let mut new = EnumSet::new();
        for p in bitfield
            .iter()
            .filter(|&p| p != prop ) {
                new.insert(p);
            }
        new
    }
    pub fn set_prop(&mut self, prop: RequestProperties_, bit: bool) {
        self.set_props.insert(prop);
        self.set_prop_value(prop, bit);
    }
    // unsets the 'this property is set' bit. Doesn't clear the actual property value bit.
    pub fn unset_prop(&mut self, prop: RequestProperties_) -> bool {
        let was_set = self.set_props.contains(&prop);
        self.set_props = RequestInfo::clear_bit(self.set_props, prop);
        was_set
    }

    // set the property value, but don't set the 'this property is set' bit.
    pub fn set_prop_value(&mut self, prop: RequestProperties_, bit: bool) -> bool {
        let mut new = RequestInfo::clear_bit(self.properties, prop);
        if bit {
            new.insert(prop);
        }

        let was_set = self.properties.contains(&prop);
        self.properties = new;
        was_set
    }

    // todo remove those asserts.
    pub fn to_ffi(self) -> Result<UrlRequestInfo> {
        let instance = Instance::current();
        let request = get_url_request();
        let res = instance.create_url_request_info().unwrap();
        let set_true = self.set_props.intersection(self.properties);
        let set_false = self.set_props - set_true;

        let true_var = unsafe { bool_to_var(1) };
        let false_var = unsafe { bool_to_var(0) };

        for prop in set_true.iter() {
            let set = request.property(res.unwrap(), prop.to_ffi(), true_var);
            assert!(set);
        }
        for prop in set_false.iter() {
            let set = request.property(res.unwrap(), prop.to_ffi(), false_var);
            assert!(set);
        }
        let RequestInfo {
            body, headers, url, method, ..
        } = self;
        for body in body.into_iter() {
            let success = match body {
                Body::File(slice, time) => {
                    request.append_file_to_body(res.unwrap(),
                                                slice.unwrap(),
                                                Some(slice.view_start() as i64),
                                                slice.view_len().map(|v| v as i64 ),
                                                time)
                }
                Body::Blob(blob) => {
                    request.append_blob_to_body(res.unwrap(),
                                                &blob)
                }
            };
            assert!(success);
        }
        let headers_str = headers.to_string();
        let headers_str = headers_str.to_string_var();
        let success = request.property(res.unwrap(),
                                       ffi::PP_URLREQUESTPROPERTY_HEADERS,
                                       headers_str.to_var());

        let url = url.to_string().to_string_var();
        let success = success && request.property(res.unwrap(),
                                                  ffi::PP_URLREQUESTPROPERTY_URL,
                                                  url.to_var());

        let method = method.to_string().to_string_var();
        let success = success && request.property(res.unwrap(),
                                                  ffi::PP_URLREQUESTPROPERTY_METHOD,
                                                  method.to_var());
        assert!(success);
        Ok(res)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct OpenedUrlLoader(UrlLoader);
impl OpenedUrlLoader {
    fn unwrap_loader<'a>(&'a self) -> &'a UrlLoader {
        let &OpenedUrlLoader(ref inner) = self;
        inner
    }
    pub fn download_progress(&self) -> Option<(i64, i64)> {
        use std::mem;
        let mut bytes_sent = unsafe { mem::uninitialized() };
        let mut total_sent = unsafe { mem::uninitialized() };

        let f = get_url_loader().GetDownloadProgress.unwrap();
        if !f(self.unwrap_loader().unwrap(), &mut bytes_sent, &mut total_sent) != 0 {
            None
        } else {
            Some((bytes_sent as i64, total_sent as i64))
        }
    }
}
impl Resource for OpenedUrlLoader {
    fn unwrap(&self) -> ffi::PP_Resource {
        self.unwrap_loader().unwrap()
    }
    fn type_of(&self) -> Option<super::ResourceType> {
        self.unwrap_loader().type_of()
    }
}

impl UrlLoader {
    // TODO: this is messy. Do it generically.
    pub fn open<F>(self, ffi_info: UrlRequestInfo, callback: F) -> Code
        where F: super::CallbackArgs<OpenedUrlLoader>,
    {
        let loader = get_url_loader();
        let res = self.unwrap();

        impl super::InPlaceInit for UrlLoader { }

        fn map(this: UrlLoader, _status: usize) -> OpenedUrlLoader {
            OpenedUrlLoader(this)
        }

        let mapper = super::StorageToArgsMapper::Take(map);
        let cc = callback.to_ffi_callback(self, mapper);

        let code = loader.open(res, ffi_info.unwrap(), cc.cc);
        cc.drop_with_code(code)
    }

}
