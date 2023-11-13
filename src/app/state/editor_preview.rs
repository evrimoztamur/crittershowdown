use shared::{Level, LobbySettings};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{Editor, Game, State};
use crate::{
    app::{
        Alignment, App, AppContext, ButtonElement, Interface, LabelTheme, LabelTrim, Particle,
        ParticleSort, ParticleSystem, StateSort, UIElement, UIEvent, BOARD_SCALE,
    },
    draw::{draw_board, draw_mage, draw_mana, draw_powerup, draw_sprite},
    tuple_as,
};

pub struct EditorPreview {
    interface: Interface,
    level: Level,
    particle_system: ParticleSystem,
    board_dirty: bool,
}

const BUTTON_BACK: usize = 0;
const BUTTON_LOCAL: usize = 1;
const BUTTON_VS_AI: usize = 2;
const BUTTON_ONLINE: usize = 3;

impl EditorPreview {
    pub fn new(level: Level) -> EditorPreview {
        let button_back = ButtonElement::new(
            (-60, 118),
            (20, 20),
            BUTTON_BACK,
            LabelTrim::Round,
            LabelTheme::Bright,
            crate::app::ContentElement::Sprite((160, 32), (16, 16)),
        );

        let button_local = ButtonElement::new(
            (240, 48),
            (72, 32),
            BUTTON_LOCAL,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Local".to_string(), Alignment::Center),
        );
        let button_vs_ai = ButtonElement::new(
            (240, 128),
            (72, 32),
            BUTTON_VS_AI,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("AI".to_string(), Alignment::Center),
        );

        #[cfg(not(feature = "demo"))]
        let button_online = ButtonElement::new(
            (240, 208),
            (72, 32),
            BUTTON_ONLINE,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Online".to_string(), Alignment::Center),
        );

        #[cfg(feature = "demo")]
        let button_online = ButtonElement::new(
            (240, 208),
            (72, 32),
            BUTTON_ONLINE,
            LabelTrim::Round,
            LabelTheme::Disabled,
            crate::app::ContentElement::Text("Online".to_string(), Alignment::Center),
        );

        let interface = Interface::new(vec![
            button_back.boxed(),
            button_local.boxed(),
            button_vs_ai.boxed(),
            button_online.boxed(),
        ]);

        EditorPreview {
            interface,
            level,
            particle_system: ParticleSystem::default(),
            board_dirty: true,
        }
    }

    pub fn board_offset(&self) -> (i32, i32) {
        (
            ((8 - self.level.board.width) as i32 * BOARD_SCALE.0) / 2,
            ((8 - self.level.board.height) as i32 * BOARD_SCALE.1) / 2,
        )
    }
}

impl State for EditorPreview {
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

        {
            context.save();

            context.translate(-32.0, 0.0)?;

            {
                context.save();

                draw_sprite(context, atlas, 256.0, 0.0, 256.0, 256.0, 0.0, 0.0)?;

                context.translate(board_offset.0 as f64, board_offset.1 as f64)?;

                // DRAW particles

                self.particle_system.tick_and_draw(context, atlas, frame)?;

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

                self.level
                    .mages
                    .sort_by(|a, b| a.position.1.cmp(&b.position.1));

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
                context.restore();
            }

            {
                context.translate(276.0, 16.0)?;

                draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

                draw_sprite(context, atlas, 96.0, 64.0, 32.0, 40.0, 16.0, -2.0)?;
            }

            {
                context.translate(0.0, 80.0)?;

                draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

                draw_sprite(context, atlas, 64.0, 64.0, 32.0, 40.0, 26.0, -4.0)?;
                draw_sprite(context, atlas, 128.0, 104.0, 32.0, 40.0, 6.0, -2.0)?;
            }

            {
                context.translate(0.0, 80.0)?;

                draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

                draw_sprite(context, atlas, 32.0, 64.0, 32.0, 40.0, 6.0, -4.0)?;
                draw_sprite(context, atlas, 0.0, 256.0, 32.0, 40.0, 26.0, -2.0)?;
            }

            context.restore();
        }

        self.interface
            .draw(interface_context, atlas, pointer, frame)?;

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let _board_offset = self.board_offset();
        let pointer = &app_context.pointer;

        if let Some(UIEvent::ButtonClick(value, clip_id)) = self.interface.tick(pointer) {
            app_context.audio_system.play_clip_option(clip_id);

            match value {
                BUTTON_BACK => {
                    return Some(StateSort::Editor(Editor::new(self.level.clone())));
                }
                BUTTON_LOCAL => {
                    return Some(StateSort::Game(Game::new(LobbySettings {
                        lobby_sort: shared::LobbySort::Local,
                        loadout_method: shared::LoadoutMethod::EditorPrefab(self.level.clone()),
                        ..Default::default()
                    })));
                }
                BUTTON_VS_AI => {
                    return Some(StateSort::Game(Game::new(LobbySettings {
                        lobby_sort: shared::LobbySort::LocalAI,
                        loadout_method: shared::LoadoutMethod::EditorPrefab(self.level.clone()),
                        ..Default::default()
                    })));
                }
                BUTTON_ONLINE => {
                    return Some(StateSort::Game(Game::new(LobbySettings {
                        lobby_sort: shared::LobbySort::Online(0),
                        loadout_method: shared::LoadoutMethod::EditorPrefab(self.level.clone()),
                        ..Default::default()
                    })));
                }
                _ => (),
            }
        }

        None
    }
}

impl Default for EditorPreview {
    fn default() -> Self {
        EditorPreview::new(App::load_level(0).unwrap_or_default())
    }
}
