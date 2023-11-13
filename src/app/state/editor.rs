use std::mem;

use shared::{Board, Level, Mage, Mages, Position, PowerUp, Team};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{EditorPreview, MainMenu, State};
use crate::{
    app::{
        Alignment, App, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, Particle, ParticleSort, ParticleSystem, StateSort, ToggleButtonElement,
        UIElement, UIEvent, BOARD_SCALE,
    },
    draw::{
        draw_board, draw_crosshair, draw_mage, draw_mana, draw_powerup, draw_spell_pattern,
        draw_sprite,
    },
    tuple_as,
};

#[derive(PartialEq)]
pub enum EditorSelection {
    Mage(Mage),
    PowerUp(PowerUp),
    Tile(Position),
    None,
}

pub struct Editor {
    button_menu: ToggleButtonElement,
    interface: Interface,
    menu_interface: Interface,
    mage_interface: Interface,
    prop_interface: Interface,
    no_mage_interface: Interface,
    board_dirty: bool,
    level: Level,
    particle_system: ParticleSystem,
    selection: EditorSelection,
}

const BUTTON_MENU: usize = 0;
const BUTTON_MODE_TOGGLE: usize = 10;
const BUTTON_SAVE: usize = 11;

const BUTTON_WIDTH_MINUS: usize = 20;
const BUTTON_WIDTH_PLUS: usize = 21;
const BUTTON_HEIGHT_MINUS: usize = 22;
const BUTTON_HEIGHT_PLUS: usize = 23;

const BUTTON_TEAM_LEFT: usize = 30;
const BUTTON_TEAM_RIGHT: usize = 31;
const BUTTON_SPELL_LEFT: usize = 32;
const BUTTON_SPELL_RIGHT: usize = 33;
const BUTTON_MANA_LEFT: usize = 34;
const BUTTON_MANA_RIGHT: usize = 35;
const BUTTON_DELETE: usize = 39;

const BUTTON_ADD_MAGE: usize = 40;
const BUTTON_ADD_PROP: usize = 41;

const BUTTON_LOAD: usize = 12;
const BUTTON_SIMULATE: usize = 50;
const BUTTON_RESET: usize = 51;
const BUTTON_LEAVE: usize = 100;

