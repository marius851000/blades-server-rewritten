use std::{
    error::Error,
    fmt::{Debug, Display},
};

use actix_web::{
    HttpResponse, HttpResponseBuilder, ResponseError, body::BoxBody, http::StatusCode,
};
use log::error;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug)]
pub struct BladeApiError {
    http_status_code: StatusCode,
    service_id: u64,
    error_code: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BladeApiErrorResponse {
    operation_id: String,
    error: BladeApiErrorResponseInner,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BladeApiErrorResponseInner {
    http_status_code: u16,
    service_id: u64,
    error_code: u64,
}

impl BladeApiError {
    pub fn new(status_code: StatusCode, service_id: u64, error_code: u64) -> Self {
        Self {
            http_status_code: status_code,
            service_id,
            error_code,
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, 1, 200)
    }

    /// Will log the error and prepare a generic internal error response
    pub fn generic_internal_error<E: Error>(error: E) -> Self {
        error!("error while processing a request: {:#?}", error);
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, 1, 100)
    }
}

impl<E: Error> From<E> for BladeApiError {
    fn from(value: E) -> Self {
        Self::generic_internal_error(value)
    }
}

impl Display for BladeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BladeApiError {{ http_status_code: {}, service_id: {}, error_code: {} }}",
            self.http_status_code.as_u16(),
            self.service_id,
            self.error_code
        )
    }
}

impl ResponseError for BladeApiError {
    fn status_code(&self) -> StatusCode {
        self.http_status_code.clone()
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = serde_json::to_string_pretty(&BladeApiErrorResponse {
            operation_id: Uuid::new_v4().to_string(),
            error: BladeApiErrorResponseInner {
                http_status_code: self.http_status_code.as_u16(),
                service_id: self.service_id,
                error_code: self.error_code,
            },
        })
        .unwrap();
        HttpResponseBuilder::new(self.status_code()).body(body)
    }
}
