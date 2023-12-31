use std::{cell::RefCell, collections::HashMap, f32::consts::TAU, f64::consts::PI, rc::Rc};

use js_sys::Math;
use nalgebra::{vector, ComplexField};
use rapier2d::prelude::point;
use shared::{Lobby, LobbySettings, LobbySort, Message, Team, Turn};
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{MainMenuState, State};
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, Particle, ParticleSort, ParticleSystem, StateSort, ToggleButtonElement,
        UIElement,
    },
    draw::{
        draw_bug, draw_bug_impulse, draw_image_centered, draw_label, draw_prop, draw_sand_circle,
        draw_text, local_to_screen, screen_to_local,
    },
    net::{create_new_lobby, fetch, request_turns_since, send_message, send_ready, MessagePool},
    tuple_as,
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
    animated_capture_progress: f32,
    capture_frame: usize,
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

        let _button_menu = ToggleButtonElement::new(
            (-128 - 18 - 8, -9 - 12),
            (20, 20),
            BUTTON_MENU,
            LabelTrim::Round,
            LabelTheme::Bright,
            crate::app::ContentElement::Sprite((112, 32), (16, 16)),
        );

        let _button_undo = ButtonElement::new(
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
            animated_capture_progress: 0.0,
            capture_frame: 0,
        }
    }

    pub fn particle_system(&mut self) -> &mut ParticleSystem {
        &mut self.particle_system
    }

    pub fn team_for(&self, session_id: &Option<String>) -> Option<Team> {
        if let Some(session_id) = session_id {
            self.lobby
                .players()
                .get(session_id)
                .map(|player| player.team)
        } else {
            None
        }
    }

    pub(crate) fn print_turns(&self) {
        let indexes: Vec<_> = self.lobby.turns().iter().map(|v| v.index).collect();
        console::log_1(&format!("{indexes:#?}").into());
    }
}

impl State for GameState {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        _interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        let my_team = self.team_for(&app_context.session_id);

        let point = tuple_as!(screen_to_local(tuple_as!(pointer.location, f64)), f32);
        let point = point![point.0, point.1];

        draw_image_centered(
            context,
            atlas,
            360.0,
            0.0,
            360.0,
            360.0,
            384.0 / 2.0,
            360.0 / 2.0,
        )?;

        self.animated_capture_progress +=
            (self.lobby.game.capture_progress() - self.animated_capture_progress) * 0.05;

        draw_sand_circle(
            &app_context.atlas_context,
            self.animated_capture_progress,
            self.lobby.game.capture_radius() * 16.0,
        )?;

        draw_image_centered(
            context,
            atlas,
            360.0,
            360.0,
            360.0,
            360.0,
            384.0 / 2.0,
            360.0 / 2.0,
        )?;

        {
            let bar_width = 7 * 24;
            let length = bar_width as f64
                - (self.lobby.game.turn_percentage_time() * bar_width as f64)
                    .floor()
                    .clamp(0.0, bar_width as f64);
            let label_length = (length as i32 / 2) * 2;

            draw_label(
                context,
                atlas,
                ((384 - bar_width) / 2, 8),
                (bar_width, 8),
                "#002a2a",
                &crate::app::ContentElement::None,
                pointer,
                frame,
                &LabelTrim::Round,
                false,
            )?;

            draw_label(
                context,
                atlas,
                ((384 - label_length) / 2, 8),
                (label_length, 8),
                "#CA891B",
                &crate::app::ContentElement::None,
                pointer,
                frame,
                &LabelTrim::Round,
                false,
            )?;

            let simulation_portion_length =
                (1.0 - self.lobby.game.turn_percentage_time_half()) * bar_width as f64;
            let simulation_portion_label_length = (simulation_portion_length as i32 / 2) * 2;

            draw_label(
                context,
                atlas,
                (
                    (384 - (simulation_portion_label_length).min(label_length)) / 2,
                    8,
                ),
                ((simulation_portion_label_length).min(label_length), 8),
                "#fff",
                &crate::app::ContentElement::None,
                pointer,
                frame,
                &LabelTrim::Round,
                false,
            )?;
        }

        {
            let capture_progress = self.animated_capture_progress;
            let length = (capture_progress * 7.0 * 12.0)
                .abs()
                .floor()
                .clamp(0.0, 7.0 * 12.0);
            let length = (length as i32 / 2) * 2;

            draw_label(
                context,
                atlas,
                ((384 - 7 * 24) / 2, 360 - 16),
                (7 * 24, 8),
                "#002a2a",
                &crate::app::ContentElement::None,
                pointer,
                frame,
                &LabelTrim::Round,
                false,
            )?;

            draw_label(
                context,
                atlas,
                ((384 / 2) + length.min(0), 360 - 16),
                (length, 8),
                if capture_progress > 0.0 {
                    "#C20005"
                } else {
                    "#00C2BD"
                },
                &crate::app::ContentElement::None,
                pointer,
                frame,
                &LabelTrim::Round,
                false,
            )?;
        }

