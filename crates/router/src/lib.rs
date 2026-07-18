use http_core::{Request, Response};
use std::collections::HashMap;

pub type HandlerResponse = Response;
pub type HandlerFn = fn(&Request, &mut Response);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl Method {
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Method::GET => 0,
            Method::POST => 1,
            Method::PUT => 2,
            Method::PATCH => 3,
            Method::DELETE => 4,
            Method::HEAD => 5,
            Method::OPTIONS => 6,
        }
    }

    #[inline]
    pub fn from_str(method: &str) -> Option<Self> {
        match method {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "PATCH" => Some(Method::PATCH),
            "DELETE" => Some(Method::DELETE),
            "HEAD" => Some(Method::HEAD),
            "OPTIONS" => Some(Method::OPTIONS),
            _ => None,
        }
    }
}

const METHOD_COUNT: usize = 7;

struct ParamChild {
    name: Box<str>,
    node: Box<TrieNode>,
}

struct TrieNode {
    children: HashMap<Box<str>, Box<TrieNode>>,
    param_child: Option<ParamChild>,
    handlers: [Option<HandlerFn>; METHOD_COUNT],
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            param_child: None,
            handlers: [None; METHOD_COUNT],
        }
    }
}

pub struct Router {
    root: TrieNode,
}

impl Router {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    pub fn add_route(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let mut current = &mut self.root;

        for part in path.split('/').filter(|s| !s.is_empty()) {
            if let Some(name) = part.strip_prefix(':') {
                let pc = current.param_child.get_or_insert_with(|| ParamChild {
                    name: name.into(),
                    node: Box::new(TrieNode::new()),
                });

                current = pc.node.as_mut();
            } else {
                current = current
                    .children
                    .entry(part.into())
                    .or_insert_with(|| Box::new(TrieNode::new()))
                    .as_mut();
            }
        }

        current.handlers[method.index()] = Some(handler);
    }

    pub fn route(&self, request: &mut Request) -> Option<Response> {
        let mut current = &self.root;

        for part in request.path.split('/').filter(|s| !s.is_empty()) {
            if let Some(next) = current.children.get(part) {
                current = next.as_ref();
            } else if let Some(pc) = &current.param_child {
                request.params.insert(pc.name.to_string(), part.to_string());
                current = pc.node.as_ref();
            } else {
                return None;
            }
        }

        let method = Method::from_str(request.method.as_str())?;

        let handler = current.handlers[method.index()]?;

        let mut response = Response::new(200, "");
        handler(request, &mut response);

        Some(response)
    }
}