impl Editor {
    pub fn new(level: Level) -> Editor {
        let button_menu = ToggleButtonElement::new(
            (-60, 118),
            (20, 20),
            BUTTON_MENU,
            LabelTrim::Round,
            LabelTheme::Bright,
            crate::app::ContentElement::Sprite((112, 32), (16, 16)),
        );

        let button_mode_toggle = ButtonElement::new(
            (236, 228),
            (80, 24),
            BUTTON_MODE_TOGGLE,
            LabelTrim::Glorious,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Battle".to_string(), Alignment::Center),
        );

        let button_save = ButtonElement::new(
            (244, 204),
            (64, 16),
            BUTTON_SAVE,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Save".to_string(), Alignment::Center),
        );

        let button_width_minus = ButtonElement::new(
            (82, 248),
            (12, 12),
            BUTTON_WIDTH_MINUS,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((88, 24), (8, 8)),
        );

        let button_width_plus = ButtonElement::new(
            (98, 248),
            (12, 12),
            BUTTON_WIDTH_PLUS,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((80, 24), (8, 8)),
        );

        let button_height_minus = ButtonElement::new(
            (216, 114),
            (12, 12),
            BUTTON_HEIGHT_MINUS,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((88, 24), (8, 8)),
        );

        let button_height_plus = ButtonElement::new(
            (216, 130),
            (12, 12),
            BUTTON_HEIGHT_PLUS,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((80, 24), (8, 8)),
        );

        let button_team_left = ButtonElement::new(
            (240, 122 - 92),
            (12, 20),
            BUTTON_TEAM_LEFT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((64, 24), (8, 8)),
        );

        let button_team_right = ButtonElement::new(
            (300, 122 - 92),
            (12, 20),
            BUTTON_TEAM_RIGHT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((72, 24), (8, 8)),
        );

        let button_spell_left = ButtonElement::new(
            (240, 122 - 38),
            (12, 32),
            BUTTON_SPELL_LEFT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((64, 24), (8, 8)),
        );

        let button_spell_right = ButtonElement::new(
            (300, 122 - 38),
            (12, 32),
            BUTTON_SPELL_RIGHT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((72, 24), (8, 8)),
        );

        let button_mana_left = ButtonElement::new(
            (244, 122 + 8),
            (12, 12),
            BUTTON_MANA_LEFT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((64, 24), (8, 8)),
        );

        let button_mana_right = ButtonElement::new(
            (296, 122 + 8),
            (12, 12),
            BUTTON_MANA_RIGHT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((72, 24), (8, 8)),
        );

        let button_delete = ButtonElement::new(
            (260, 160),
            (32, 20),
            BUTTON_DELETE,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((128, 32), (16, 16)),
        );

        let mage_interface = Interface::new(vec![
            button_team_left.clone().boxed(),
            button_team_right.clone().boxed(),
            button_delete.boxed(),
            button_spell_left.boxed(),
            button_spell_right.boxed(),
            button_mana_left.boxed(),
            button_mana_right.boxed(),
        ]);

        let button_delete = ButtonElement::new(
            (260, 80),
            (32, 20),
            BUTTON_DELETE,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((128, 32), (16, 16)),
        );

        let prop_interface = Interface::new(vec![
            button_team_left.boxed(),
            button_team_right.boxed(),
            button_delete.boxed(),
        ]);

        let button_add_mage = ButtonElement::new(
            (252, 118 - 14),
            (48, 20),
            BUTTON_ADD_MAGE,
            LabelTrim::Glorious,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((160, 16), (16, 16)),
        );

        let button_add_prop: ButtonElement = ButtonElement::new(
            (252, 118 + 14),
            (48, 20),
            BUTTON_ADD_PROP,
            LabelTrim::Glorious,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((176, 16), (16, 16)),
        );

        let no_mage_interface =
            Interface::new(vec![button_add_mage.boxed(), button_add_prop.boxed()]);

        let root_element = Interface::new(vec![
            button_mode_toggle.boxed(),
            button_save.boxed(),
            button_width_minus.boxed(),
            button_width_plus.boxed(),
            button_height_minus.boxed(),
            button_height_plus.boxed(),
        ]);

        let button_load = ButtonElement::new(
            (96 - 44, 128 - 32),
            (88, 24),
            BUTTON_LOAD,
            LabelTrim::Round,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Load".to_string(), Alignment::Center),
        );

        let button_simulate = ButtonElement::new(
            (96 - 44, 128),
            (88, 16),
            BUTTON_SIMULATE,
            LabelTrim::Round,
            LabelTheme::Disabled,
            crate::app::ContentElement::Text("Simulate".to_string(), Alignment::Center),
        );

        let button_reset = ConfirmButtonElement::new(
            (96 - 44, 128 + 20),
            (88, 16),
            BUTTON_RESET,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Reset".to_string(), Alignment::Center),
        );

        let button_leave = ConfirmButtonElement::new(
            (96 - 36, 128 + 48),
            (72, 16),
            BUTTON_LEAVE,
            LabelTrim::Return,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Leave".to_string(), Alignment::Center),
        );

        let menu_interface = Interface::new(vec![
            button_load.boxed(),
            button_simulate.boxed(),
            button_reset.boxed(),
            button_leave.boxed(),
        ]);

        Editor {
            button_menu,
            interface: root_element,
            mage_interface,
            prop_interface,
            menu_interface,
            no_mage_interface,
            level,
            particle_system: ParticleSystem::default(),
            selection: EditorSelection::None,
            board_dirty: true,
        }
    }

    pub fn board_offset(&self) -> (i32, i32) {
        (
            ((8 - self.level.board.width) as i32 * BOARD_SCALE.0) / 2,
            ((8 - self.level.board.height) as i32 * BOARD_SCALE.1) / 2,
        )
    }

