use shared::{LoadoutMethod, Lobby, LobbySettings, LobbySort, Team};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{Game, MainMenu, State, TeleportMenu};
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ButtonGroupElement, Interface, LabelTheme, LabelTrim,
        StateSort, UIElement, UIEvent,
    },
    draw::{draw_mage, draw_mana, draw_sprite},
    window,
};

pub struct SkirmishMenu {
    interface: Interface,
    sentinel_lobby: Lobby,
    lobby_settings: LobbySettings,
}

const BUTTON_LOCAL: usize = 1;
const BUTTON_VS_AI: usize = 2;
const BUTTON_ONLINE: usize = 3;
const BUTTON_DEFAULT: usize = 10;
const BUTTON_RANDOM: usize = 11;
const BUTTON_SYMMETRIC_RANDOM: usize = 12;
// const BUTTON_ROUND_ROBIN: usize = 13;
const BUTTON_BATTLE: usize = 20;
const BUTTON_BACK: usize = 21;
const BUTTON_TELEPORT: usize = 30;

impl SkirmishMenu {
    fn refresh_lobby(&mut self) {
        self.lobby_settings.seed = window().performance().unwrap().now() as u64;
        self.sentinel_lobby = Lobby::new(self.lobby_settings.clone());
    }
}

impl State for SkirmishMenu {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        let frame = app_context.frame;
        let pointer = &app_context.pointer;

        context.save();
        context.translate(16.0, 32.0)?;
        draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

        draw_sprite(context, atlas, 96.0, 64.0, 32.0, 40.0, 16.0, -2.0)?;
        context.translate(80.0, 0.0)?;
        draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

        draw_sprite(context, atlas, 64.0, 64.0, 32.0, 40.0, 26.0, -4.0)?;
        draw_sprite(context, atlas, 128.0, 104.0, 32.0, 40.0, 6.0, -2.0)?;
        context.translate(80.0, 0.0)?;
        draw_sprite(context, atlas, 256.0, 256.0, 64.0, 32.0, 0.0, 0.0)?;

        draw_sprite(context, atlas, 32.0, 64.0, 32.0, 40.0, 6.0, -4.0)?;
        draw_sprite(context, atlas, 0.0, 256.0, 32.0, 40.0, 26.0, -2.0)?;
        context.restore();

        context.save();
        context.translate(108.0, 112.0)?;

        draw_sprite(context, atlas, 256.0, 320.0, 128.0, 64.0, 0.0, 0.0)?;

        for mage in self.sentinel_lobby.game.iter_mages() {
            context.save();
            context.translate(
                -16.0 + mage.position.0 as f64 * 32.0,
                15.0 + if mage.team == Team::Red { 0.0 } else { 1.0 } * 32.0,
            )?;
            draw_mage(
                context,
                atlas,
                mage,
                frame,
                self.sentinel_lobby.game.starting_team(),
                true,
                None,
            )?;
            draw_mana(context, atlas, mage)?;
            context.restore();
        }

        self.interface
            .draw(interface_context, atlas, pointer, frame)?;

        context.restore();

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let pointer = &app_context.pointer;

        if let Some(UIEvent::ButtonClick(value, clip_id)) = self.interface.tick(pointer) {
            app_context.audio_system.play_clip_option(clip_id);

            match value {
                BUTTON_LOCAL => {
                    self.lobby_settings.lobby_sort = LobbySort::Local;
                }
                BUTTON_VS_AI => {
                    self.lobby_settings.lobby_sort = LobbySort::LocalAI;
                }
                BUTTON_ONLINE => {
                    self.lobby_settings.lobby_sort = LobbySort::Online(0);
                }
                BUTTON_DEFAULT => {
                    self.lobby_settings.loadout_method = LoadoutMethod::Default;
                    self.refresh_lobby();
                }
                BUTTON_RANDOM => {
                    self.lobby_settings.loadout_method = LoadoutMethod::Random { symmetric: false };
                    self.refresh_lobby();
                }
                BUTTON_SYMMETRIC_RANDOM => {
                    self.lobby_settings.loadout_method = LoadoutMethod::Random { symmetric: true };
                    self.refresh_lobby();
                }
                BUTTON_BATTLE => {
                    return Some(StateSort::Game(Game::new(self.lobby_settings.clone())));
                }
                BUTTON_TELEPORT => {
                    return Some(StateSort::TeleportMenu(TeleportMenu::default()));
                }
                BUTTON_BACK => {
                    return Some(StateSort::MainMenu(MainMenu::default()));
                }
                _ => (),
            }
        }

