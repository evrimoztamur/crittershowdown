use std::{cell::RefCell, collections::HashMap, rc::Rc};

use js_sys::Math;
use nalgebra::vector;
use rapier2d::{dynamics::RigidBodyHandle, prelude::point};
use shared::{BugData, Lobby, LobbySettings, LobbySort, Message, Turn};
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::State;
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, ParticleSystem, StateSort, ToggleButtonElement, UIElement,
    },
    draw::{
        draw_bug, draw_bug_impulse, draw_image, draw_image_centered, draw_text, local_to_screen,
    },
    net::{create_new_lobby, send_message, send_ready, MessagePool, request_turns_since},
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
    selected_bug_index: Option<usize>,
}

impl GameState {
    pub fn new(lobby_settings: LobbySettings, session_id: String) -> GameState {
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
            let _ = create_new_lobby(lobby_settings.clone(), session_id)
                .unwrap()
                .then(&message_closure);
        } else if let shared::LobbySort::Online(lobby_id) = lobby_settings.sort() {
            let _ = send_ready(*lobby_id, session_id)
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
            lobby: Lobby::new(lobby_settings, 0.0),
            particle_system: ParticleSystem::default(),
            message_pool,
            message_closure,
            shake_frame: (0, 0),
            selected_bug_index: None,
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

        if let Some((_, rigid_body, bug_data)) = self.lobby.game.intersecting_bug(point) {
            let (dx, dy) = local_to_screen(rigid_body.translation());

            draw_image_centered(context, atlas, 0.0, 176.0, 32.0, 32.0, dx, dy)?;
        }

        for (index, bug) in self.lobby.game.iter_bugs().enumerate() {
            draw_bug(context, atlas, bug, index, frame)?;
            draw_bug_impulse(context, atlas, bug, index, frame)?;
        }

        if let Some(selected_bug_index) = self.selected_bug_index {
            if let Some((rigid_body, bug_data)) = self.lobby.game.get_bug(selected_bug_index) {
                let (dx, dy) = local_to_screen(rigid_body.translation());

                draw_image_centered(context, atlas, 0.0, 176.0, 32.0, 32.0, dx, dy)?;
            }
        }

        draw_text(
            context,
            atlas,
            8.0,
            8.0,
            format!("{:?}", self.lobby.settings).as_str(),
        )?;
        draw_text(
            context,
            atlas,
            8.0,
            24.0,
            format!(
                "{:?}",
                self.lobby
                    .game
                    .iter_bugs()
                    .map(|(a, b)| b)
                    .collect::<Vec<&BugData>>()
            )
            .as_str(),
        )?;

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        let point = point![
            (pointer.location.0 - 128) as f32 / 16.0,
            (pointer.location.1 - 128) as f32 / 16.0
        ];

        let mut message_pool = self.message_pool.borrow_mut();

        for message in &message_pool.messages {
            match message {
                Message::Ok => (),
                Message::Lobby(lobby) => {
                    self.lobby = *lobby.clone();
                }
                Message::Lobbies(lobbies) => (),
                Message::LobbyError(_) => (),
                Message::Move(_) => (),
                Message::Moves(_) => (),
            }
        }

        message_pool.clear();

        if message_pool.available(frame) {
            request_turns_since(lobby_id, 0);

            message_pool.block(frame);
        }

        if let Some(bug_index) = self.selected_bug_index {
            if let Some((rigid_body, bug_data)) = self.lobby.game.get_bug_mut(bug_index) {
                let impulse_intent = vector![point.x, point.y] - rigid_body.translation();
                bug_data.set_impulse_intent(impulse_intent);
            }
        }

        if let Some((rigid_body_handle, rigid_body, bug_data)) =
            self.lobby.game.intersecting_bug_mut(point)
        {
            if pointer.clicked() {
                self.selected_bug_index = Some(rigid_body_handle);
            }
        } else {
            if pointer.clicked() {
                if let Some(bug_index) = self.selected_bug_index {
                    if let Some((rigid_body, bug_data)) = self.lobby.game.get_bug_mut(bug_index) {
                        if let LobbySort::Online(lobby_id) = self.lobby.settings.sort() {
                            send_message(
                                *lobby_id,
                                app_context.session_id.clone().unwrap(),
                                Message::Move(Turn {
                                    impulse_intents: HashMap::from([(
                                        bug_index,
                                        *bug_data.impulse_intent(),
                                    )]),
                                }),
                            );
                        }
                    }
                }

                self.selected_bug_index = None;
            }
        }

        // if pointer.alt_clicked() {
        //     self.lobby.game.execute_turn();
        // }

        self.lobby.game.tick();

        None
    }
}