    pub fn is_interface_active(&self) -> bool {
        self.button_menu.selected()
    }

    fn sparkle_create(&mut self, position: Position) {
        for _ in 0..40 {
            let d = js_sys::Math::random() * std::f64::consts::TAU;
            let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
            self.particle_system.add(Particle::new(
                (position.0 as f64, position.1 as f64),
                (d.cos() * v * 2.0, d.sin() * v),
                (js_sys::Math::random() * 20.0) as u64,
                ParticleSort::Missile,
            ));
        }
    }

    fn occupied(&self, position: &Position) -> bool {
        self.level.mages.occupied(position)
    }
}

impl State for Editor {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let board_scale = tuple_as!(BOARD_SCALE, f64);
        let board_offset = self.board_offset();

        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        if self.board_dirty {
            self.board_dirty = false;
            draw_board(atlas, 256.0, 0.0, &self.level.board, 8, 8).unwrap();
        }

        context.save();

        context.translate(-32.0, 0.0)?;

        draw_sprite(context, atlas, 256.0, 0.0, 256.0, 256.0, 0.0, 0.0)?;
        draw_sprite(context, atlas, 256.0, 256.0, 64.0, 64.0, 276.0, 8.0)?;

        context.translate(board_offset.0 as f64, board_offset.1 as f64)?;

        self.particle_system.tick_and_draw(context, atlas, frame)?;

        self.level
            .mages
            .sort_by(|a, b| a.position.1.cmp(&b.position.1));

        // DRAW powerups
        for (position, powerup) in self.level.powerups.iter() {
            context.save();

            context.translate(
                16.0 + position.0 as f64 * board_scale.0,
                16.0 + position.1 as f64 * board_scale.1,
            )?;
            draw_powerup(context, atlas, position, powerup, frame)?;

            if let Some(particle_sort) = ParticleSort::for_powerup(powerup) {
                for _ in 0..1 {
                    let d = js_sys::Math::random() * std::f64::consts::TAU;
                    let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.05;
                    self.particle_system.add(Particle::new(
                        (position.0 as f64, position.1 as f64),
                        (d.cos() * v, d.sin() * v),
                        (js_sys::Math::random() * 20.0) as u64,
                        particle_sort,
                    ));
                }
            }

            context.restore();
        }

        // DRAW mages
        for mage in &self.level.mages {
            context.save();

            context.translate(
                16.0 + mage.position.0 as f64 * board_scale.0,
                16.0 + mage.position.1 as f64 * board_scale.1,
            )?;

            draw_mage(context, atlas, mage, frame, mage.team, true, None)?;
            draw_mana(context, atlas, mage)?;

            context.restore();
        }

        let selected_tile = self.level.board.location_as_position(
            pointer.location,
            (board_offset.0 - 32, board_offset.1),
            BOARD_SCALE,
        );

        if let Some(selected_tile) = selected_tile {
            draw_crosshair(context, atlas, &selected_tile, (32.0, 32.0), frame)?;
        }

        let board_scale = tuple_as!(BOARD_SCALE, f64);
        let board_offset = tuple_as!(board_offset, f64);

