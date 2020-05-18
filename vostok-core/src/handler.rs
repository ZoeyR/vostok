use std::{collections::HashMap, future::Future, pin::Pin};

use crate::request::{Bundle, Request};

pub(crate) struct RequestRouter<Req: 'static, Res: 'static> {
    commands: HashMap<&'static str, &'static dyn RequestHandler<Req, Res>>,
}

impl<Req, Res> RequestRouter<Req, Res> {
    pub fn new() -> Self {
        RequestRouter {
            commands: HashMap::new(),
        }
    }

    pub fn add_handlers(&mut self, handlers: Vec<&'static dyn RequestHandler<Req, Res>>) {
        for handler in handlers {
            self.commands.insert(handler.route_id(), handler);
        }
    }

    pub async fn route<'a, 'b>(&self, bundle: &'a Bundle<'b, Req>) -> Option<Res>
    where
        Req: Request,
    {
        let path = bundle.request.path();
        let handler = self.commands.get(path)?;
        Some(handler.handle(&bundle).await)
    }
}

pub trait RequestHandler<Req, Res>: Send + Sync {
    fn route_id(&self) -> &'static str;

    fn handle<'a, 'b>(
        &'a self,
        request: &'a Bundle<'b, Req>,
    ) -> Pin<Box<dyn Future<Output = Res> + Send + 'a>>;
}

impl<T, Req, Res> RequestHandler<Req, Res> for (&'static str, &'static T)
where
    T: for<'a, 'b> Fn(&'a Bundle<'b, Req>) -> Pin<Box<dyn Future<Output = Res> + Send + 'a>>
        + Send
        + Sync,
{
    fn route_id(&self) -> &'static str {
        self.0
    }

    fn handle<'a, 'b>(
        &'a self,
        request: &'a Bundle<'b, Req>,
    ) -> Pin<Box<dyn Future<Output = Res> + Send + 'a>> {
        self.1(request)
    }
}
