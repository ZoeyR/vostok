use std::ops::Deref;

use state::Container;

use crate::Vostok;

pub struct Bundle<'b, Req> {
    pub request: Req,
    pub state: &'b Container,
}

impl<'b, Req> Bundle<'b, Req> {
    pub fn from_request<Res>(vostok: &'b Vostok<Req, Res>, request: Req) -> Option<Self> {
        Some(Bundle {
            request,
            state: &vostok.state,
        })
    }
}

pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
    pub fn inner(&self) -> &'r T {
        self.0
    }
}

impl<'r, T: Send + Sync + 'static> Deref for State<'r, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pub trait Request {
    fn path(&self) -> &str;
}

pub trait FromRequest: Sized {
    type Request;
    fn from_request(request: &Self::Request) -> Option<Self>;
}

pub trait FromBundle<'a, 'b, R>: Sized {
    fn from_bundle(bundle: &'a Bundle<'b, R>) -> Option<Self>;
}

impl<'a, 'b, R, T: Send + Sync + 'static> FromBundle<'a, 'b, R> for State<'b, T> {
    fn from_bundle(bundle: &'a Bundle<'b, R>) -> Option<Self> {
        bundle.state.try_get::<T>().map(State)
    }
}

impl<'a, 'b, R, T: Send + Sync + 'static> FromBundle<'a, 'b, R> for T
where
    T: FromRequest<Request = R>,
{
    fn from_bundle(bundle: &'a Bundle<'b, R>) -> Option<Self> {
        T::from_request(&bundle.request)
    }
}
