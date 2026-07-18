pub type HandlerResponse = (u16, String);
pub type HandlerFn = Box<dyn Fn(&str, &str) -> HandlerResponse + Send + Sync>;

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
        F: Fn(&str, &str) -> HandlerResponse + 'static + Send + Sync,
    {
        self.routes.push((
            Route {
                method: method.to_string(),
                path: path.to_string(),
            },
            Box::new(handler),
        ));
    }

    pub fn route(&self, method: &str, path: &str) -> Option<HandlerResponse> {
        for (route, handler) in &self.routes {
            if route.method == method && route.path == path {
                return Some(handler(method, path));
            }
        }
        None
    }
}
