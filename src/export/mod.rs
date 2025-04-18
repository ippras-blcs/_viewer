#[cfg(target_arch = "wasm32")]
use self::web::save;

// https://github.com/ippras-utca/utca/blob/bca91021413c4089f412d07267147db097c94eb6/src/widgets/file_dialog/mod.rs
// https://stackoverflow.com/questions/69556755/web-sysurlcreate-object-url-with-blobblob-not-formatting-binary-data-co
#[cfg(target_arch = "wasm32")]
mod web {
    use anyhow::{Result, bail};
    use base64::prelude::*;
    use js_sys::{Array, ArrayBuffer, Uint8Array};
    use tracing::instrument;
    use wasm_bindgen::prelude::*;
    use web_sys::{
        Blob, BlobPropertyBag, Document, Element, HtmlAnchorElement, Url, Window, window,
    };

    const XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
    const _DOCX: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    const _XLS: &str = "application/vnd.ms-excel";
    const _TYPE: &str = "application/octet-stream";

    #[instrument(err(Debug))]
    pub(super) fn save(content: &[u8], name: &str) -> Result<(), JsValue> {
        let Some(window) = window() else {
            return Err(JsError::new("window is none").into());
        };
        let Some(document) = window.document() else {
            return Err(JsError::new("document is none").into());
        };
        let bytes = Uint8Array::from(content);
        let array = Array::new();
        array.push(&bytes.buffer());
        let blob = Blob::new_with_u8_array_sequence_and_options(
            &array,
            BlobPropertyBag::new().type_(XLSX),
        )?;
        let url = Url::create_object_url_with_blob(&blob)?;
        // window.location().set_href(&url)?;
        let a = document.create_element("a")?;
        let link = HtmlAnchorElement::unchecked_from_js(a.into());
        link.set_download(name);
        link.set_href(&url);
        link.click();
        Ok(())
    }
}

pub mod xlsx;
