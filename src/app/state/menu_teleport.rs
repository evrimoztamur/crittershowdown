use shared::{Board, BoardStyle, LobbySettings, LobbySort};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{Game, SkirmishMenu, State};
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ClipId, Interface, LabelTheme, LabelTrim, Particle,
        ParticleSort, ParticleSystem, StateSort, UIElement, UIEvent, BOARD_SCALE,
    },
    draw::{draw_board, draw_crosshair, draw_sprite},
    tuple_as,
};

pub struct TeleportMenu {
    interface: Interface,
    lobby_id: u16,
    particle_system: ParticleSystem,
    board_dirty: bool,
    board: Board,
}

const BOARD_OFFSET: (i32, i32) = ((4 * BOARD_SCALE.0) / 2, (4 * BOARD_SCALE.1) / 2 - 16);

const BUTTON_TELEPORT: usize = 20;
const BUTTON_BACK: usize = 21;
impl State for TeleportMenu {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let board_scale = tuple_as!(BOARD_SCALE, f64);

        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        if self.board_dirty {
            self.board_dirty = false;
            draw_board(atlas, 256.0, 0.0, &self.board, 8, 8).unwrap();
        }

        context.save();

        context.translate(0.0, -16.0)?;

        draw_sprite(context, atlas, 256.0, 0.0, 256.0, 256.0, 0.0, 0.0)?;

        context.translate(64.0, 64.0)?;

        self.particle_system.tick_and_draw(context, atlas, frame)?;

        if let Some(selected_tile) = self.board.location_as_position(
            pointer.location,
            (BOARD_OFFSET.0, BOARD_OFFSET.1),
            BOARD_SCALE,
        ) {
            if self.lobby_id & (1 << ((selected_tile.1 << 2) | selected_tile.0)) as u16 == 0 {
                draw_crosshair(context, atlas, &selected_tile, (32.0, 32.0), frame)?;
            } else {
                draw_crosshair(context, atlas, &selected_tile, (64.0, 32.0), frame)?;
            }
        }

        let mut lid = self.lobby_id;

        while lid != 0 {
            let tz = lid.trailing_zeros();
            let x = tz % 4;
            let y = tz / 4;

            lid ^= 1 << tz;

            draw_sprite(
                context,
                atlas,
                96.0,
                32.0,
                16.0,
                16.0,
                x as f64 * board_scale.0 + 8.0,
                y as f64 * board_scale.1 + 8.0,
            )?;
        }

        context.restore();

        self.interface
            .draw(interface_context, atlas, pointer, frame)?;

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let pointer = &app_context.pointer;

        if let Some(selected_tile) = self.board.location_as_position(
            pointer.location,
            (BOARD_OFFSET.0, BOARD_OFFSET.1),
            BOARD_SCALE,
        ) {
            if pointer.clicked() {
                for _ in 0..40 {
                    let d = js_sys::Math::random() * std::f64::consts::TAU;
                    let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                    self.particle_system.add(Particle::new(
                        (selected_tile.0 as f64, selected_tile.1 as f64),
                        (d.cos() * v, d.sin() * v),
                        (js_sys::Math::random() * 20.0) as u64,
                        ParticleSort::Missile,
                    ));
                }

                self.lobby_id ^= 1 << ((selected_tile.1 << 2) | selected_tile.0);
            }
        }

        if let Some(UIEvent::ButtonClick(value, clip_id)) = self.interface.tick(pointer) {
            app_context.audio_system.play_clip_option(clip_id);

            match value {
                BUTTON_BACK => {
                    return Some(StateSort::SkirmishMenu(SkirmishMenu::default()));
                }
                BUTTON_TELEPORT => {
                    return Some(StateSort::Game(Game::new(LobbySettings {
                        lobby_sort: LobbySort::Online(self.lobby_id),
                        ..Default::default()
                    })));
                }
                _ => (),
            }
        }

        None
    }
}

impl Default for TeleportMenu {
    fn default() -> TeleportMenu {
        let button_teleport = ButtonElement::new(
            (8, 188),
            (96, 24),
            BUTTON_TELEPORT,
            LabelTrim::Glorious,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Teleport".to_string(), Alignment::Center),
        );

        let button_back = ButtonElement::new(
            (156, 192),
            (88, 16),
            BUTTON_BACK,
            LabelTrim::Return,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Back".to_string(), Alignment::Center),
        );

        let root_element = Interface::new(vec![button_teleport.boxed(), button_back.boxed()]);
        let board = Board::with_style(4, 4, BoardStyle::Teleport).unwrap();

        TeleportMenu {
            interface: root_element,
            lobby_id: 0,
            particle_system: ParticleSystem::default(),
            board_dirty: true,
            board,
        }
    }
}
