use crate::error::LoadoutError;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct OkEnvelope<T: Serialize> {
    ok: bool,
    result: T,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ErrEnvelope {
    ok: bool,
    error: crate::error::ErrorBody,
}

pub fn ok<T: Serialize>(result: T) -> String {
    serde_json::to_string(&OkEnvelope { ok: true, result }).expect("json serialize ok")
}

pub fn err(err: &LoadoutError) -> String {
    serde_json::to_string(&ErrEnvelope {
        ok: false,
        error: err.to_error_body(),
    })
    .expect("json serialize err")
}
