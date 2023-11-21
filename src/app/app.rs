use serde::{Deserialize, Serialize};
use shared::{LobbyError, SessionRequest};
use wasm_bindgen::JsValue;
use web_sys::{
    CanvasRenderingContext2d, DomRectReadOnly, FocusEvent, HtmlCanvasElement,
    HtmlInputElement, KeyboardEvent, MouseEvent, TouchEvent,
};

use super::{AudioSystem, GameState, MainMenuState, Pointer};
use crate::{app::State, draw::draw_image, net::get_session_id, storage, window};

/// Errors concerning the [`App`].
#[derive(Debug, Serialize, Deserialize)]
pub struct AppError(String);

impl From<LobbyError> for AppError {
    fn from(lobby_error: LobbyError) -> Self {
        AppError(format!("LobbyError: {0}", lobby_error.0))
    }
}

pub enum StateSort {
    MainMenu(MainMenuState),
    Game(GameState),
}

pub struct AppContext {
    pub session_id: Option<String>,
    pub pointer: Pointer,
    pub frame: usize,
    pub canvas_settings: CanvasSettings,
    pub text_input: Option<(String, String)>,
    pub audio_system: AudioSystem,
    pub atlas_context: CanvasRenderingContext2d,
}

pub struct App {
    app_context: AppContext,
    state_sort: StateSort,
    atlas_complete: bool,
}

impl App {
    pub fn new(
        canvas_settings: &CanvasSettings,
        atlas_context: CanvasRenderingContext2d,
        audio_system: AudioSystem,
    ) -> App {
        App {
            app_context: AppContext {
                session_id: get_session_id(),
                pointer: Pointer::new(canvas_settings),
                frame: 0,
                canvas_settings: canvas_settings.clone(),
                text_input: None,
                audio_system,
                atlas_context,
            },
            // state_sort: StateSort::Game(GameState::new(LobbySettings::new(shared::LobbySort::Local))),
            state_sort: StateSort::MainMenu(MainMenuState::default()),
            atlas_complete: false,
        }
    }

    pub fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        context.clear_rect(
            0.0,
            0.0,
            self.app_context.canvas_settings.element_width() as f64,
            self.app_context.canvas_settings.element_height() as f64,
        );
        interface_context.clear_rect(
            0.0,
            0.0,
            self.app_context.canvas_settings.element_width() as f64,
            self.app_context.canvas_settings.element_height() as f64,
        );

        context.save();
        interface_context.save();

        if self.app_context.canvas_settings.orientation {
            context.translate(self.app_context.canvas_settings.element_width() as f64, 0.0)?;
            context.rotate(std::f64::consts::PI / 2.0)?;
            interface_context
                .translate(self.app_context.canvas_settings.element_width() as f64, 0.0)?;
            interface_context.rotate(std::f64::consts::PI / 2.0)?;
        }

        let canvas_scale = self.app_context.canvas_settings.canvas_scale;

        context.scale(canvas_scale, canvas_scale)?;
        interface_context.scale(canvas_scale, canvas_scale)?;

        context.translate(
            self.app_context.canvas_settings.padding_x() as f64,
            self.app_context.canvas_settings.padding_y() as f64,
        )?;

        interface_context.translate(
            self.app_context.canvas_settings.padding_x() as f64,
            self.app_context.canvas_settings.padding_y() as f64,
        )?;

        let mut result = Ok(());

        if !self.atlas_complete {
            self.atlas_complete = true;
        } else {
            result = match &mut self.state_sort {
                StateSort::Game(state) => {
                    state.draw(context, interface_context, atlas, &self.app_context)
                }
                StateSort::MainMenu(state) => {
                    state.draw(context, interface_context, atlas, &self.app_context)
                }
            };
        }

        // DRAW cursor
        draw_image(
            interface_context,
            atlas,
            0.0,
            208.0,
            16.0,
            16.0,
            self.app_context.pointer.location.0 as f64 - 5.0,
            self.app_context.pointer.location.1 as f64 - 2.0,
        )?;

        context.restore();
        interface_context.restore();

        self.app_context.frame = (window().performance().unwrap().now() * 0.06) as usize;
        self.app_context.pointer.swap();
        self.app_context.text_input = None;

