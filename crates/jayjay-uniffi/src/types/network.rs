use jayjay_network::{Header, Method, Request, Response};

#[uniffi::remote(Enum)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[uniffi::remote(Record)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[uniffi::remote(Record)]
pub struct Request {
    pub url: String,
    pub method: Method,
    pub headers: Vec<Header>,
    pub body: Option<Vec<u8>>,
    pub max_response_bytes: u32,
}

#[uniffi::remote(Record)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}
