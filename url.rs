use super::{UrlLoader, UrlRequestInfo, UrlResponseInfo, Callback,
            Resource, FileSliceRef, OptionalName, Instance, ToStringVar,
            ToVar};
use super::ppb::{get_url_loader, get_url_request, get_url_response};
use std::option::{Option, Some, None};
use std::{fmt, default, str};
use std::io::IoResult;
use collections::enum_set::{CLike, EnumSet};
use http::headers::request;
use super::ffi;
use super::ffi::bool_to_var;
use url;

impl super::Instance {
    pub fn create_url_loader(&self) -> Option<UrlLoader> {
        get_url_loader().create(self.unwrap()).map(|loader| UrlLoader(loader) )
    }
    fn create_url_request_info(&self) -> Option<UrlRequestInfo> {
        get_url_request().create(self.unwrap()).map(|info| UrlRequestInfo(info) )
    }
}
pub type RequestProperties = EnumSet<RequestProperties_>;
#[deriving(Eq, TotalEq, Clone, Hash)]
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
#[deriving(Clone, Eq, TotalEq, Hash)]
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
    pub url: url::Url,

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

    fn to_ffi(self, instance: &super::Instance) -> IoResult<UrlRequestInfo> {
        use std::io::MemWriter;
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

        let url = url.to_str().to_string_var();
        let success = success && request.property(res.unwrap(),
                                                  ffi::PP_URLREQUESTPROPERTY_URL,
                                                  url.to_var());

        let method = method.to_str().to_string_var();
        let success = success && request.property(res.unwrap(),
                                                  ffi::PP_URLREQUESTPROPERTY_METHOD,
                                                  method.to_var());
        assert!(success);
        Ok(res)
    }
}

#[deriving(Clone, Hash, Eq, TotalEq)]
pub struct OpenedUrlLoader(UrlLoader);
impl OpenedUrlLoader {
    fn unwrap_loader<'a>(&'a self) -> &'a UrlLoader {
        let &OpenedUrlLoader(ref inner) = self;
        inner
    }
    pub fn download_progress(&self) -> Option<(i64, i64)> {
        use std::mem;
        let mut bytes_sent = unsafe { mem::uninit() };
        let mut total_sent = unsafe { mem::uninit() };

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

impl super::UrlLoader {
    pub fn open<TCb: Callback<OpenedUrlLoader>>(&self,
                                                info: RequestInfo,
                                                callback: TCb,
                                                name: OptionalName) -> super::Result<OpenedUrlLoader> {
        let instance = Instance::current();
        let loader = get_url_loader();
        let info = match info.to_ffi(&instance) {
            Ok(info) => info,
            Err(_) => {
                callback.sync_call(instance, name, None, super::Failed);
                return Err(super::Failed);
            }
        };
        let open_loader = OpenedUrlLoader(self.clone());
        loader.open(self.unwrap(),
                    info.unwrap(),
                    callback.to_ffi_callback(name,
                                             open_loader.clone()))
            .map(|_| open_loader.clone() )
    }
                
}
