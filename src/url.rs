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

use super::{Callback, Resource, FileSliceRef, Instance, ToStringVar, ToVar};
use super::ppb::{get_url_loader, get_url_request};
use ppb::{URLRequestInfoIf, URLResponseInfoIf, URLLoaderIf};
use std::option::{Option, Some, None};
use std::{fmt, default, str};
use std::io::IoResult;
use collections::enum_set::{CLike, EnumSet};
use http::headers::request;
use super::ffi;
use super::ffi::bool_to_var;
use iurl::Url;

#[deriving(Hash, Eq, PartialEq, Show)] pub struct UrlLoader(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct UrlRequestInfo(ffi::PP_Resource);
#[deriving(Hash, Eq, PartialEq, Show)] pub struct UrlResponseInfo(ffi::PP_Resource);

impl_resource_for!(UrlLoader UrlLoaderRes)
impl_resource_for!(UrlRequestInfo UrlRequestInfoRes)
impl_resource_for!(UrlResponseInfo UrlResponseInfoRes)

pub type RequestProperties = EnumSet<RequestProperties_>;
#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum RequestProperties_ {
    AllowCrossOriginRequests,
    AllowCredentials,
    RecordUploadProgress,
    RecordDownloadProgress,
    FollowRedirects,
    StreamToFile,
}
impl CLike for RequestProperties_ {
    fn to_uint(&self) -> uint {
        match self {
            &AllowCrossOriginRequests => 0,
            &AllowCredentials => 1,
            &RecordUploadProgress => 2,
            &RecordDownloadProgress => 3,
            &FollowRedirects => 4,
            &StreamToFile => 5,
        }
    }
    fn from_uint(v: uint) -> RequestProperties_ {
        match v {
            0 => AllowCrossOriginRequests,
            1 => AllowCredentials,
            2 => RecordUploadProgress,
            3 => RecordDownloadProgress,
            4 => FollowRedirects,
            5 => StreamToFile,
            _ => unreachable!(),
        }
    }
}
impl RequestProperties_ {
    fn to_ffi(&self) -> ffi::PP_URLRequestProperty {
        match self {
            &AllowCrossOriginRequests => ffi::PP_URLREQUESTPROPERTY_ALLOWCROSSORIGINREQUESTS,
            &AllowCredentials => ffi::PP_URLREQUESTPROPERTY_ALLOWCREDENTIALS,
            &RecordUploadProgress => ffi::PP_URLREQUESTPROPERTY_RECORDUPLOADPROGRESS,
            &RecordDownloadProgress => ffi::PP_URLREQUESTPROPERTY_RECORDDOWNLOADPROGRESS,
            &FollowRedirects => ffi::PP_URLREQUESTPROPERTY_FOLLOWREDIRECTS,
            &StreamToFile => ffi::PP_URLREQUESTPROPERTY_STREAMTOFILE,
        }
    }
}
#[deriving(Clone)]
pub enum Body {
    FileBody(FileSliceRef, Option<super::Time>),
    BlobBody(Vec<u8>),
}
#[deriving(Clone, Eq, PartialEq, Hash)]
pub enum Method {
    GetMethod,
    PostMethod,
}
impl fmt::Show for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &GetMethod => f.pad("GET"),
            &PostMethod => f.pad("POST"),
        }
    }
}
impl default::Default for Method {
    fn default() -> Method {
        GetMethod
    }
}
#[deriving(Clone)]
pub struct RequestInfo {
    pub url: Url,

    pub method: Method,

    pub prefetch_buffer: Option<(i32, i32)>,
    properties: RequestProperties,
    set_props:  RequestProperties,

    pub body: Vec<Body>,

    pub headers: request::HeaderCollection,
}
impl RequestInfo {
    fn clear_bit(bitfield: RequestProperties, prop: RequestProperties_) -> RequestProperties {
        let mut new = EnumSet::empty();
        for p in bitfield
            .iter()
            .filter(|&p| p != prop ) {
                new.add(p);
            }
        new
    }
    pub fn set_prop(&mut self, prop: RequestProperties_, bit: bool) {
        self.set_props.add(prop);
        self.set_prop_value(prop, bit);
    }
    // unsets the 'this property is set' bit. Doesn't clear the actual property value bit.
    pub fn unset_prop(&mut self, prop: RequestProperties_) -> bool {
        let was_set = self.set_props.contains_elem(prop);
        self.set_props = RequestInfo::clear_bit(self.set_props, prop);
        was_set
    }

    // set the property value, but don't set the 'this property is set' bit.
    pub fn set_prop_value(&mut self, prop: RequestProperties_, bit: bool) -> bool {
        let mut new = RequestInfo::clear_bit(self.properties, prop);
        if bit {
            new.add(prop);
        }

        let was_set = self.properties.contains_elem(prop);
        self.properties = new;
        was_set
    }

    pub fn to_ffi(self) -> IoResult<UrlRequestInfo> {
        use std::io::MemWriter;
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
            body: body,
            headers: headers,
            url: url,
            method: method,
            ..
        } = self;
        for body in body.move_iter() {
            let success = match body {
                FileBody(FileSliceRef(file, start_opt, len_opt), time) => {
                    request.append_file_to_body(res.unwrap(),
                                                file.unwrap(),
                                                start_opt,
                                                len_opt,
                                                time)
                }
                BlobBody(blob) => {
                    request.append_blob_to_body(res.unwrap(),
                                                &blob)
                }
            };
            assert!(success);
        }
        let mut headers_str = MemWriter::new();
        try!(headers.write_all(&mut headers_str));
        let headers_str = headers_str.unwrap();
        let headers_str =
            str::from_utf8(headers_str.as_slice())
            .expect("HTTP headers should always be valid UTF8.");
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

#[deriving(Clone, Hash, Eq, PartialEq)]
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
    fn type_of(&self) -> super::ResourceType {
        self.unwrap_loader().type_of()
    }
}

impl UrlLoader {
    // TODO: this is messy. Do it generically.
    pub fn open(&self,
                ffi_info: UrlRequestInfo,
                callback: proc(OpenedUrlLoader): 'static + Send) -> super::Result<OpenedUrlLoader> {
        let loader = get_url_loader();
        let open_loader = OpenedUrlLoader(self.clone());
        let open_loader2 = open_loader.clone();
        let cb = proc() {
            callback(open_loader2);
        };
        loader.open(self.unwrap(),
                    ffi_info.unwrap(),
                    cb.to_ffi_callback())
            .map(|_| open_loader.clone() )
    }

}
