use js_sys::{Array, Object, Promise, Uint8Array};
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{AttributeDataType, DracoDecodeConfig};

thread_local! {
    static DRACO_DECODE_FUNC_MODULE: RefCell<Option<JsValue>> = RefCell::new(None);
    static DRACO_CORE_MODULE: RefCell<Option<JsValue>> = RefCell::new(None);
}

/// Imports an embedded JS bundle by eval'ing it into a blob module.
///
/// Works in any wasm context (main thread or a dedicated worker), which is
/// what lets the pure-decode path run inside a host's own worker.
async fn import_bundle(code: &'static str) -> Result<JsValue, JsValue> {
    let escaped = code.replace("\\", "\\\\").replace("`", "\\`");

    let setup_code = format!(
        r#"
        (function() {{
            const code = `{escaped}`;
            const blob = new Blob([code], {{ type: "application/javascript" }});
            const url = URL.createObjectURL(blob);
            return import(url).then(mod => {{
                URL.revokeObjectURL(url);
                return mod;
            }});
        }})()
    "#
    );

    let js_module = js_sys::eval(&setup_code)?;
    let module_promise: Promise = js_module.dyn_into()?;
    JsFuture::from(module_promise).await
}

/// The worker-hosted bundle (`javascript/index.es.js`): decoding runs in the
/// bundle's own dedicated worker.
async fn get_js_module() -> Result<JsValue, JsValue> {
    if let Some(module) = DRACO_DECODE_FUNC_MODULE.with(|m| m.borrow().clone()) {
        return Ok(module);
    }

    let module = import_bundle(include_str!("../javascript/index.es.js")).await?;
    DRACO_DECODE_FUNC_MODULE.with(|m| m.replace(Some(module.clone())));

    Ok(module)
}

/// The pure-decode bundle (`javascript/core.es.js`): decoding runs in the
/// CURRENT context — no worker is spawned.
async fn get_core_js_module() -> Result<JsValue, JsValue> {
    if let Some(module) = DRACO_CORE_MODULE.with(|m| m.borrow().clone()) {
        return Ok(module);
    }

    let module = import_bundle(include_str!("../javascript/core.es.js")).await?;
    DRACO_CORE_MODULE.with(|m| m.replace(Some(module.clone())));

    Ok(module)
}

/// Converts the JS config object `{ vertex_count, index_count, buffer_size,
/// attributes: [{dim, data_type, offset, length}] }` into a
/// [`DracoDecodeConfig`].
fn parse_decode_config(config_obj: &JsValue) -> Result<DracoDecodeConfig, JsValue> {
    let vertex_count = js_sys::Reflect::get(config_obj, &JsValue::from_str("vertex_count"))?
        .as_f64()
        .unwrap_or(0.0) as u32;
    let index_count = js_sys::Reflect::get(config_obj, &JsValue::from_str("index_count"))?
        .as_f64()
        .unwrap_or(0.0) as u32;
    let buffer_size = js_sys::Reflect::get(config_obj, &JsValue::from_str("buffer_size"))?
        .as_f64()
        .unwrap_or(0.0) as usize;

    let attributes_array =
        js_sys::Reflect::get(config_obj, &JsValue::from_str("attributes"))?.dyn_into::<Array>()?;

    let mut config = DracoDecodeConfig::new(vertex_count, index_count, buffer_size);

    for i in 0..attributes_array.length() {
        let attr_obj = attributes_array.get(i).dyn_into::<Object>()?;

        let dim = js_sys::Reflect::get(&attr_obj, &JsValue::from_str("dim"))?
            .as_f64()
            .unwrap_or(0.0) as u32;
        let data_type = js_sys::Reflect::get(&attr_obj, &JsValue::from_str("data_type"))?
            .as_f64()
            .unwrap_or(0.0) as i32;
        let offset = js_sys::Reflect::get(&attr_obj, &JsValue::from_str("offset"))?
            .as_f64()
            .unwrap_or(0.0) as u32;
        let length = js_sys::Reflect::get(&attr_obj, &JsValue::from_str("length"))?
            .as_f64()
            .unwrap_or(0.0) as u32;

        let attr_data_type = AttributeDataType::from_draco_data_type(data_type);

        config.add_attribute(dim, attr_data_type, offset, length);
    }

    Ok(config)
}

/// Calls `decodeDracoMeshInWorkerWithConfig` on the worker-hosted bundle.
async fn decode_draco_mesh_from_embedded_js_with_config(
    data: &js_sys::Uint8Array,
) -> Result<(Vec<u8>, DracoDecodeConfig), JsValue> {
    let module = get_js_module().await?;

    let decode_fn = js_sys::Reflect::get(
        &module,
        &JsValue::from_str("decodeDracoMeshInWorkerWithConfig"),
    )?
    .dyn_into::<js_sys::Function>()?;

    let this = JsValue::NULL;
    let result = decode_fn.call1(&this, data)?;
    let decode_promise: Promise = result.dyn_into()?;
    let out_obj = JsFuture::from(decode_promise).await?;

    // Parse the result: { decoded: Uint8Array, config: Object }
    let decoded_array =
        js_sys::Reflect::get(&out_obj, &JsValue::from_str("decoded"))?.dyn_into::<Uint8Array>()?;
    let config_obj = js_sys::Reflect::get(&out_obj, &JsValue::from_str("config"))?;

    Ok((decoded_array.to_vec(), parse_decode_config(&config_obj)?))
}

/// Calls `parseDracoMeshWithConfig` on the pure-decode bundle, running the
/// decode in the current context (no worker round-trip).
async fn decode_draco_mesh_local_with_config(
    data: &js_sys::Uint8Array,
) -> Result<(Vec<u8>, DracoDecodeConfig), JsValue> {
    let module = get_core_js_module().await?;

    let decode_fn = js_sys::Reflect::get(&module, &JsValue::from_str("parseDracoMeshWithConfig"))?
        .dyn_into::<js_sys::Function>()?;

    let this = JsValue::NULL;
    let result = decode_fn.call1(&this, data)?;
    let decode_promise: Promise = result.dyn_into()?;
    let out_obj = JsFuture::from(decode_promise).await?;

    // Parse the result: { decoded: Uint8Array, config: Object }
    let decoded_array =
        js_sys::Reflect::get(&out_obj, &JsValue::from_str("decoded"))?.dyn_into::<Uint8Array>()?;
    let config_obj = js_sys::Reflect::get(&out_obj, &JsValue::from_str("config"))?;

    Ok((decoded_array.to_vec(), parse_decode_config(&config_obj)?))
}

pub async fn decode_mesh_wasm_worker_with_config(data: &[u8]) -> Option<crate::MeshDecodeResult> {
    let js_array = Uint8Array::from(data);

    match decode_draco_mesh_from_embedded_js_with_config(&js_array).await {
        Ok((decoded, config)) => Some(crate::MeshDecodeResult {
            data: decoded,
            config,
        }),
        Err(err) => {
            web_sys::console::error_1(&err);
            None
        }
    }
}

pub async fn decode_mesh_local_with_config(data: &[u8]) -> Option<crate::MeshDecodeResult> {
    let js_array = Uint8Array::from(data);

    match decode_draco_mesh_local_with_config(&js_array).await {
        Ok((decoded, config)) => Some(crate::MeshDecodeResult {
            data: decoded,
            config,
        }),
        Err(err) => {
            web_sys::console::error_1(&err);
            None
        }
    }
}
