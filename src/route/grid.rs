use crate::route::route::RouteStrategy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRouter {}

impl RouteStrategy for GridRouter {
    fn route() -> super::route::RouteResult {
        todo!()
    }
}
