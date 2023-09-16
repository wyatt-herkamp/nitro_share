use std::fmt::Debug;

use actix_web::{
    body::BoxBody,
    http::{
        header::{CACHE_CONTROL, ETAG, EXPIRES, IF_NONE_MATCH, LAST_MODIFIED},
        StatusCode,
    },
    HttpRequest, HttpResponse, Responder,
};
use common::response_type::{ResponseType, ToCacheControl};
use serde::Serialize;

use crate::{utils::sha256, Error};

#[derive(Debug)]
pub struct JsonResponse<T: Serialize + ResponseType>(pub T, pub StatusCode);
impl<T: Serialize + ResponseType> From<T> for JsonResponse<T> {
    fn from(result: T) -> Self {
        JsonResponse(result, StatusCode::OK)
    }
}

impl<T: Serialize + ResponseType> Responder for JsonResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let Self(result, status_code) = self;
        let etag = sha256::digest_to_string(&result);

        if let Some(requested_etag) = req
            .headers()
            .get(IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
        {
            if requested_etag == etag {
                return HttpResponse::NotModified().finish();
            }
        }

        let mut response = HttpResponse::build(status_code);
        response.insert_header((ETAG, etag));

        let vec = result.cache_control_params();
        if vec.len() > 0 {
            response.insert_header((CACHE_CONTROL, vec.to_cache_control()));
        }
        if let Some(expires) = result.expires() {
            response.insert_header((EXPIRES, expires.to_rfc2822()));
        }
        if let Some(last_modified) = result.last_modified() {
            response.insert_header((LAST_MODIFIED, last_modified.to_rfc2822()));
        }

        response.json(result)
    }
}
pub type JsonOrErrorResult<T> = Result<JsonOrError<T>, Error>;
#[derive(Debug)]
pub enum JsonOrError<T: Serialize + ResponseType> {
    Json(JsonResponse<T>),
    Error(HttpResponse),
}

impl<T: Serialize + ResponseType> From<HttpResponse> for JsonOrError<T> {
    fn from(value: HttpResponse) -> Self {
        JsonOrError::Error(value)
    }
}

impl<T: Serialize + ResponseType> From<(T, StatusCode)> for JsonOrError<T> {
    fn from((result, status_code): (T, StatusCode)) -> Self {
        JsonOrError::Json(JsonResponse(result, status_code))
    }
}

impl<T: Serialize + ResponseType> Responder for JsonOrError<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        return match self {
            JsonOrError::Json(result) => result.respond_to(req),
            JsonOrError::Error(err) => err.into(),
        };
    }
}
