// Copyright (c) 2014 Richard Diamond & contributors.
//
// This file is part of the Rust PPApi project.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Callback, Resource, Instance, ToStringVar, ToVar, Code,
            StringVar, GenericResource, ResourceType, BlockUntilComplete,
            Var};
use super::ppb::{get_url_loader, get_url_loader_opt,
                 get_url_request_opt,
                 get_url_response};
use ppb::{self, URLRequestInfoIf, URLResponseInfoIf, URLLoaderIf};

use std::str::FromStr;
use std::ops::Deref;

use collections::enum_set::{CLike, EnumSet};

use hyper;
use hyper::header::Headers;
use httparse;

use super::ffi;
use super::ffi::bool_to_var;
use iurl::Url;

use fs::{SliceIo, FileIo, FileView};

#[derive(Hash, Eq, PartialEq, Debug)] pub struct UrlRequestInfo(ffi::PP_Resource);
#[derive(Hash, Eq, PartialEq, Debug)] pub struct ResponseInfo(ffi::PP_Resource);

impl_resource_for!(UrlRequestInfo, ResourceType::UrlRequestInfo);
impl_resource_for!(ResponseInfo, ResourceType::UrlResponseInfo);
impl_clone_drop_for!(UrlRequestInfo);
impl_clone_drop_for!(ResponseInfo);

