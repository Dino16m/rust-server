use std::vec;

use http::StatusCode;

use crate::{
    request::{HTTPContext, HTTPMethod},
    response::HttpResponse,
};

pub struct HTTPRequest {
    pub context: HTTPContext,
}

enum HTTPPath {
    Parameterized(String),
    Plain(String),
}

pub enum PreRequestMiddlewareResult {
    Next,
    End(HttpResponse),
}

pub enum PostRequestMiddlewareResult {
    Next(HttpResponse),
    End(HttpResponse),
}
pub trait PreRequestMiddleware {
    fn handle(&self, request: &mut HTTPRequest) -> PreRequestMiddlewareResult;
}

pub trait PostRequestMiddleware {
    fn handle(&self, request: &HTTPRequest, response: &HttpResponse)
        -> PostRequestMiddlewareResult;
}

enum MatchedPath {
    Plain,
    Parameterized { key: String, value: String },
}

impl HTTPPath {
    fn match_segment(&self, path: &str) -> Option<MatchedPath> {
        return match self {
            HTTPPath::Parameterized(segment) => Some(MatchedPath::Parameterized {
                key: segment.to_string(),
                value: path.to_string(),
            }),
            HTTPPath::Plain(segment) => {
                if path == segment {
                    return Some(MatchedPath::Plain);
                }
                return None;
            }
        };
    }
}

fn is_parameterized_path(path: &str) -> bool {
    return path.starts_with(":");
}

fn split_path(path: &str) -> Vec<&str> {
    let mut split_paths: Vec<&str> = path
        .split("/")
        .map(|segment| {
            if segment.trim().is_empty() {
                return "/";
            }
            return segment;
        })
        .collect();
    if !split_paths.ends_with(&["/"]) {
        split_paths.push("/");
    }
    return split_paths;
}

fn build_paths(path: String) -> Vec<HTTPPath> {
    split_path(&path)
        .iter()
        .map(|segment| {
            if is_parameterized_path(segment) {
                return HTTPPath::Parameterized(segment.to_string());
            }
            return HTTPPath::Plain(segment.to_string());
        })
        .collect()
}

struct RouteMiddleware {
    pre_request_handlers: Vec<Box<dyn PreRequestMiddleware>>,
    post_request_handlers: Vec<Box<dyn PostRequestMiddleware>>,
}

impl RouteMiddleware {
    fn pre_request_hook(&self, request: &mut HTTPRequest) -> Option<HttpResponse> {
        for handler in self.pre_request_handlers.iter() {
            let res = handler.handle(request);
            match res {
                PreRequestMiddlewareResult::Next => continue,
                PreRequestMiddlewareResult::End(response) => return Some(response),
            };
        }
        return None;
    }

    fn post_request_hook(&self, request: &HTTPRequest, response: HttpResponse) -> HttpResponse {
        let mut response_copy = response;
        for handler in self.post_request_handlers.iter() {
            let res = handler.handle(request, &response_copy);
            match res {
                PostRequestMiddlewareResult::Next(response) => {
                    response_copy = response;
                }
                PostRequestMiddlewareResult::End(response) => return response,
            };
        }
        return response_copy;
    }
}

struct RouteHandler {
    inner_handler: Option<Box<dyn Fn(&mut HTTPRequest) -> HttpResponse>>,
}

impl RouteHandler {
    fn new<T>(inner_handler: T) -> Self
    where
        T: Fn(&mut HTTPRequest) -> HttpResponse + 'static,
    {
        return RouteHandler {
            inner_handler: Some(Box::new(inner_handler)),
        };
    }

    fn call(&self, request: &mut HTTPRequest) -> Option<HttpResponse> {
        return match &self.inner_handler {
            Some(inner_handler) => Some((inner_handler)(request)),
            None => None,
        };
    }
}

pub struct RouteMapping {
    method: Option<HTTPMethod>,
    segments: Vec<HTTPPath>,
    handler: Option<RouteHandler>,
    middleware: Option<RouteMiddleware>,
}

impl RouteMapping {
    fn new<T>(method: Option<HTTPMethod>, path: String, handler: T) -> Self
    where
        T: Fn(&mut HTTPRequest) -> HttpResponse + 'static,
    {
        return RouteMapping {
            method,
            segments: build_paths(path),
            handler: Some(RouteHandler::new(handler)),
            middleware: None,
        };
    }

    fn from_path(path: &str) -> Self {
        return RouteMapping {
            method: None,
            segments: build_paths(path.to_string()),
            handler: None,
            middleware: None,
        };
    }

    fn add_post_request_middleware(&mut self, post_request: Box<dyn PostRequestMiddleware>) {
        if self.middleware.is_none() {
            self.middleware = Some(RouteMiddleware {
                pre_request_handlers: vec![],
                post_request_handlers: vec![post_request],
            })
        } else {
            let middleware = self.middleware.as_mut().unwrap();
            middleware.post_request_handlers.push(post_request)
        }
    }

