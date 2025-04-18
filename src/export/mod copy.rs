#[cfg(target_arch = "wasm32")]
use self::web::save;

#[cfg(target_arch = "wasm32")]
mod web {
    use anyhow::{Result, bail};
    use tracing::instrument;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{HtmlAnchorElement, window};

    #[instrument(err(Debug))]
    pub(super) fn save(buffer: Vec<u8>, name: &str) -> Result<()> {
        let Some(window) = window() else {
            bail!("window is none");
        };
        let Some(document) = window.document() else {
            bail!("document is none");
        };
        // let link = match document.create_element("a") {
        //     Ok(link) => link,
        //     Err(error) => bail!("create link element: {error:?}"),
        // };
        // if let Err(error) = link.set_attribute("href", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR42mP4z8AAAAMBAQD3A0FDAAAAAElFTkSuQmCC") {
        //     bail!("set link attribute: {error:?}");
        // }
        // link.set_attribute("download", name);
        // let link = HtmlAnchorElement::unchecked_from_js(link.into());
        // link.click();
        Ok(())
    }
    // #[instrument(err(Debug))]
    // pub(super) fn save(buffer: Vec<u8>, name: &str) -> Result<()> {
    //     if let Some(document) = window().and_then(|window| window.document()) {
    //         let link = document.create_element("a").unwrap();
    //         link.set_attribute("href", "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR42mP4z8AAAAMBAQD3A0FDAAAAAElFTkSuQmCC");
    //         link.set_attribute("download", "output.png");
    //         let link = HtmlAnchorElement::unchecked_from_js(link.into());
    //         link.click();
    //     }
    //     Ok(())
    // }
}

// use js_sys::{Array, ArrayBuffer, Uint8Array};
// use std::{
//     rc::Rc,
//     sync::mpsc::{channel, Receiver, Sender},
// };
// use wasm_bindgen::{prelude::*, JsCast, JsError, JsValue};
// use web_sys::{
//     window, Blob, BlobPropertyBag, File, FilePropertyBag, FileReader, HtmlAnchorElement,
//     HtmlInputElement, Url,
// };

// pub fn save(&self, file_name: &str, bytes: &[u8]) -> Result<(), JsValue> {
//     if let Some(window) = window() {
//         // let array = Uint8Array::from(&*content);
//         // let file_bits = Array::new();
//         // file_bits.push(&array.buffer());
//         let array = Array::from(&Uint8Array::from(bytes));
//         let blob = Blob::new_with_u8_array_sequence_and_options(
//             &array,
//             BlobPropertyBag::new().type_("application/octet-stream"),
//         )?;
//         // let file = File::new_with_blob_sequence_and_options(
//         //     &file_bits.into(),
//         //     file_name,
//         //     FilePropertyBag::new().type_("application/octet-stream"),
//         // )?;
//         let url = Url::create_object_url_with_blob(&blob)?;
//         window.location().set_href(&url)?;
//     }
//     Ok(())
// }
pub mod xlsx;
