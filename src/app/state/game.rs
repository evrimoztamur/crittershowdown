use std::{cell::RefCell, rc::Rc};

use js_sys::Math;
use rapier2d::prelude::{point, vector};
use shared::{Lobby, LobbySettings, Message, Physics};
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::State;
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, ParticleSystem, StateSort, ToggleButtonElement, UIElement,
    },
    draw::{draw_image, draw_image_centered},
    net::{create_new_lobby, MessagePool},
    window,
};

const BUTTON_REMATCH: usize = 1;
const BUTTON_LEAVE: usize = 2;
const BUTTON_MENU: usize = 10;
const BUTTON_UNDO: usize = 20;

pub struct Game {
    interface: Interface,
    lobby: Lobby,
    particle_system: ParticleSystem,
    message_pool: Rc<RefCell<MessagePool>>,
    message_closure: Closure<dyn FnMut(JsValue)>,
    shake_frame: (u64, usize),
    physics: Physics,
}

impl Game {
    pub fn new(lobby_settings: LobbySettings) -> Game {
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

        Game {
            interface: root_element,
            lobby: Lobby::new(lobby_settings),
            particle_system: ParticleSystem::default(),
            message_pool,
            message_closure,
            shake_frame: (0, 0),
            physics: Physics::from_settings(),
        }
    }

    pub fn particle_system(&mut self) -> &mut ParticleSystem {
        &mut self.particle_system
    }

    pub fn lobby(&self) -> &Lobby {
        &self.lobby
    }

    // pub fn lobby_id(&self) -> Result<LobbyID, LobbyError> {
    //     self.lobby
    //         .settings
    //         .sort
    //         .lobby_id()
    //         .ok_or(LobbyError("lobby has no ID".to_string()))
    // }
}

impl State for Game {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        // if let Some(ball_position) = self.physics.ball_position() {
        //     draw_sprite(
        //         context,
        //         atlas,
        //         32.0,
        //         320.0,
        //         32.0,
        //         32.0,
        //         ball_position.x as f64 - 16.0,
        //         ball_position.y as f64 - 16.0,
        //     )?;

        //     console::log_1(&format!("Ball altitude: {}", ball_position.y).into());

        // }

        let point = point![
            (pointer.location.0 - 128) as f32 / 16.0,
            (pointer.location.1 - 128) as f32 / 16.0
        ];

        if let Some((collider_position, point_projection)) =
            self.physics.intersecting_collider(point)
        {
            draw_image_centered(
                context,
                atlas,
                0.0,
                176.0,
                32.0,
                32.0,
                collider_position.x as f64 * 16.0 + 128.0,
                collider_position.y as f64 * 16.0 + 128.0,
            )?;
        }

        for (i, ball_position) in self.physics.ball_positions().iter().enumerate() {
            let i = i as u64;
            draw_image_centered(
                context,
                atlas,
                16.0 * ((i % 2) as f64),
                16.0 * (((frame / (6 + (i % 3)) + (i % 3)) % 2) as f64),
                16.0,
                16.0,
                ball_position.x as f64,
                ball_position.y as f64,
            )?;
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

        let random = (Math::random() * self.physics.collider_set.len() as f64).floor() as usize;

        for (i, (_, rb)) in self.physics.rigid_body_set.iter_mut().enumerate() {
            if i == random {
                rb.apply_impulse(
                    (vector![Math::random() as f32 - 0.5, Math::random() as f32 - 0.5]).scale(2.0),
                    true,
                );
            }
        }

        self.physics.tick();

        None
    }
}
