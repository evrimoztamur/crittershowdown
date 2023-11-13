use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use crate::app::{AppContext, StateSort};

pub trait State {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue>;

    fn tick(
        &mut self,
        text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort>;
}
