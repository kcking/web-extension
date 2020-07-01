use crate::{Event, Port};
use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;

// TODO
#[wasm_bindgen]
extern "C" {
    pub type Runtime;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(
        this: &Runtime,
        extension_id: Option<&str>,
        message: &JsValue,
        options: Option<&Object>,
    ) -> Promise;

    #[wasm_bindgen(method)]
    pub fn connect(this: &Runtime, extension_id: Option<&str>, connect_info: &Object) -> Port;

    #[wasm_bindgen(method, js_name = connectNative)]
    pub fn connect_native(this: &Runtime, application_id: &str) -> Port;

    #[wasm_bindgen(method, getter, js_name = onMessage)]
    pub fn on_message(this: &Runtime) -> Event;

    #[wasm_bindgen(method, getter, js_name = onConnect)]
    pub fn on_connect(this: &Runtime) -> Event;

    #[wasm_bindgen(method, getter, js_name = onDisconnect)]
    pub fn on_disconnect(this: &Runtime) -> Event;

    #[wasm_bindgen(method, getter, js_name = lastError)]
    pub fn last_error(this: &Runtime) -> Option<js_sys::Error>;
}