        result
    }

    pub fn tick(&mut self, text_input: &HtmlInputElement) {
        let next_state = match &mut self.state_sort {
            StateSort::Game(state) => state.tick(text_input, &self.app_context),
            StateSort::MainMenu(state) => state.tick(text_input, &self.app_context),
        };

        if let Some(next_state) = next_state {
            self.state_sort = next_state;
        }
    }

    pub fn session_id(&self) -> Option<&String> {
        self.app_context.session_id.as_ref()
    }

    pub fn set_session_id(&mut self, session_id: String) {
        self.app_context.session_id = Some(session_id);
    }

    pub fn on_blur(&mut self, _event: FocusEvent, text_input: &HtmlInputElement) {
        if let Some(field) = text_input.dataset().get("field") {
            self.app_context.text_input = Some((field, text_input.value()));
            text_input.dataset().delete("field");
        }
    }

    pub fn on_mouse_down(&mut self, event: MouseEvent) {
        match event.button() {
            0 => self.app_context.pointer.button = true,
            2 => self.app_context.pointer.alt_button = true,
            _ => (),
        }
    }

    pub fn on_mouse_up(&mut self, event: MouseEvent) {
        match event.button() {
            0 => self.app_context.pointer.button = false,
            2 => self.app_context.pointer.alt_button = false,
            _ => (),
        }
    }

    pub fn on_mouse_move(&mut self, bound: &DomRectReadOnly, event: MouseEvent) {
        let x = event.page_x() - bound.left() as i32;
        let y = event.page_y() - bound.top() as i32;
        let pointer_location =
            App::transform_pointer(&self.app_context.canvas_settings, bound, x, y);

        self.app_context.pointer.location = pointer_location;

        event.prevent_default();
    }

    pub fn on_touch_start(&mut self, bound: &DomRectReadOnly, event: TouchEvent) {
        if let Some(touch) = event.target_touches().item(0) {
            let x = touch.page_x() - bound.left() as i32;
            let y = touch.page_y() - bound.top() as i32;

            let pointer_location =
                App::transform_pointer(&self.app_context.canvas_settings, bound, x, y);

            self.app_context.pointer.location = pointer_location;
        }
    }

    pub fn on_touch_end(&mut self, bound: &DomRectReadOnly, event: TouchEvent) {
        if let Some(touch) = event.target_touches().item(0) {
            let x = touch.page_x() - bound.left() as i32;
            let y = touch.page_y() - bound.top() as i32;

            let pointer_location =
                App::transform_pointer(&self.app_context.canvas_settings, bound, x, y);
            self.app_context.pointer.location = pointer_location;
        }

        self.app_context.pointer.button = false;
    }

    pub fn on_touch_move(&mut self, bound: &DomRectReadOnly, event: TouchEvent) {
        if let Some(touch) = event.target_touches().item(0) {
            let x = touch.page_x() - bound.left() as i32;
            let y = touch.page_y() - bound.top() as i32;

            let pointer_location =
                App::transform_pointer(&self.app_context.canvas_settings, bound, x, y);
            self.app_context.pointer.location = pointer_location;
        }

        event.prevent_default();
    }

    fn transform_pointer(
        canvas_settings: &CanvasSettings,
        bound: &DomRectReadOnly,
        x: i32,
        y: i32,
    ) -> (i32, i32) {
        let x = (x as f64 * (canvas_settings.element_width() as f64 / bound.width()))
            / canvas_settings.canvas_scale;
        let y = (y as f64 * (canvas_settings.element_height() as f64 / bound.height()))
            / canvas_settings.canvas_scale;

        Pointer::location_from_real(canvas_settings, (x as i32, y as i32))
    }

    #[allow(clippy::single_match)]
    pub fn on_key_down(&mut self, event: KeyboardEvent) {
        #[cfg(not(feature = "deploy"))]
        match &mut self.state_sort {
            StateSort::Game(state) => {
                match event.code().as_str() {
                    "KeyM" => {
                        state.print_turns();
                    }
                    _ => (),
                };
            }
            _ => (),
        }
    }

    pub fn on_session_response(&mut self, value: JsValue) {
        let session_request: SessionRequest = serde_wasm_bindgen::from_value(value).unwrap();
        let session_id = session_request.session_id;

        self.set_session_id(session_id.clone());

        storage().map(|storage| storage.set_item("session_id", session_id.as_str()));
    }

    pub fn kv_set(key: &str, value: &str) {
        storage().and_then(|storage| storage.set_item(key, value).ok());
    }

    pub fn kv_get(key: &str) -> String {
        storage()
            .and_then(|storage| storage.get_item(key).unwrap_or_default())
            .unwrap_or_default()
    }
}

#[derive(Clone, Default)]
pub struct CanvasSettings {
    pub interface_width: u32,
    pub interface_height: u32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub canvas_scale: f64,
    pub orientation: bool,
}

impl CanvasSettings {
    pub fn inverse_interface_center(&self) -> (i32, i32) {
        (
            -((self.interface_width / 2) as i32),
            -((self.interface_height / 2) as i32),
        )
    }

    pub fn element_width(&self) -> u32 {
        if self.orientation {
            (self.canvas_height as f64 * self.canvas_scale) as u32
        } else {
            (self.canvas_width as f64 * self.canvas_scale) as u32
        }
    }

    pub fn element_height(&self) -> u32 {
        if self.orientation {
            (self.canvas_width as f64 * self.canvas_scale) as u32
        } else {
            (self.canvas_height as f64 * self.canvas_scale) as u32
        }
    }

    pub fn padding_x(&self) -> u32 {
        (self.canvas_width - self.interface_width) / 2
    }

    pub fn padding_y(&self) -> u32 {
        (self.canvas_height - self.interface_height) / 2
    }

    pub fn padding(&self) -> (i32, i32) {
        (self.padding_x() as i32, self.padding_y() as i32)
    }

    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        interface_width: u32,
        interface_height: u32,
        canvas_scale: f64,
        orientation: bool,
    ) -> CanvasSettings {
        CanvasSettings {
            interface_width,
            interface_height,
            canvas_width,
            canvas_height,
            canvas_scale,
            orientation,
        }
    }
}
