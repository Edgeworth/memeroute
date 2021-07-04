use crate::route::router::{RouteResult, RouteStrategy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRouter {}

impl RouteStrategy for GridRouter {
    fn route() -> RouteResult {
        todo!()
    }
}
