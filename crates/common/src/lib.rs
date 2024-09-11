use std::fmt::Write as _;

pub mod api;
pub mod generate;
mod id;
pub mod upload;

pub use id::*;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate log;

pub struct AddonPermission {
    pub scope: String,
    pub category: String,
    pub operation: Option<String>,
    pub info: Option<String>,
}

impl ToString for AddonPermission {
    fn to_string(&self) -> String {
        let mut value = format!("{}.{}", self.scope, self.category);

        if let Some(val) = self.operation.as_deref() {
            write!(&mut value, ".{val}").unwrap();
        }

        if let Some(val) = self.info.as_deref() {
            write!(&mut value, ".{val}").unwrap();
        }

        value
    }
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

impl<T> ListResponse<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            offset: 0,
            limit: 0,
            total: 0,
        }
    }

    pub fn all(value: Vec<T>) -> Self {
        Self {
            offset: 0,
            limit: value.len(),
            total: value.len(),
            items: value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleValue<V>(pub V);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum WrappingResponse<V> {
    Resp(V),
    Error(ApiErrorResponse),
}

impl<V> WrappingResponse<V> {
    pub fn okay(value: V) -> Self {
        Self::Resp(value)
    }

    pub fn error<S: Into<String>>(value: S) -> Self {
        Self::Error(ApiErrorResponse::new(value))
    }

    pub fn ok(self) -> std::result::Result<V, ApiErrorResponse> {
        match self {
            Self::Resp(v) => Ok(v),
            Self::Error(e) => Err(e),
        }
    }

    pub fn as_ok(&self) -> std::result::Result<&V, &ApiErrorResponse> {
        match self {
            Self::Resp(v) => Ok(v),
            Self::Error(e) => Err(e),
        }
    }

    pub fn map<N, F: Fn(V) -> N>(self, func: F) -> WrappingResponse<N> {
        match self {
            Self::Resp(v) => WrappingResponse::Resp(func(v)),
            Self::Error(e) => WrappingResponse::Error(e),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, thiserror::Error)]
pub struct ApiErrorResponse {
    pub description: String,
}

impl ApiErrorResponse {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self {
            description: value.into(),
        }
    }
}

impl std::fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Api Error Occurred: {}", self.description)
    }
}
