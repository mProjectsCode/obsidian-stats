mod utils;

use data_lib::plugin::{analysis::FullPluginDataArray, PluginData, PluginExtraData};
use wasm_bindgen::prelude::*;

use crate::utils::set_panic_hook;

#[wasm_bindgen]
pub fn load_data_from_chunks(
    data_chunks: Vec<String>,
    extended_data_chunks: Vec<String>,
) -> Result<FullPluginDataArray, JsValue> {
    set_panic_hook();

    let data = data_chunks
        .iter()
        .map(|chunk| serde_json::from_str::<Vec<PluginData>>(chunk))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| JsValue::from_str(&format!("Failed to parse data chunks: {}", e)))?;
    let data = data.into_iter().flatten().collect::<Vec<PluginData>>();

    let extended_data = extended_data_chunks
        .iter()
        .map(|chunk| serde_json::from_str::<Vec<PluginExtraData>>(chunk))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| JsValue::from_str(&format!("Failed to parse repo data chunks: {}", e)))?;
    let extended_data = extended_data
        .into_iter()
        .flatten()
        .collect::<Vec<PluginExtraData>>();

    Ok(FullPluginDataArray::new(data, extended_data))
}
