use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};

use actix_web::{
    body::BoxBody, http::StatusCode, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    ResponseError,
};
use anyhow::anyhow;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResult<T = ()> {
    #[serde(skip_serializing)] // Status code set via header, does not need to be in the response
    pub code: u16,
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none", rename = "cause")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResult<T> {
    pub fn new() -> Self {
        Self {
            code: 200,
            success: true,
            error: None,
            data: None,
        }
    }

    pub fn ok() -> Self {
        Self {
            code: 200,
            success: true,
            error: None,
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: 500,
            success: false,
            error: None,
            data: None,
        }
    }

    pub fn error<S: Into<String>>(code: StatusCode, error_msg: S) -> Self {
        Self {
            code: code.as_u16(),
            success: false,
            error: Some(error_msg.into()),
            data: None,
        }
    }

    pub fn data(code: StatusCode, data: T) -> Self {
        Self {
            code: code.as_u16(),
            success: false,
            error: None,
            data: Some(data),
        }
    }

    // pub fn code(code: StatusCode) -> Self {
    //     Self {
    //         code: code.as_u16(),
    //         success: true,
    //         error: None,
    //         data: None,
    //     }
    // }
    //
    // pub fn code_raw(mut self, code: u16) -> Self {
    //     self.code = code;
    //     self
    // }

    pub fn add_error<S: Into<String>>(mut self, msg: S) -> Self {
        self.error = Some(msg.into());
        self
    }

    // pub fn add_data(mut self, data: T) -> Self
    // where
    //     T: Serialize,
    // {
    //     self.data = Some(data);
    //     self
    // }

    pub fn convert_to_response(mut self) -> HttpResponse {
        self.success = self.code >= 200 && self.code < 300;
        let status = StatusCode::from_u16(self.code).unwrap();
        HttpResponseBuilder::new(status).json(self)
    }
}

impl<T: Serialize> Responder for ApiResult<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        self.convert_to_response()
    }
}

pub struct Error {}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        ApiResult::<String>::internal_error()
            .add_error("Internal Server error")
            .convert_to_response()
    }
}

macro_rules! impl_error {
    ($t:ty) => {
        impl From<$t> for Error {
            fn from(_value: $t) -> Self {
                Error {}
            }
        }
    };
}

impl_error!(sqlx::Error);
impl_error!(anyhow::Error);
impl_error!(argon2::password_hash::Error);
