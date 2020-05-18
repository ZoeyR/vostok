use handler::{RequestHandler, RequestRouter};
use request::{Bundle, Request};
use state::Container;

pub use vostok_codegen::request;
pub use vostok_codegen::routes;

pub mod handler;
pub mod request;
pub mod response;

pub struct Vostok<Req: 'static, Res: 'static> {
    state: Container,
    router: RequestRouter<Req, Res>,
}

impl<Req: Request, Res> Vostok<Req, Res> {
    pub fn build() -> Self {
        Vostok {
            state: Container::new(),
            router: RequestRouter::new(),
        }
    }

    pub fn manage<T: Send + Sync + 'static>(self, state: T) -> Self {
        self.state.set(state);

        self
    }

    pub fn route(mut self, routes: Vec<&'static dyn RequestHandler<Req, Res>>) -> Self {
        self.router.add_handlers(routes);

        self
    }

    pub async fn handle(&self, req: Req) -> Option<Res> {
        let bundle = Bundle {
            state: &self.state,
            request: req,
        };

        self.router.route(&bundle).await
    }
}