    fn add_pre_request_middleware(&mut self, pre_request: Box<dyn PreRequestMiddleware>) {
        if self.middleware.is_none() {
            self.middleware = Some(RouteMiddleware {
                pre_request_handlers: vec![pre_request],
                post_request_handlers: vec![],
            })
        } else {
            let middleware = self.middleware.as_mut().unwrap();
            middleware.pre_request_handlers.push(pre_request)
        }
    }

    fn pre_request_hook(&self, request: &mut HTTPRequest) -> Option<HttpResponse> {
        match &self.middleware {
            Some(middleware) => middleware.pre_request_hook(request),
            None => None,
        }
    }

    fn post_request_hook(&self, request: &mut HTTPRequest, response: HttpResponse) -> HttpResponse {
        match &self.middleware {
            Some(middleware) => middleware.post_request_hook(request, response),
            None => response,
        }
    }

    fn request_handler(&self, request: &mut HTTPRequest) -> Option<HttpResponse> {
        match &self.handler {
            Some(handler) => handler.call(request),
            None => None,
        }
    }

    fn handle(&self, request: &mut HTTPRequest) -> Option<HttpResponse> {
        let response = self.pre_request_hook(request);
        if response.is_some() {
            return response;
        };
        return match self.request_handler(request) {
            Some(response) => Some(self.post_request_hook(request, response)),
            None => None,
        };
    }

    fn match_method(&self, method: &HTTPMethod) -> bool {
        match &self.method {
            Some(m) => m == method,
            None => true,
        }
    }

    fn match_path(&self, path: &str) -> bool {
        let split_paths = split_path(path);
        if split_paths.len() != self.segments.len() {
            return false;
        }
        split_paths
            .iter()
            .zip(self.segments.iter())
            .all(|(&path, segment)| segment.match_segment(path).is_some())
    }

    pub fn match_full(&self, path: &str, method: &HTTPMethod) -> bool {
        if !self.match_method(method) {
            return false;
        }
        return self.match_path(path);
    }
}

pub struct Router {
    routes: Vec<RouteMapping>,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            routes: Default::default(),
        }
    }
}

impl Router {
    pub fn new() -> Self {
        return Router { routes: vec![] };
    }

    pub fn route<T>(&mut self, method: HTTPMethod, path: &str, handler: T) -> &mut Self
    where
        T: Fn(&mut HTTPRequest) -> HttpResponse + 'static,
    {
        self.routes
            .push(RouteMapping::new(method.into(), path.to_string(), handler));
        return self;
    }

    pub fn nest(&mut self, router: Router) -> &mut Self {
        self.routes.extend(router.routes.into_iter());
        return self;
    }

    pub fn before<T>(&mut self, path: &str, middleware: T) -> &mut Self
    where
        T: PreRequestMiddleware + 'static,
    {
        let mapping = self
            .routes
            .iter_mut()
            .find(|mapping| mapping.match_path(path));
        if mapping.is_some() {
            mapping
                .unwrap()
                .add_pre_request_middleware(Box::new(middleware))
        } else {
            let mut mapping = RouteMapping::from_path(path);
            mapping.add_pre_request_middleware(Box::new(middleware));
            self.routes.push(mapping);
        }
        return self;
    }

    pub fn after<T>(&mut self, path: &str, middleware: T) -> &mut Self
    where
        T: PostRequestMiddleware + 'static,
    {
        let mapping = self
            .routes
            .iter_mut()
            .find(|mapping| mapping.match_path(path));
        if mapping.is_some() {
            mapping
                .unwrap()
                .add_post_request_middleware(Box::new(middleware))
        } else {
            let mut mapping = RouteMapping::from_path(path);
            mapping.add_post_request_middleware(Box::new(middleware));
            self.routes.push(mapping);
        }
        return self;
    }

    pub fn handle(&self, context: HTTPContext) -> HttpResponse {
        let handlers = self.get_handlers(&context.path, &context.method);

        let request = &mut HTTPRequest { context };

        let mut response: Option<HttpResponse> = None;
        for &handler in handlers.iter() {
            let handler_response = handler.handle(request);
            if handler_response.is_some() {
                response = handler_response;
            }
        }

        return match response {
            Some(response) => response,
            None => {
                let mut response = HttpResponse::new();
                response.set_status(StatusCode::NOT_FOUND);
                return response;
            }
        };
    }

    fn get_handlers(&self, path: &str, method: &HTTPMethod) -> Vec<&RouteMapping> {
        return self
            .routes
            .iter()
            .filter(|&route| route.match_full(path, method))
            .map(|route| route)
            .collect::<Vec<_>>();
    }
}