        match &self.selection {
            EditorSelection::Mage(mage) => {
                if let Some(selected_tile) = selected_tile {
                    for position in &mage.targets(&self.level.board, &selected_tile) {
                        draw_sprite(
                            context,
                            atlas,
                            80.0,
                            32.0,
                            16.0,
                            16.0,
                            position.0 as f64 * board_scale.0 + 8.0,
                            position.1 as f64 * board_scale.1 + 8.0,
                        )?;
                    }
                }

                interface_context.save();
                interface_context.translate(
                    (pointer.location.0 as f64).clamp(
                        board_offset.0 - 16.0,
                        board_offset.0 - 48.0 + board_scale.0 * self.level.board.width as f64,
                    ),
                    (pointer.location.1 as f64).clamp(
                        board_offset.1 + 16.0,
                        board_offset.1 - 16.0 + board_scale.1 * self.level.board.height as f64,
                    ),
                )?;
                draw_mage(interface_context, atlas, mage, frame, mage.team, true, None)?;
                interface_context.restore();

                interface_context.save();
                interface_context.translate(276.0, 40.0)?;
                draw_mage(interface_context, atlas, mage, frame, mage.team, true, None)?;
                draw_mana(interface_context, atlas, mage)?;
                interface_context.restore();
            }
            EditorSelection::PowerUp(powerup) => {
                interface_context.save();
                interface_context.translate(
                    (pointer.location.0 as f64).clamp(
                        board_offset.0 - 16.0,
                        board_offset.0 - 48.0 + board_scale.0 * self.level.board.width as f64,
                    ),
                    (pointer.location.1 as f64).clamp(
                        board_offset.1 + 16.0,
                        board_offset.1 - 16.0 + board_scale.1 * self.level.board.height as f64,
                    ),
                )?;
                draw_powerup(interface_context, atlas, &Position(0, 0), powerup, frame)?;
                interface_context.restore();

                interface_context.save();
                interface_context.translate(276.0, 40.0)?;
                draw_powerup(interface_context, atlas, &Position(0, 0), powerup, frame)?;
                interface_context.restore();
            }
            EditorSelection::Tile(position) => {
                draw_crosshair(context, atlas, position, (48.0, 32.0), frame)?;

                if let Some(mage) = self.level.mages.occupant(position) {
                    for position in &mage.targets(&self.level.board, position) {
                        draw_sprite(
                            context,
                            atlas,
                            80.0,
                            32.0,
                            16.0,
                            16.0,
                            position.0 as f64 * board_scale.0 + 8.0,
                            position.1 as f64 * board_scale.1 + 8.0,
                        )?;
                    }

                    self.mage_interface
                        .draw(interface_context, atlas, pointer, frame)?;

                    interface_context.save();
                    interface_context.translate(276.0, 40.0)?;
                    draw_mage(interface_context, atlas, mage, frame, mage.team, true, None)?;
                    draw_mana(interface_context, atlas, mage)?;

                    interface_context.translate(-20.0, 40.0)?;

                    draw_spell_pattern(interface_context, atlas, mage)?;

                    interface_context.translate(20.0, 44.0)?;

                    if mage.mana > 0 {
                        draw_mana(interface_context, atlas, mage)?;
                    } else {
                        draw_sprite(interface_context, atlas, 112.0, 16.0, 16.0, 16.0, -8.0, 4.0)?;
                    }

                    interface_context.restore();
                } else if let Some(powerup) = self.level.powerups.get(position) {
                    self.prop_interface
                        .draw(interface_context, atlas, pointer, frame)?;

                    interface_context.save();
                    interface_context.translate(276.0, 40.0)?;
                    draw_powerup(interface_context, atlas, &Position(0, 0), powerup, frame)?;
                    interface_context.restore();
                } else {
                    self.no_mage_interface
                        .draw(interface_context, atlas, pointer, frame)?;
                }
            }
            EditorSelection::None => (),
        }

        context.restore();

        self.interface
            .draw(interface_context, atlas, pointer, frame)?;

        self.button_menu
            .draw(interface_context, atlas, pointer, frame)?;

        if self.is_interface_active() {
            self.menu_interface
                .draw(interface_context, atlas, pointer, frame)?;
        }

        if pointer.location.0 >= 244 + 16
            && pointer.location.0 < 308 - 16
            && pointer.location.1 >= 8 + 16
            && pointer.location.1 < 72 - 16
        {
            if pointer.clicked() {
                self.level.board.style = self.level.board.style.next();
                self.board_dirty = true;
            } else if pointer.alt_clicked() {
                self.level.board.style = self.level.board.style.previous();
                self.board_dirty = true;
            }
        }