pub type RequestProperties = EnumSet<RequestProperties_>;
#[derive(Eq, PartialEq, Clone, Hash, Copy, Debug)]
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
impl Into<ffi::PP_URLRequestProperty> for RequestProperties_ {
    fn into(self) -> ffi::PP_URLRequestProperty {
        match self {
            RequestProperties_::AllowCrossOriginRequests =>
                ffi::PP_URLREQUESTPROPERTY_ALLOWCROSSORIGINREQUESTS,
            RequestProperties_::AllowCredentials =>
                ffi::PP_URLREQUESTPROPERTY_ALLOWCREDENTIALS,
            RequestProperties_::RecordUploadProgress =>
                ffi::PP_URLREQUESTPROPERTY_RECORDUPLOADPROGRESS,
            RequestProperties_::RecordDownloadProgress =>
                ffi::PP_URLREQUESTPROPERTY_RECORDDOWNLOADPROGRESS,
            RequestProperties_::FollowRedirects =>
                ffi::PP_URLREQUESTPROPERTY_FOLLOWREDIRECTS,
            RequestProperties_::StreamToFile =>
                ffi::PP_URLREQUESTPROPERTY_STREAMTOFILE,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Body {
    File(SliceIo<FileIo>, Option<super::Time>),
    Blob(Vec<u8>),
}

pub type Method = hyper::method::Method;

trait CreateUrl {
    fn create_url_loader(&self) -> Code<ffi::PP_Resource>;
    fn create_url_request_info(&self) -> Code<UrlRequestInfo>;
}
impl CreateUrl for Instance {
    fn create_url_loader(&self) -> Code<ffi::PP_Resource> {
        get_url_loader_opt()
            .map(|i| {
                i.create(self.unwrap())
                    .map(|info| Code::Ok(info) )
                    .unwrap_or(Code::BadInstance)
            })
            .unwrap_or(Code::NoInterface)
    }
    fn create_url_request_info(&self) -> Code<UrlRequestInfo> {
        get_url_request_opt()
            .map(|i| {
                i.create(self.unwrap())
                    .map(|info| Code::Ok(UrlRequestInfo::new(info)) )
                    .unwrap_or(Code::BadInstance)
            })
            .unwrap_or(Code::NoInterface)
    }
}

#[derive(Clone, Debug)]
pub struct RequestInfo {
    pub url: Url,

    pub method: Method,

    pub prefetch_buffer: Option<(i32, i32)>,
    properties: RequestProperties,
    set_props:  RequestProperties,

    pub bodies: Vec<Body>,

    pub headers: Headers,
}
impl RequestInfo {
    pub fn new(url: Url, method: Method,
               body: Option<Body>, headers: Option<Headers>) -> RequestInfo {
        RequestInfo {
            url: url,
            method: method,

            prefetch_buffer: None,
            properties: EnumSet::new(),
            set_props: EnumSet::new(),

            bodies: body
                .map(|b| vec!(b) )
                .unwrap_or_default(),

            headers: headers.unwrap_or_else(|| Headers::new() ),
        }
    }
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

    pub fn follow_redirects(mut self) -> self {
        self.set_prop_value(RequestProperties_::FollowRedirects, true);
        self
    }

}
impl Into<Code<UrlRequestInfo>> for (Instance, RequestInfo) {
    fn into(self) -> Code<UrlRequestInfo> {
        let (instance, this) = self;

        let request = get_url_request_opt();
        if request.is_none() { return Code::NoInterface; }
        let request = request.unwrap();

        let res = try_code!(instance.create_url_request_info());

        let set_true = this.set_props.intersection(this.properties);
        let set_false = this.set_props - set_true;

        let true_var = unsafe { bool_to_var(1) };
        let false_var = unsafe { bool_to_var(0) };

        for prop in set_true.iter() {
            let set = request.property(res.unwrap(), prop.into(), true_var);
            if !set { return Code::Failed; }
        }
        for prop in set_false.iter() {
            let set = request.property(res.unwrap(), prop.into(), false_var);
            if !set { return Code::Failed; }
        }
        let RequestInfo {
            bodies, headers, url, method, ..
        } = this;
        for body in bodies.into_iter() {
            let success = match body {
                Body::File(ref slice, time) => {
                    request.append_file_to_body(res.unwrap(),
                                                slice.unwrap(),
                                                Some(slice.view_start() as i64),
                                                slice.view_len().map(|v| v as i64 ),
                                                time)
                }
                Body::Blob(ref blob) => {
                    request.append_blob_to_body(res.unwrap(),
                                                blob)
                }
            };
            if !success { return Code::Failed; }
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
        if !success { return Code::Failed; }
        Code::Ok(res)
    }
}

impl ResponseInfo {
    pub fn raw_url(&self) -> StringVar {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_URL);
        assert!(v.is_a_string());

        From::from(v)
    }
    pub fn url(&self) -> Url {
        let v = self.raw_url();
        FromStr::from_str(v.as_ref())
            .unwrap()
    }

    pub fn raw_redirect_url(&self) -> StringVar {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_REDIRECTURL);
        assert!(v.is_a_string());

        From::from(v)
    }
    pub fn redirect_url(&self) -> Option<Url> {
        let v = self.raw_redirect_url();
        if v == "" {
            None
        } else {
            Some(FromStr::from_str(v.as_ref()).unwrap())
        }
    }

    pub fn raw_redirect_method(&self) -> StringVar {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_REDIRECTMETHOD);
        assert!(v.is_a_string());

        From::from(v)
    }
    pub fn redirect_method(&self) -> Option<Method> {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_REDIRECTMETHOD);
        debug_assert!(v.is_a_string());

        let v = self.raw_redirect_method();
        if v == "" {
            None
        } else {
            Some(FromStr::from_str(v.as_ref()).unwrap())
        }
    }

    pub fn raw_status_code(&self) -> u16 {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_STATUSCODE);
        debug_assert!(v.is_an_i32());

