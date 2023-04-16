extern crate libc;

extern crate llhttp_sys;

use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_uint};

mod consts;
pub mod ffi {
    pub use llhttp_sys::*;
}

pub use consts::*;
use ffi::llhttp_method_name;

pub type CallBack = ffi::llhttp_cb;
pub type DataCallBack = ffi::llhttp_data_cb;

#[derive(Copy, Clone, Debug, Default)]
pub struct Settings(ffi::llhttp_settings_t);

unsafe impl Send for Settings {}

#[macro_export]
macro_rules! cb_wrapper {
    ($fname:ident, $func:ident, $data_type:ty) => {
        #[inline]
        unsafe extern "C" fn $fname(arg1: *mut llhttp::ffi::llhttp_t) -> libc::c_int {
            let parser = &mut *(arg1 as *mut llhttp::Parser<$data_type>);
            match $func(parser) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
    };
}

#[macro_export]
macro_rules! data_cb_wrapper {
    ($fname:ident, $func:ident, $data_type:ty) => {
        #[inline]
        unsafe extern "C" fn $fname(
            arg1: *mut llhttp::ffi::llhttp_t,
            at: *const ::libc::c_char,
            length: usize,
        ) -> libc::c_int {
            let parser = &mut *(arg1 as *mut llhttp::Parser<$data_type>);
            let data = std::slice::from_raw_parts(at as *const u8, length + 1);
            match $func(parser, data) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
    };
}

impl Settings {
    pub fn on_message_begin(&mut self, cb: CallBack) {
        self.0.on_message_begin = cb;
    }
    pub fn on_url(&mut self, cb: DataCallBack) {
        self.0.on_url = cb;
    }
    pub fn on_status(&mut self, cb: DataCallBack) {
        self.0.on_status = cb;
    }
    pub fn on_header_field(&mut self, cb: DataCallBack) {
        self.0.on_header_field = cb;
    }
    pub fn on_header_value(&mut self, cb: DataCallBack) {
        self.0.on_header_value = cb;
    }
    pub fn on_headers_complete(&mut self, cb: CallBack) {
        self.0.on_headers_complete = cb;
    }
    pub fn on_body(&mut self, cb: DataCallBack) {
        self.0.on_body = cb;
    }
    pub fn on_message_complete(&mut self, cb: CallBack) {
        self.0.on_message_complete = cb;
    }
    pub fn on_chunk_header(&mut self, cb: CallBack) {
        self.0.on_chunk_header = cb;
    }
    pub fn on_chunk_complete(&mut self, cb: CallBack) {
        self.0.on_chunk_complete = cb;
    }
    pub fn on_url_complete(&mut self, cb: CallBack) {
        self.0.on_url_complete = cb;
    }
    pub fn on_status_complete(&mut self, cb: CallBack) {
        self.0.on_status_complete = cb;
    }
    pub fn on_header_field_complete(&mut self, cb: CallBack) {
        self.0.on_header_field_complete = cb;
    }
    pub fn on_header_value_complete(&mut self, cb: CallBack) {
        self.0.on_header_value_complete = cb;
    }
}

impl Settings {
    pub fn new() -> Settings {
        let mut settings = Settings::default();
        unsafe {
            ffi::llhttp_settings_init(&mut settings.0);
        }
        settings
    }
}

/// llhttp parser
#[derive(Clone, Default, Debug)]
pub struct Parser<'a, T> {
    _llhttp: ffi::llhttp_t,
    _settings: PhantomData<&'a Settings>,
    _data: PhantomData<T>,
}

impl<'a, T> Parser<'a, T> {
    #[inline]
    pub fn init(&mut self, settings: &Settings, lltype: Type) {
        unsafe {
            ffi::llhttp_init(&mut self._llhttp, lltype.into(), &settings.0);
        }
    }

    #[inline]
    pub fn parse(&mut self, data: &[u8]) -> Error {
        unsafe {
            ffi::llhttp_execute(
                &mut self._llhttp,
                data.as_ptr() as *const c_char,
                data.len(),
            )
        }
    }

    #[inline]
    pub fn finish(&mut self) -> Error {
        unsafe { ffi::llhttp_finish(&mut self._llhttp) }
    }

    #[inline]
    pub fn message_needs_eof(&self) -> bool {
        unsafe {
            match ffi::llhttp_message_needs_eof(&self._llhttp) {
                1 => true,
                _ => false,
            }
        }
    }

    #[inline]
    pub fn should_keep_alive(&self) -> bool {
        unsafe {
            match ffi::llhttp_should_keep_alive(&self._llhttp) {
                1 => true,
                _ => false,
            }
        }
    }

    #[inline]
    pub fn pause(&mut self) {
        unsafe { ffi::llhttp_pause(&mut self._llhttp) }
    }

    #[inline]
    pub fn resume(&mut self) {
        unsafe { ffi::llhttp_resume(&mut self._llhttp) }
    }

    #[inline]
    pub fn resume_after_upgrade(&mut self) {
        unsafe { ffi::llhttp_resume_after_upgrade(&mut self._llhttp) }
    }

    #[inline]
    pub fn errno(&self) -> Error {
        unsafe { ffi::llhttp_get_errno(&self._llhttp) }
    }

    #[inline]
    pub fn get_error_reason(&self) -> &CStr {
        unsafe { CStr::from_ptr(ffi::llhttp_get_error_reason(&self._llhttp)) }
    }

    #[inline]
    pub fn get_error_pos(&self) -> &CStr {
        unsafe { CStr::from_ptr(ffi::llhttp_get_error_pos(&self._llhttp)) }
    }

    #[inline]
    pub fn status_code(&self) -> u16 {
        self._llhttp.status_code
    }

    #[inline]
    pub fn data(&self) -> Option<&mut T> {
        if self._llhttp.data.is_null() {
            None
        } else {
            unsafe { Some(&mut *(self._llhttp.data as *mut T)) }
        }
    }

    #[inline]
    /// Retrieve old data, and set new data
    pub fn set_data(&mut self, data: Option<Box<T>>) -> Option<Box<T>> {
        let old = if !self._llhttp.data.is_null() {
            unsafe { Some(Box::from_raw(self._llhttp.data as *mut T)) }
        } else {
            None
        };

        match data {
            Some(data) => self._llhttp.data = Box::into_raw(data) as *mut libc::c_void,
            None => self._llhttp.data = std::ptr::null_mut(),
        }

        old
    }

    #[inline]
    pub fn method(&self) -> Method {
        ffi::llhttp_method(self._llhttp.method as c_uint)
    }

    #[inline]
    pub fn method_name(&self) -> &str {
        unsafe {
            let method = llhttp_method_name(self.method());
            let method = std::ffi::CStr::from_ptr(method);
            match method.to_str() {
                Err(_) => unreachable!(),
                Ok(method) => method,
            }
        }
    }

    #[inline]
    pub fn lltype(&self) -> Type {
        ffi::llhttp_type_t(self._llhttp.type_ as c_uint)
    }

    #[inline]
    pub fn reset(&mut self) {
        unsafe { ffi::llhttp_reset(&self._llhttp as *const _ as *mut _) }
    }

    #[inline]
    pub fn major(&self) -> u8 {
        self._llhttp.http_major
    }

    #[inline]
    pub fn minor(&self) -> u8 {
        self._llhttp.http_minor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(self) use crate as llhttp;

    #[test]
    fn test_method() {
        let settings = Settings::new();
        let mut parser = Parser::<()>::default();
        parser.init(&settings, Type::HTTP_BOTH);

        let payload = r#"NOTIFY * HTTP/1.1\r
        HOST: 239.255.255.250:1900\r
        CACHE-CONTROL: max-age=60\r
        LOCATION: http://192.168.2.1:5000/rootDesc.xml\r
        SERVER: K2A5/OpenWrt/Barrier_Breaker__unknown_ UPnP/1.1 MiniUPnPd/1.8\r
        NT: upnp:rootdevice\r
        USN: uuid:0aeec1da-795c-448c-864b-11b838fe5945::upnp:rootdevice\r
        NTS: ssdp:alive\r
        OPT: "http://schemas.upnp.org/upnp/1/0/"; ns=01\r
        01-NLS: 1\r
        BOOTID.UPNP.ORG: 1\r
        CONFIGID.UPNP.ORG: 1337\r\n\r\n"#;
        parser.parse(payload.as_bytes());
        assert!(matches!(parser.method(), Method::HTTP_NOTIFY));

        let payload = b"GET /ocsp-devid01/ME4wTKADAgEAMEUwQzBBMAkGBSsOAwIaBQAEFDOB0e%2FbaLCFIU0u76%2BMSmlkPCpsBBRXF%2B2iz9x8mKEQ4Py%2Bhy0s8uMXVAIITaFtmUYgLaY%3D HTTP/1.1\r\n
        Host: ocsp.apple.com\r\n
        Accept: */*\r\n
        Accept-Language: zh-cn\r\n
        Connection: keep-alive\r\n
        Accept-Encoding: gzip, deflate\r\n
        User-Agent: com.apple.trustd/2.0\r\n";

        let mut parser = Parser::<()>::default();
        parser.init(&settings, Type::HTTP_BOTH);

        parser.parse(payload);
        println!("{:?}", parser.method());
        assert!(matches!(parser.method(), Method::HTTP_GET));
    }

    #[derive(Default, Debug)]
    struct TmpStore(pub std::collections::HashMap<String, String>);
    impl From<*mut ffi::llhttp_t> for &mut TmpStore {
        fn from(value: *mut ffi::llhttp_t) -> Self {
            unsafe { ((*value).data as *mut TmpStore).as_mut().unwrap() }
        }
    }

    fn on_url(parser: &mut Parser<TmpStore>, data: &[u8]) -> anyhow::Result<()> {
        let tmp_store = parser.data().unwrap();
        let data = unsafe { CStr::from_bytes_with_nul_unchecked(data).to_str().unwrap() };
        let value = tmp_store
            .0
            .entry("url".to_string())
            .or_insert(String::new());
        let data = value.to_owned() + data;
        *value = data;
        Ok(())
    }

    fn on_header_field(parser: &mut Parser<TmpStore>, data: &[u8]) -> anyhow::Result<()> {
        let tmp_store = parser.data().unwrap();
        let data = unsafe { CStr::from_bytes_with_nul_unchecked(data).to_str().unwrap() };
        let value = tmp_store
            .0
            .entry("header_field/".to_string() + &data)
            .or_insert(String::new());

        let data = value.to_owned() + data;
        *value = data;
        Ok(())
    }

    fn on_header_value(parser: &mut Parser<TmpStore>, data: &[u8]) -> anyhow::Result<()> {
        let tmp_store = parser.data().unwrap();
        let data = unsafe { CStr::from_bytes_with_nul_unchecked(data).to_str().unwrap() };
        let value = tmp_store
            .0
            .entry("header_value/".to_string() + &data)
            .or_insert(String::new());

        let data = value.to_owned() + data;
        *value = data;
        Ok(())
    }

    fn on_body(parser: &mut Parser<TmpStore>, data: &[u8]) -> anyhow::Result<()> {
        let tmp_store = parser.data().unwrap();
        let data = unsafe { CStr::from_bytes_with_nul_unchecked(data).to_str().unwrap() };
        let value = tmp_store
            .0
            .entry("body".to_string())
            .or_insert(String::new());

        let data = value.to_owned() + data;
        *value = data;
        Ok(())
    }

    fn on_message_complete(parser: &mut Parser<TmpStore>) -> anyhow::Result<()> {
        let tmp_store = parser.data().unwrap();
        let value = tmp_store
            .0
            .entry("on_message_complete".to_string())
            .or_insert(String::new());
        *value = "on_message_complete".to_string();

        Ok(())
    }

    data_cb_wrapper!(on_url_wrapped, on_url, TmpStore);
    data_cb_wrapper!(on_header_field_warpped, on_header_field, TmpStore);
    data_cb_wrapper!(on_header_value_wrapped, on_header_value, TmpStore);
    data_cb_wrapper!(on_body_wrapped, on_body, TmpStore);
    cb_wrapper!(on_message_complete_wrapped, on_message_complete, TmpStore);

    #[test]
    fn test_callback() {
        use map_macro::map;
        use std::collections::HashMap;

        let mut settings = Settings::new();

        settings.on_url(Some(on_url_wrapped));
        settings.on_header_field(Some(on_header_field_warpped));
        settings.on_header_value(Some(on_header_value_wrapped));
        settings.on_body(Some(on_body_wrapped));
        settings.on_message_complete(Some(on_message_complete_wrapped));

        let payload = b"POST /user_info HTTP/1.1\r\nHost: localhost:5555\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\nContent-Length: 17\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\nusername=cattchen";

        let mut parser = Parser::<TmpStore>::default();
        parser.init(&settings, Type::HTTP_BOTH);
        parser.set_data(Some(Box::new(TmpStore::default())));

        parser.parse(payload);
        let tmp_store = parser.data().unwrap();

        assert!(matches!(parser.method(), Method::HTTP_POST));

        let m: HashMap<String, String> = map! {
            "header_value/localhost:5555".to_string() => "localhost:5555".to_string(),
            "header_value/application/x-www-form-urlencoded".to_string()=> "application/x-www-form-urlencoded".to_string(),
            "header_field/User-Agent".to_string()=> "User-Agent".to_string(),
            "header_field/Content-Length".to_string()=> "Content-Length".to_string(),
            "header_value/17".to_string()=> "17".to_string(),
            "url".to_string()=> "/user_info".to_string(),
            "header_field/Accept".to_string()=> "Accept".to_string(),
            "header_value/curl/7.81.0".to_string()=> "curl/7.81.0".to_string(),
            "header_value/*/*".to_string()=> "*/*".to_string(),
            "body".to_string()=> "username=cattchen".to_string(),
            "header_field/Content-Type".to_string()=> "Content-Type".to_string(),
            "on_message_complete".to_string()=> "on_message_complete".to_string(),
            "header_field/Host".to_string()=> "Host".to_string()
        };

        for (key, value) in m {
            assert!(tmp_store.0.get(&key).unwrap() == &value);
        }
    }
}
