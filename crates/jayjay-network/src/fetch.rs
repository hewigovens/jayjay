use crate::{NetError, Request, Response};

pub trait Fetch {
    fn fetch(&self, request: &Request) -> Result<Response, NetError>;
}

impl<T: Fetch + ?Sized> Fetch for &T {
    fn fetch(&self, request: &Request) -> Result<Response, NetError> {
        (**self).fetch(request)
    }
}