        Ok(())
    }

    fn tick(
        &mut self,
        text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let board_offset = self.board_offset();
        let pointer = &app_context.pointer;

        if text_input.dataset().get("field").is_some() {
            return None;
        }

        if let Some((field, value)) = &app_context.text_input {
            if field == "level_code" {
                return Some(StateSort::Editor(Editor::new(value.as_str().into())));
            }
        }

        if self.is_interface_active() {
            if let Some(UIEvent::ButtonClick(value, clip_id)) = self.menu_interface.tick(pointer) {
                app_context.audio_system.play_clip_option(clip_id);

                match value {
                    BUTTON_LOAD => {
                        text_input.set_value(self.level.as_code().as_str());
                        text_input.set_placeholder("Enter level code");
                        text_input.dataset().set("field", "level_code").unwrap();
                        text_input.focus().unwrap();
                        self.button_menu.set_selected(false);
                    }
                    BUTTON_SIMULATE => {
                        // let simulations = Level::simulate(
                        //     Level::new(
                        //         self.level.board.clone(),
                        //         self.level.mages.clone(),
                        //         Team::Red,
                        //     ),
                        //     5,
                        //     window().performance().unwrap().now() as u64,
                        // );

                        // console::log_1(
                        //     &format!(
                        //         "{}",
                        //         simulations
                        //             .iter()
                        //             .map(|game| {
                        //                 console::log_1(
                        //                     &format!(
                        //                         "{} {} {:?}",
                        //                         game.turns(),
                        //                         game.evaluate(),
                        //                         game
                        //                     )
                        //                     .into(),
                        //                 );
                        //                 game.evaluate()
                        //             })
                        //             .sum::<isize>()
                        //             / 5
                        //     )
                        //     .into(),
                        // );
                    }
                    BUTTON_RESET => {
                        return Some(StateSort::Editor(Editor::new(Level::default())));
                    }
                    BUTTON_LEAVE => {
                        return Some(StateSort::MainMenu(MainMenu::default()));
                    }
                    _ => (),
                }
            }
        } else {
            if let Some(UIEvent::ButtonClick(value, clip_id)) = self.interface.tick(pointer) {
                app_context.audio_system.play_clip_option(clip_id);

                match value {
                    BUTTON_SAVE => {
                        App::save_level(0, self.level.clone());

                        text_input.set_value(self.level.as_code().as_str());
                        text_input
                            .dataset()
                            .set("field", "save_level_code")
                            .unwrap();
                        text_input.focus().unwrap();
                    }
                    BUTTON_MODE_TOGGLE => {
                        // return Some(StateSort::Lobby(LobbyState::new(LobbySettings {
                        //     lobby_sort: shared::LobbySort::Local,
                        //     loadout_method: shared::LoadoutMethod::Prefab(self.level.mages.clone()),
                        //     board: self.level.board.clone(),
                        //     ..Default::default()
                        // })));
                        return Some(StateSort::EditorPreview(EditorPreview::new(
                            self.level.clone(),
                        )));
                    }
                    BUTTON_WIDTH_MINUS => {
                        let min_width = self
                            .level
                            .mages
                            .iter()
                            .map(|mage| mage.position.0)
                            .reduce(|acc, e| acc.max(e))
                            .unwrap_or_default() as usize;

                        let min_width = min_width.max(
                            self.level
                                .powerups
                                .keys()
                                .max_by(|a, b| a.0.cmp(&b.0))
                                .map(|pos| pos.0)
                                .unwrap_or_default() as usize,
                        );

                        if self.level.board.width - 1 <= min_width {
                            for _ in 0..self.level.board.width * 5 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (
                                        self.level.board.width as f64 - 0.5,
                                        js_sys::Math::random() * self.level.board.height as f64
                                            - 0.5,
                                    ),
                                    (-v, d.sin() * v * 0.1),
                                    (js_sys::Math::random() * 40.0) as u64,
                                    ParticleSort::Diagonals,
                                ));
                            }
                        } else if let Ok(board) =
                            Board::new(self.level.board.width - 1, self.level.board.height)
                        {
                            self.level.board = board;
                            self.board_dirty = true;
                        }
                    }
                    BUTTON_WIDTH_PLUS => {
                        if let Ok(board) =
                            Board::new(self.level.board.width + 1, self.level.board.height)
                        {
                            self.level.board = board;
                            self.board_dirty = true;
                        }
                    }
                    BUTTON_HEIGHT_MINUS => {
                        let min_height = self
                            .level
                            .mages
                            .iter()
                            .map(|mage| mage.position.1)
                            .reduce(|acc, e| acc.max(e))
                            .unwrap_or_default() as usize;

                        let min_height = min_height.max(
                            self.level
                                .powerups
                                .keys()
                                .max_by(|a, b| a.1.cmp(&b.1))
                                .map(|pos| pos.1)
                                .unwrap_or_default() as usize,
                        );

                        if self.level.board.height - 1 <= min_height {
                            for _ in 0..self.level.board.height * 5 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (
                                        js_sys::Math::random() * self.level.board.width as f64
                                            - 0.5,
                                        self.level.board.height as f64 - 0.5,
                                    ),
                                    (d.cos() * v * 0.1, -v),
                                    (js_sys::Math::random() * 40.0) as u64,
                                    ParticleSort::Diagonals,
                                ));
                            }
                        } else if let Ok(board) =
                            Board::new(self.level.board.width, self.level.board.height - 1)
                        {
                            self.level.board = board;
                            self.board_dirty = true;
                        }
                    }
                    BUTTON_HEIGHT_PLUS => {
                        if let Ok(board) =
                            Board::new(self.level.board.width, self.level.board.height + 1)
                        {
                            self.level.board = board;
                            self.board_dirty = true;
                        }
                    }

                    _ => (),
                }
            }

            match self.selection {
                EditorSelection::Mage(_) => (),
                EditorSelection::PowerUp(_) => (),
                EditorSelection::Tile(position) => {
                    if self.level.mages.occupant(&position).is_some() {
                        if let Some(UIEvent::ButtonClick(value, clip_id)) =
                            self.mage_interface.tick(pointer)
                        {
                            app_context.audio_system.play_clip_option(clip_id);

                            match value {
                                BUTTON_TEAM_LEFT => {
                                    if let Some(selected_mage) =
                                        self.level.mages.occupant_mut(&position)
                                    {
                                        selected_mage.team = selected_mage.team.enemy();
                                    }
                                }
                                BUTTON_TEAM_RIGHT => {
                                    if let Some(selected_mage) =
                                        self.level.mages.occupant_mut(&position)
                                    {
                                        selected_mage.team = selected_mage.team.enemy();
                                    }
                                }
                                BUTTON_SPELL_LEFT => {
                                    if let EditorSelection::Tile(position) = self.selection {
                                        if let Some(selected_mage) =
                                            self.level.mages.occupant_mut(&position)
                                        {
                                            *selected_mage = Mage::editor_new(
                                                selected_mage.index,
                                                selected_mage.team,
                                                selected_mage.sort.previous(),
                                                selected_mage.mana.clone(),
                                                selected_mage.position,
                                            );
                                        }
                                    }
                                }
                                BUTTON_SPELL_RIGHT => {
                                    if let EditorSelection::Tile(position) = self.selection {
                                        if let Some(selected_mage) =
                                            self.level.mages.occupant_mut(&position)
                                        {
                                            *selected_mage = Mage::editor_new(
                                                selected_mage.index,
                                                selected_mage.team,
                                                selected_mage.sort.next(),
                                                selected_mage.mana.clone(),
                                                selected_mage.position,
                                            );
                                        }
                                    }
                                }
                                BUTTON_MANA_LEFT => {
                                    if let EditorSelection::Tile(position) = self.selection {
                                        if let Some(selected_mage) =
                                            self.level.mages.occupant_mut(&position)
                                        {
                                            selected_mage.mana -= 1;
                                        }
                                    }
                                }
                                BUTTON_MANA_RIGHT => {
                                    if let EditorSelection::Tile(position) = self.selection {
                                        if let Some(selected_mage) =
                                            self.level.mages.occupant_mut(&position)
                                        {
                                            selected_mage.mana += 1;
                                        }
                                    }
                                }
                                BUTTON_DELETE => {
                                    if let EditorSelection::Tile(position) = self.selection {
                                        self.level.mages.retain(|mage| mage.position != position);

                                        for _ in 0..40 {
                                            let d = js_sys::Math::random() * std::f64::consts::TAU;
                                            let v = (js_sys::Math::random()
                                                + js_sys::Math::random())
                                                * 0.1;
                                            self.particle_system.add(Particle::new(
                                                (
                                                    position.0 as f64 + d.cos() * 1.25,
                                                    position.1 as f64 + d.sin() * 1.25,
                                                ),
                                                (-d.cos() * v, -d.sin() * v),
                                                (js_sys::Math::random() * 20.0) as u64,
                                                ParticleSort::Missile,
                                            ));
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    } else if self.level.powerups.get(&position).is_some() {
                        if let Some(UIEvent::ButtonClick(value, clip_id)) =
                            self.prop_interface.tick(pointer)
                        {
                            app_context.audio_system.play_clip_option(clip_id);

                            match value {
                                BUTTON_TEAM_LEFT => {
                                    if let Some(selected_powerup) =
                                        self.level.powerups.get_mut(&position)
                                    {
                                        *selected_powerup = selected_powerup.next();
                                    }
                                }
                                BUTTON_TEAM_RIGHT => {
                                    if let Some(selected_powerup) =
                                        self.level.powerups.get_mut(&position)
                                    {
                                        *selected_powerup = selected_powerup.previous();
                                    }
                                }
                                BUTTON_DELETE => {
                                    self.level.powerups.remove(&position);

                                    for _ in 0..40 {
                                        let d = js_sys::Math::random() * std::f64::consts::TAU;
                                        let v =
                                            (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                        self.particle_system.add(Particle::new(
                                            (
                                                position.0 as f64 + d.cos() * 1.25,
                                                position.1 as f64 + d.sin() * 1.25,
                                            ),
                                            (-d.cos() * v, -d.sin() * v),
                                            (js_sys::Math::random() * 20.0) as u64,
                                            ParticleSort::Missile,
                                        ));
                                    }
                                }
                                _ => (),
                            }
                        }
                    } else if let Some(UIEvent::ButtonClick(value, clip_id)) =
                        self.no_mage_interface.tick(pointer)
                    {
                        app_context.audio_system.play_clip_option(clip_id);

                        match value {
                            BUTTON_ADD_MAGE => {
                                if !self.occupied(&position) {
                                    self.level.mages.push(Mage::new(
                                        self.level.mage_index,
                                        Team::Red,
                                        shared::MageSort::Cross,
                                        position,
                                    ));
                                    self.level.mage_index += 1;

                                    self.sparkle_create(position);
                                }
                            }
                            BUTTON_ADD_PROP => {
                                if !self.occupied(&position) {
                                    self.level.powerups.insert(position, PowerUp::Diagonal);

                                    self.sparkle_create(position);
                                }
                            }
                            _ => (),
                        }
                    }
                }
                EditorSelection::None => (),
            }
        }

        if let Some(selected_tile) = self.level.board.location_as_position(
            pointer.location,
            (board_offset.0 - 32, board_offset.1),
            BOARD_SCALE,
        ) {
            if pointer.clicked() {
                for _ in 0..10 {
                    let d = js_sys::Math::random() * std::f64::consts::TAU;
                    let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                    self.particle_system.add(Particle::new(
                        (selected_tile.0 as f64, selected_tile.1 as f64),
                        (d.cos() * v, d.sin() * v),
                        (js_sys::Math::random() * 20.0) as u64,
                        ParticleSort::Missile,
                    ));
                }

                if let Some(selected_mage) = self.level.mages.occupant(&selected_tile).cloned() {
                    match &mut self.selection {
                        EditorSelection::Tile(tile) if *tile == selected_tile => {
                            self.selection = EditorSelection::Mage(selected_mage);
                        }
                        EditorSelection::Mage(mage) => {
                            mage.position = selected_tile;
                            self.level.mages.push(mage.clone());

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }

                            self.selection = EditorSelection::Mage(selected_mage);
                        }
                        EditorSelection::PowerUp(powerup) => {
                            self.level.powerups.insert(selected_tile, *powerup);

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }

                            self.selection = EditorSelection::Mage(selected_mage);
                        }
                        _ => self.selection = EditorSelection::Tile(selected_tile),
                    }
                } else if let Some(selected_powerup) =
                    self.level.powerups.get(&selected_tile).cloned()
                {
                    match &mut self.selection {
                        EditorSelection::Tile(tile) if *tile == selected_tile => {
                            self.level.powerups.remove(&selected_tile);
                            self.selection = EditorSelection::PowerUp(selected_powerup);
                        }
                        EditorSelection::PowerUp(powerup) => {
                            self.level.powerups.insert(selected_tile, *powerup);

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }

                            self.selection = EditorSelection::PowerUp(selected_powerup);
                        }
                        EditorSelection::Mage(mage) => {
                            self.level.powerups.remove(&selected_tile);
                            mage.position = selected_tile;
                            self.level.mages.push(mage.clone());

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }

                            self.selection = EditorSelection::PowerUp(selected_powerup);
                        }
                        _ => self.selection = EditorSelection::Tile(selected_tile),
                    }
                } else {
                    match mem::replace(&mut self.selection, EditorSelection::Tile(selected_tile)) {
                        EditorSelection::Mage(mut selected_mage) => {
                            selected_mage.position = selected_tile;
                            self.level.mages.push(selected_mage);

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }
                        }
                        EditorSelection::PowerUp(selected_powerup) => {
                            self.level.powerups.insert(selected_tile, selected_powerup);

                            for _ in 0..40 {
                                let d = js_sys::Math::random() * std::f64::consts::TAU;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                                self.particle_system.add(Particle::new(
                                    (selected_tile.0 as f64, selected_tile.1 as f64),
                                    (d.cos() * v * 2.0, d.sin() * v),
                                    (js_sys::Math::random() * 20.0) as u64,
                                    ParticleSort::Missile,
                                ));
                            }
                        }
                        EditorSelection::Tile(_) => {}
                        EditorSelection::None => {}
                    }
                }
            } else if pointer.alt_clicked() {
                if let EditorSelection::Mage(_mage) = &self.selection {
                    for _ in 0..40 {
                        let d = js_sys::Math::random() * std::f64::consts::TAU;
                        let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                        self.particle_system.add(Particle::new(
                            (
                                selected_tile.0 as f64 + d.cos() * 1.25,
                                selected_tile.1 as f64 + d.sin() * 1.25,
                            ),
                            (-d.cos() * v, -d.sin() * v),
                            (js_sys::Math::random() * 20.0) as u64,
                            ParticleSort::Missile,
                        ));
                    }
                } else if let EditorSelection::PowerUp(_) = &self.selection {
                    for _ in 0..40 {
                        let d = js_sys::Math::random() * std::f64::consts::TAU;
                        let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                        self.particle_system.add(Particle::new(
                            (
                                selected_tile.0 as f64 + d.cos() * 1.25,
                                selected_tile.1 as f64 + d.sin() * 1.25,
                            ),
                            (-d.cos() * v, -d.sin() * v),
                            (js_sys::Math::random() * 20.0) as u64,
                            ParticleSort::Missile,
                        ));
                    }
                }

                self.selection = EditorSelection::None;
            }
        }

        match &mut self.selection {
            EditorSelection::Mage(selected_mage) => {
                self.level
                    .mages
                    .retain(|mage| mage.index != selected_mage.index);
            }
            EditorSelection::Tile(position) => {
                *position = self.level.board.clamp_position(*position);
            }
            _ => (),
        }

        self.button_menu.tick(pointer);

        None
    }
}

impl Default for Editor {
    fn default() -> Self {
        Editor::new(App::load_level(0).unwrap_or_default())
    }
}