        {
            context.save();
            context.translate(384.0 / 2.0, 360.0 / 2.0)?;
            self.particle_system()
                .tick_and_draw(context, atlas, frame)?;
            context.restore();
        }

        if let Some((_, rigid_body, _bug_data)) = self.lobby.game.intersecting_bug(point) {
            let (dx, dy) = local_to_screen(rigid_body.translation());

            draw_image_centered(context, atlas, 0.0, 176.0, 32.0, 32.0, dx, dy)?;
        }

        for (index, prop) in self.lobby.game.iter_props().enumerate() {
            draw_prop(context, atlas, prop, index, frame)?;
        }

        for (index, bug) in self.lobby.game.iter_bugs().enumerate() {
            draw_bug(context, atlas, bug, index, frame)?;

            if my_team == Some(*bug.1.team()) {
                draw_bug_impulse(context, atlas, bug, index, frame)?;
            }
        }

        for (_index, (rigid_body, bug_data)) in self.lobby.game.iter_bugs().enumerate() {
            let (dx, dy) = local_to_screen(rigid_body.translation());

            if my_team == Some(*bug_data.team()) {
                match bug_data.team() {
                    shared::Team::Red => {
                        draw_image_centered(context, atlas, 32.0, 176.0, 8.0, 8.0, dx, dy - 12.0)?;
                    }
                    shared::Team::Blue => {
                        draw_image_centered(context, atlas, 40.0, 176.0, 8.0, 8.0, dx, dy - 12.0)?;
                    }
                }
            }
        }

        if let Some(selected_bug_index) = self.selected_bug_index {
            if let Some((rigid_body, _bug_data)) = self.lobby.game.get_bug(selected_bug_index) {
                let (dx, dy) = local_to_screen(rigid_body.translation());

                draw_image_centered(context, atlas, 0.0, 176.0, 32.0, 32.0, dx, dy)?;
            }
        }

        match (self.lobby.game.turn_tick_count() as i64 - self.lobby.game.turn_ticks() as i64) / 60
        {
            2 => draw_image_centered(
                context,
                atlas,
                96.0,
                256.0,
                48.0,
                48.0,
                384.0 / 2.0,
                360.0 / 2.0,
            )?,
            1 => draw_image_centered(
                context,
                atlas,
                48.0,
                256.0,
                48.0,
                48.0,
                384.0 / 2.0,
                360.0 / 2.0,
            )?,
            0 => draw_image_centered(
                context,
                atlas,
                0.0,
                256.0,
                48.0,
                48.0,
                384.0 / 2.0,
                360.0 / 2.0,
            )?,
            _ => (),
        }

        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     8.0,
        //     format!("{:?}", self.lobby.settings).as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     24.0,
        //     format!(
        //         "{:?}",
        //         self.lobby
        //             .game
        //             .iter_bugs()
        //             .map(|(a, b)| b)
        //             .collect::<Vec<&BugData>>()
        //     )
        //     .as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     16.0,
        //     format!("{:?}", self.lobby.game.turn_ticks()).as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     24.0,
        //     format!("{:?}", self.lobby.game.target_tick()).as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     72.0,
        //     16.0,
        //     format!(
        //         "{:?}",
        //         self.lobby.game.target_tick().saturating_sub(self.lobby.game.ticks())
        //     )
        //     .as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     32.0,
        //     format!("{:?}", self.lobby.game.turns_count()).as_str(),
        // )?;
        // draw_text(
        //     context,
        //     atlas,
        //     8.0,
        //     48.0,
        //     format!("{:?}", self.lobby.game.all_turns_count()).as_str(),
        // )?;

        // if let Some(turn) = self.lobby.game.last_turn() {
        //     for (i, (bug, intent)) in turn
        //         .impulse_intents
        //         .iter()
        //         .enumerate()
        //         .sorted_by(|a, b| a.0.cmp(&b.0))
        //     {
        //         draw_text(
        //             context,
        //             atlas,
        //             8.0,
        //             64.0 + i as f64 * 12.0,
        //             format!("{:?} {:?}", bug, intent).as_str(),
        //         )?;
        //     }
        // }

        // console::log_1(&format!("{:?}", self.lobby.game.get_bug(0)).into());

        if self.lobby.game.turn_ticks() == self.lobby.game.turn_tick_count_half() {
            self.particle_system().spawn(100, |_| {
                let round = std::f64::consts::TAU * Math::random();
                let x = round.cos() * 4.0 * 16.0;
                let y = round.sin() * 4.0 * 16.0;

                Particle::new(
                    (x, y),
                    (
                        (Math::random()) * round.cos() * 7.0,
                        (Math::random()) * round.sin() * 7.0,
                    ),
                    20 + (Math::random() * 40.0) as usize,
                    crate::app::ParticleSort::Missile,
                )
            });
        }

        let capture_progress_unsigned_distance =
            (self.animated_capture_progress - self.lobby.game.capture_progress()).abs() as f64;

