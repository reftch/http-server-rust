use http_core::{Request, Response};

pub type HandlerResponse = Response;
pub type HandlerFn = Box<dyn Fn(&Request, &mut Response) + Send + Sync>;

#[derive(Clone)]
struct Route {
    method: String,
    path: String,
}

pub struct Router {
    routes: Vec<(Route, HandlerFn)>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    pub fn add_route<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static + Send + Sync,
    {
        self.routes.push((
            Route {
                method: method.to_string(),
                path: path.to_string(),
            },
            Box::new(handler),
        ));
    }

    pub fn route(&self, request: &Request) -> Option<HandlerResponse> {
        for (route, handler) in &self.routes {
            if route.method == request.method && route.path == request.path {
                let mut response = Response::new(200, ""); 
                handler(request, &mut response);
                return Some(response);
            }
        }
        None
    }
}
