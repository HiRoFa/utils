//! This contains abstract traits and structs for use with different javascript runtimes
//! the Adapter traits are use in the worker thread (EventLoop) of the Runtime and thus are not Send, they should never leave the thread
//! The facade classes are for use outside the worker thread, they are Send
//!

use std::future::Future;
use std::str::FromStr;

pub mod adapters;
pub mod facades;
pub mod fetch_impl;

pub trait JsError {
    fn get_message(&self) -> &str;
    fn get_filename(&self) -> Option<&str>;
    fn get_line_num(&self) -> usize;
    fn get_position(&self) -> usize;
}

pub struct Script {
    code: String,
    path: String,
}

impl Script {
    pub fn get_code(&self) -> &str {
        self.code.as_str()
    }
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }
    pub fn set_code(&mut self, code: &str) {
        self.code = code.to_string();
    }
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }
}

pub trait FetchResponder {
    fn fetch(&self, url: &str, init: dyn FetchInit) -> dyn Future<Output = Box<dyn Response>>;
}

pub enum Mode {
    Cors,
    NoCors,
    SameOrigin,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cors => "cors",
            Self::NoCors => "no-cors",
            Self::SameOrigin => "same-origin",
        }
    }
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "cors" => Ok(Self::Cors),
            "no-cors" => Ok(Self::NoCors),
            "same-origin" => Ok(Self::SameOrigin),
            _ => Err(()),
        }
    }
}

pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    Copy,
    Lock,
    Mkcol,
    Move,
    Propfind,
    Proppatch,
    Unlock,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
            Method::Copy => "COPY",
            Method::Lock => "LOCK",
            Method::Mkcol => "MKCOL",
            Method::Move => "MOVE",
            Method::Propfind => "PROPFIND",
            Method::Proppatch => "PROPPATCH",
            Method::Unlock => "UNLOCK",
        }
    }
}

impl FromStr for Method {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val.to_ascii_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "PATCH" => Ok(Self::Patch),
            "COPY" => Ok(Self::Copy),
            "LOCK" => Ok(Self::Lock),
            "MKCOL" => Ok(Self::Mkcol),
            "MOVE" => Ok(Self::Move),
            "PROPFIND" => Ok(Self::Propfind),
            "PROPPATCH" => Ok(Self::Proppatch),
            "UNLOCK" => Ok(Self::Unlock),

            _ => Err(()),
        }
    }
}

pub enum Credentials {
    Omit,
    SameOrigin,
    Include,
}

impl Credentials {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Omit => "omit",
            Self::SameOrigin => "same-origin",
            Self::Include => "include",
        }
    }
}

impl FromStr for Credentials {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "omit" => Ok(Self::Omit),
            "same-origin" => Ok(Self::SameOrigin),
            "include" => Ok(Self::Include),
            _ => Err(()),
        }
    }
}

pub enum Cache {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

impl Cache {
    pub fn as_str(&self) -> &'static str {
        match self {
            Cache::Default => "default",
            Cache::NoStore => "no-store",
            Cache::Reload => "reload",
            Cache::NoCache => "no-cache",
            Cache::ForceCache => "force-cache",
            Cache::OnlyIfCached => "only-if-cached",
        }
    }
}

impl FromStr for Cache {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Self::Default),
            "no-store" => Ok(Self::NoStore),
            "reload" => Ok(Self::Reload),
            "no-cache" => Ok(Self::NoCache),
            "force-cache" => Ok(Self::ForceCache),
            "only-if-cached" => Ok(Self::OnlyIfCached),
            _ => Err(()),
        }
    }
}

pub trait FetchInit {
    fn get_method(&self) -> Method;
    fn get_headers(&self) -> dyn Headers;
    fn get_body(&self) -> dyn Body;
    fn get_mode(&self) -> Mode;
    fn get_credentials(&self) -> Credentials;
    fn get_cache(&self) -> Cache;
}

pub trait Headers {
    fn append(&mut self, name: &str, value: &str);
}

pub trait Body {}

pub trait Response {}

pub trait Request {
    fn get_url(&self) -> &str;
    fn get_header(&self, name: &str) -> &[String];
}