        if capture_progress_unsigned_distance > 0.05 || self.animated_capture_progress.abs() > 1.0 {
            let particle_sort =
                if self.animated_capture_progress < self.lobby.game.capture_progress() {
                    ParticleSort::RedWin
                } else {
                    ParticleSort::BlueWin
                };

            self.particle_system().spawn(
                2 + (capture_progress_unsigned_distance * 6.0).round() as usize,
                |_| {
                    let round = std::f64::consts::TAU * Math::random();
                    let x = round.cos() * 4.0 * 16.0;
                    let y = round.sin() * 4.0 * 16.0;

                    Particle::new(
                        (x, y),
                        (
                            (Math::random())
                                * round.cos()
                                * 6.0
                                * (1.0 + capture_progress_unsigned_distance * 4.0),
                            (Math::random())
                                * round.sin()
                                * 6.0
                                * (1.0 + capture_progress_unsigned_distance * 4.0),
                        ),
                        20 + (Math::random() * 40.0) as usize,
                        particle_sort,
                    )
                },
            );
        }

        for ((a, b), data) in self.lobby.game.bug_impacts() {
            self.particle_system().spawn(10, |_| {
                let round = std::f64::consts::TAU * Math::random();
                let x = data.x as f64 * 16.0;
                let y = data.y as f64 * 16.0;

                Particle::new(
                    (x, y),
                    (
                        (Math::random()) * round.cos() * 5.0,
                        (Math::random()) * round.sin() * 5.0,
                    ),
                    20 + (Math::random() * 10.0) as usize,
                    crate::app::ParticleSort::Missile,
                )
            });
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

        let point = tuple_as!(screen_to_local(tuple_as!(pointer.location, f64)), f32);
        let point = point![point.0, point.1];

        let my_team = self.team_for(&app_context.session_id);

        let mut message_pool = self.message_pool.borrow_mut();

        for message in &message_pool.messages {
            match message {
                Message::Ok => (),
                Message::Lobby(lobby) => {
                    self.lobby = *lobby.clone();
                }
                Message::Lobbies(_lobbies) => (),
                Message::LobbyError(_) => (),
                Message::Move(_) => (),
                Message::TurnSync(turns) => {
                    self.lobby.game.queue_turns(turns.clone());
                }
            }
        }

        message_pool.clear();

        if message_pool.available(frame) {
            if let LobbySort::Online(lobby_id) = self.lobby.settings.sort() {
                let _ = fetch(&request_turns_since(
                    *lobby_id,
                    self.lobby.game.all_turns_count(),
                ))
                .then(&self.message_closure);
            }

            message_pool.block(frame);
        }

        if self.animated_capture_progress.abs() > 1.0 {
            if self.capture_frame == 0 {
                self.capture_frame = frame;
            } else if frame - self.capture_frame > 180 {
                return Some(StateSort::MainMenu(MainMenuState::default()));
            }
        }

        if let Some(bug_index) = self.selected_bug_index {
            if let Some((rigid_body, bug_data)) = self.lobby.game.get_bug_mut(bug_index) {
                if Some(*bug_data.team()) == my_team {
                    let impulse_intent = vector![point.x, point.y] - rigid_body.translation();
                    bug_data.set_impulse_intent(impulse_intent);
                }
            }
        }

        if pointer.clicked() {
            if let Some(bug_index) = self.selected_bug_index {
                if let Some((_rigid_body, bug_data)) = self.lobby.game.get_bug_mut(bug_index) {
                    if let LobbySort::Online(lobby_id) = self.lobby.settings.sort() {
                        send_message(
                            *lobby_id,
                            app_context.session_id.clone().unwrap(),
                            Message::Move(Turn {
                                impulse_intents: HashMap::from([(
                                    bug_index,
                                    *bug_data.impulse_intent(),
                                )]),
                                timestamp: 0.0,
                                index: self.lobby.game.turns_count(),
                            }),
                        );
                    }
                }
            }

            if let Some((rigid_body_handle, _rigid_body, bug_data)) =
                self.lobby.game.intersecting_bug_mut(point)
            {
                if Some(*bug_data.team()) == my_team && bug_data.health() > 1 {
                    self.selected_bug_index = Some(rigid_body_handle);
                } else {
                    self.selected_bug_index = None
                }
            } else {
                self.selected_bug_index = None;
            }
        }

        // if pointer.alt_clicked() {
        //     self.lobby.game.execute_turn();
        // }

        // self.target_tick =
        //     ((self.lobby.game.all_turns_count() as f64 - 1.0) * 7.0 * 60.0).max(0.0) as u64;

        // self.server_target_tick = self.server_target_tick.max(self.lobby.target_tick());

        self.lobby.game.tick();

        // console::log_1(
        //     &format!(
        //         "{:?} {:?}",
        //         self.lobby.game.target_tick(),
        //         self.lobby.game.ticks()
        //     )
        //     .into(),
        // );

        None
    }
}
