use macropad_model::MacroConfig;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn default_config() -> Result<JsValue, JsValue> {
    let config = MacroConfig::default();
    to_value(&config).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn import_config_from_cbor(bytes: &[u8]) -> Result<JsValue, JsValue> {
    let config: MacroConfig = serde_cbor::from_slice(bytes)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    to_value(&config).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn export_config_to_cbor(config: &JsValue) -> Result<Vec<u8>, JsValue> {
    let config: MacroConfig = from_value(config.clone()).map_err(|err| JsValue::from_str(&err.to_string()))?;
    serde_cbor::to_vec(&config).map_err(|err| JsValue::from_str(&err.to_string()))
}
