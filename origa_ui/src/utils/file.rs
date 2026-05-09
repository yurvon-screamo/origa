pub async fn read_file_as_bytes(file: &web_sys::File) -> Result<Vec<u8>, String> {
    let array_buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| format!("Failed to read file: {e:?}"))?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}
