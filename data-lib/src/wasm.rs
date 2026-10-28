//! Fallible serialization for typed JavaScript return values.

use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::JsError;

pub(crate) fn to_ts<T: Tsify + Serialize>(value: T) -> Result<Ts<T>, JsError> {
    Ok(Ts::from_rust(&value)?)
}

pub(crate) fn to_ts_vec<T: Tsify + Serialize>(values: Vec<T>) -> Result<Vec<Ts<T>>, JsError> {
    values.into_iter().map(to_ts).collect()
}