        let v = unsafe { ffi::i32_from_var(v) };
        v as u16
    }
    pub fn status_code(&self) -> hyper::status::StatusCode {
        let v = self.raw_status_code();
        hyper::status::StatusCode::from_u16(v)
    }

    /// Returns a newline delimited string of `key: value` pairs.
    pub fn raw_headers_str<'a>(&'a self) -> &'a str {
        use std::str::from_utf8_unchecked;
        use std::slice::from_raw_parts;
        use std::mem::{transmute, uninitialized};

        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_HEADERS);
        assert!(v.is_a_string());

        let f = ppb::get_var().VarToUtf8.unwrap();

        unsafe {
            let mut len: u32 = uninitialized();
            let buf = f(v, &mut len as *mut u32);
            let len = len as usize;
            let slice = from_raw_parts(transmute(&buf), len);
            transmute(from_utf8_unchecked(slice))
        }
    }
    pub fn raw_headers_var(&self) -> StringVar {
        let v = get_url_response()
            .property(self.unwrap(), ffi::PP_URLRESPONSEPROPERTY_HEADERS);
        assert!(v.is_a_string());
        From::from(v)
    }

    pub fn headers(&self) -> Headers {
        let mut headers = Vec::with_capacity(5);
        let mut offset = 0;
        let raw_headers: &[u8] = self.raw_headers_str().as_ref();
        loop {
            headers.reserve(1);
            let current_len = headers.len();
            let current_cap = headers.capacity();
            unsafe { headers.set_len(current_cap) };
            let (bytes, parsed_len) = {
                let (b, p) =
                    httparse::parse_headers(&raw_headers[offset..],
                                            &mut headers[current_len..current_cap])
                    .unwrap()
                    .unwrap();
                (b, p.len())
            };
            unsafe { headers.set_len(current_len + parsed_len) };
            if bytes == 0 { break; }
            offset += bytes;
        }

        Headers::from_raw(&headers[..])
            .unwrap()
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Loader {
    res: GenericResource,
    info: ResponseInfo,
}
impl Resource for Loader {
    fn unwrap(&self) -> ffi::PP_Resource { self.res.unwrap() }
    fn type_of(&self) -> Option<ResourceType> { Some(ResourceType::UrlLoader) }
}
impl From<ffi::PP_Resource> for Loader {
    fn from(loader: ffi::PP_Resource) -> Loader {
        let info = get_url_loader()
            .get_response_info(loader)
            .unwrap();

        Loader {
            res: From::from(loader),
            info: ResponseInfo(info),
        }
    }
}
impl Deref for Loader {
    type Target = ResponseInfo;
    fn deref(&self) -> &ResponseInfo { &self.info }
}

impl Loader {
    pub fn download_progress(&self) -> Option<(u64, Option<u64>)> {
        use std::mem;
        let mut bytes = unsafe { mem::uninitialized() };
        let mut total = unsafe { mem::uninitialized() };

        let f = get_url_loader().GetDownloadProgress.unwrap();
        if !f(self.unwrap(), &mut bytes, &mut total) != 0 {
            None
        } else {
            if total == -1 {
                Some((bytes as u64, None))
            } else {
                Some((bytes as u64, Some(total as u64)))
            }
        }
    }
    pub fn upload_progress(&self) -> Option<(u64, u64)> {
        use std::mem;
        let mut bytes = unsafe { mem::uninitialized() };
        let mut total = unsafe { mem::uninitialized() };

        let f = get_url_loader().GetUploadProgress.unwrap();
        if !f(self.unwrap(), &mut bytes, &mut total) != 0 {
            None
        } else {
            Some((bytes as u64, total as u64))
        }
    }

    pub fn finish_streaming_to_file(&self) { unimplemented!() }

    pub fn info(&self) -> ResponseInfo { self.info.clone() }

    pub fn async_open<F>(instance: Instance, info: UrlRequestInfo,
                         callback: super::CallbackArgs<F, Loader>) ->
        Code<Loader> where F: FnOnce(Code<Loader>)
    {
        let loader = try_code!(instance.create_url_loader());

        impl super::InPlaceInit for ffi::PP_Resource { }

        fn map(loader: ffi::PP_Resource, _status: Code) -> Loader {
            From::from(loader)
        }

        let mapper = super::StorageToArgsMapper(map);
        let cc = callback.to_ffi_callback(loader, mapper);

        try_code!(get_url_loader().open(loader, info.unwrap(), cc.cc()) => CC(cc))
    }
    pub fn sync_open(instance: Instance, info: UrlRequestInfo) -> Code<Loader> {
        let loader = try_code!(instance.create_url_loader());

        try_code!(get_url_loader().open(loader, info.unwrap(),
                                        BlockUntilComplete::new()));
        Code::Ok(From::from(loader))
    }
}
