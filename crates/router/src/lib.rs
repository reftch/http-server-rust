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

    pub fn route(&self, request: &mut Request) -> Option<HandlerResponse> {
        for (route, handler) in &self.routes {
            if route.method == request.method && self.match_path(route, request) {
                let mut response = Response::new(200, ""); 
                handler(request, &mut response);
                return Some(response);
            }
        }
        None
    }

    fn match_path(&self, route: &Route, request: &mut Request) -> bool {
        let route_parts: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();
        let request_parts: Vec<&str> = request.path.split('/').filter(|s| !s.is_empty()).collect();

        if route_parts.len() != request_parts.len() {
            return false;
        }

        for (route_part, request_part) in route_parts.iter().zip(request_parts.iter()) {
            if route_part.starts_with(':') {
                let key = &route_part[1..];
                request.params.insert(key.to_string(), request_part.to_string());
            } else if route_part != request_part {
                return false;
            }
        }

        true
    }
}
