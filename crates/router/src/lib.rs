use std::collections::HashMap;
use http_core::{Request, Response};

pub type HandlerResponse = Response;
pub type HandlerFn = Box<dyn Fn(&Request, &mut Response) + Send + Sync>;

struct ParamChild {
    name: String,
    node: Box<TrieNode>,
}

struct TrieNode {
    children: HashMap<String, TrieNode>,
    param_child: Option<ParamChild>,
    handlers: HashMap<String, HandlerFn>,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            param_child: None,
            handlers: HashMap::new(),
        }
    }
}

pub struct Router {
    root: TrieNode,
}

impl Router {
    pub fn new() -> Self {
        Router {
            root: TrieNode::new(),
        }
    }

    pub fn add_route<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static + Send + Sync,
    {
        let mut current = &mut self.root;
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        for part in parts {
            if part.starts_with(':') {
                let name = part[1..].to_string();
                if current.param_child.is_none() {
                    current.param_child = Some(ParamChild {
                        name,
                        node: Box::new(TrieNode::new()),
                    });
                }
                // We need to get the node out of the Option<ParamChild>
                if let Some(ref mut pc) = current.param_child {
                    current = &mut *pc.node;
                }
            } else {
                current = current.children.entry(part.to_string()).or_insert_with(TrieNode::new);
            }
        }
        current.handlers.insert(method.to_string(), Box::new(handler));
    }

    pub fn route(&self, request: &mut Request) -> Option<HandlerResponse> {
        let parts: Vec<&str> = request.path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;

        for part in parts {
            if let Some(next) = current.children.get(part) {
                current = next;
            } else if let Some(ref pc) = current.param_child {
                request.params.insert(pc.name.clone(), part.to_string());
                current = &*pc.node;
            } else {
                return None;
            }
        }

        if let Some(handler) = current.handlers.get(&request.method) {
            let mut response = Response::new(200, "");
            handler(request, &mut response);
            return Some(response);
        }

        None
    }
}

