use std::{cell::RefCell, rc::Rc};

use js_sys::Math;
use rapier2d::prelude::{point, vector};
use shared::{Bug, Game, Lobby, LobbySettings, Message, Physics};
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::State;
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, ParticleSystem, StateSort, ToggleButtonElement, UIElement,
    },
    draw::{draw_bug, draw_image, draw_image_centered, local_to_screen},
    net::{create_new_lobby, MessagePool},
    window,
};

const BUTTON_REMATCH: usize = 1;
const BUTTON_LEAVE: usize = 2;
const BUTTON_MENU: usize = 10;
const BUTTON_UNDO: usize = 20;

pub struct GameState {
    interface: Interface,
    lobby: Lobby,
    particle_system: ParticleSystem,
    message_pool: Rc<RefCell<MessagePool>>,
    message_closure: Closure<dyn FnMut(JsValue)>,
    shake_frame: (u64, usize),
}

impl GameState {
    pub fn new(lobby_settings: LobbySettings) -> GameState {
        let message_pool = Rc::new(RefCell::new(MessagePool::new()));

        let message_closure = {
            let message_pool = message_pool.clone();

            Closure::<dyn FnMut(JsValue)>::new(move |value| {
                let mut message_pool = message_pool.borrow_mut();
                let message: Message = serde_wasm_bindgen::from_value(value).unwrap();
                message_pool.push(message);
            })
        };

        if let shared::LobbySort::Online(0) = lobby_settings.sort() {
            let _ = create_new_lobby(lobby_settings.clone())
                .unwrap()
                .then(&message_closure);
        }

        let button_menu = ToggleButtonElement::new(
            (-128 - 18 - 8, -9 - 12),
            (20, 20),
            BUTTON_MENU,
            LabelTrim::Round,
            LabelTheme::Bright,
            crate::app::ContentElement::Sprite((112, 32), (16, 16)),
        );

        let button_undo = ButtonElement::new(
            (-128 - 18 - 8, -9 + 12),
            (20, 20),
            BUTTON_UNDO,
            LabelTrim::Round,
            LabelTheme::Action,
            crate::app::ContentElement::Sprite((144, 16), (16, 16)),
        );

        let button_rematch = ButtonElement::new(
            (-44, -24),
            (88, 24),
            BUTTON_REMATCH,
            LabelTrim::Glorious,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Rematch".to_string(), Alignment::Center),
        );

        let button_leave = ConfirmButtonElement::new(
            (-36, 8),
            (72, 16),
            BUTTON_LEAVE,
            LabelTrim::Return,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Leave".to_string(), Alignment::Center),
        );

        let root_element = Interface::new(vec![button_rematch.boxed(), button_leave.boxed()]);

        GameState {
            interface: root_element,
            lobby: Lobby::new(lobby_settings),
            particle_system: ParticleSystem::default(),
            message_pool,
            message_closure,
            shake_frame: (0, 0),
            // physics: Physics::from_settings(),
        }
    }

    pub fn particle_system(&mut self) -> &mut ParticleSystem {
        &mut self.particle_system
    }
}

impl State for GameState {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        let point = point![
            (pointer.location.0 - 128) as f32 / 16.0,
            (pointer.location.1 - 128) as f32 / 16.0
        ];

        if let Some(bug) = self.lobby.game.intersecting_bug(point) {
            let (dx, dy) = local_to_screen(bug.rigid_body.translation());

            draw_image_centered(context, atlas, 0.0, 176.0, 32.0, 32.0, dx, dy)?;
        }

        for (index, bug) in self.lobby.game.iter_bugs().enumerate() {
            draw_bug(context, atlas, &bug, index, frame)?;
        }

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        let message_pool = self.message_pool.clone();

        self.lobby.tick();

        None
    }
}