        None
    }
}

impl Default for SkirmishMenu {
    fn default() -> SkirmishMenu {
        let button_local = ButtonElement::new(
            (0, 0),
            (72, 32),
            BUTTON_LOCAL,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Local".to_string(), Alignment::Center),
        );
        let button_vs_ai = ButtonElement::new(
            (80, 0),
            (72, 32),
            BUTTON_VS_AI,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("AI".to_string(), Alignment::Center),
        );

        #[cfg(not(feature = "demo"))]
        let button_online = ButtonElement::new(
            (160, 0),
            (72, 32),
            BUTTON_ONLINE,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Online".to_string(), Alignment::Center),
        );

        #[cfg(feature = "demo")]
        let button_online = ButtonElement::new(
            (160, 0),
            (72, 32),
            BUTTON_ONLINE,
            LabelTrim::Round,
            LabelTheme::Disabled,
            crate::app::ContentElement::Text("Online".to_string(), Alignment::Center),
        );

        let group_lobby_type = ButtonGroupElement::new(
            (12, 64),
            vec![button_local, button_vs_ai, button_online],
            BUTTON_LOCAL,
        );

        let button_default = ButtonElement::new(
            (0, 0),
            (80, 18),
            BUTTON_DEFAULT,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Default".to_string(), Alignment::Center),
        );
        let button_random = ButtonElement::new(
            (0, 22),
            (80, 18),
            BUTTON_SYMMETRIC_RANDOM,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Random".to_string(), Alignment::Center),
        );

        let button_symmetric_random = ButtonElement::new(
            (0, 22 * 2),
            (80, 18),
            BUTTON_RANDOM,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Chaos".to_string(), Alignment::Center),
        );

        // let button_round_robin = ButtonElement::new(
        //     (0, 22 * 3),
        //     (80, 18),
        //     BUTTON_ROUND_ROBIN,
        //     ButtonTrim::Round,
        //     ButtonClass::Default,
        //     crate::app::ContentElement::Text("Draft".to_string(), Alignment::Center),
        // );

        let group_loadout_type = ButtonGroupElement::new(
            (16, 112),
            vec![
                button_default,
                button_random,
                button_symmetric_random,
                // button_round_robin,
            ],
            BUTTON_DEFAULT,
        );

        let button_battle = ButtonElement::new(
            (64, 188),
            (128, 24),
            BUTTON_BATTLE,
            LabelTrim::Glorious,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Battle".to_string(), Alignment::Center),
        );

        let button_teleport = ButtonElement::new(
            (35, 220),
            (88, 16),
            BUTTON_TELEPORT,
            LabelTrim::Glorious,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Join".to_string(), Alignment::Center),
        );

        let button_back = ButtonElement::new(
            (129, 220),
            (88, 16),
            BUTTON_BACK,
            LabelTrim::Return,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Back".to_string(), Alignment::Center),
        );

        let root_element = Interface::new(vec![
            group_lobby_type.boxed(),
            group_loadout_type.boxed(),
            button_battle.boxed(),
            button_teleport.boxed(),
            button_back.boxed(),
        ]);

        SkirmishMenu {
            interface: root_element,
            sentinel_lobby: Lobby::new(LobbySettings::default()),
            lobby_settings: LobbySettings::default(),
        }
    }
}
