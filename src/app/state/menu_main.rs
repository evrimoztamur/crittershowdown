use shared::{Board, LoadoutMethod, LobbySettings, LobbySort};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{ArenaMenu, Editor, Game, SkirmishMenu, State, Tutorial};
use crate::{
    app::{
        Alignment, AppContext, ButtonElement, ConfirmButtonElement, Interface, LabelTheme,
        LabelTrim, Pointer, StateSort, UIElement, UIEvent,
    },
    window,
};

pub struct MainMenu {
    interface: Interface,
    button_reset: ConfirmButtonElement,
    preview_state: Game,
}

const BUTTON_ARENA: usize = 20;
const BUTTON_SKIRMISH: usize = 21;
const BUTTON_TUTORIAL: usize = 22;
const BUTTON_EDITOR: usize = 23;
const BUTTON_RESET: usize = 50;

impl State for MainMenu {
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

        context.translate(-72.0, 0.0)?;

        self.preview_state.draw_game(
            context,
            interface_context,
            atlas,
            frame,
            &Pointer::default(),
        )?;

        context.restore();

        self.interface
            .draw(interface_context, atlas, pointer, frame)?;

        if self.preview_state.lobby().finished() {
            self.button_reset
                .draw(interface_context, atlas, pointer, frame)?;
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

        self.preview_state.tick_game(frame, app_context);

        if self.preview_state.frames_since_last_move(frame) > 70 {
            self.preview_state.take_best_turn_quick();
        }

        if let Some(UIEvent::ButtonClick(value, clip_id)) = self.interface.tick(pointer) {
            app_context.audio_system.play_clip_option(clip_id);

            match value {
                BUTTON_ARENA => {
                    return Some(StateSort::ArenaMenu(ArenaMenu::default()));
                }
                BUTTON_EDITOR => {
                    return Some(StateSort::Editor(Editor::default()));
                }
                BUTTON_SKIRMISH => {
                    return Some(StateSort::SkirmishMenu(SkirmishMenu::default()));
                }
                BUTTON_TUTORIAL => {
                    return Some(StateSort::Tutorial(Tutorial::default()));
                }
                _ => (),
            }
        }

        if self.preview_state.lobby().finished() && self.button_reset.tick(pointer).is_some() {
            return Some(StateSort::MainMenu(MainMenu::default()));
        }

        None
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        let button_arena = ButtonElement::new(
            (192, 68),
            (112, 24),
            BUTTON_ARENA,
            LabelTrim::Glorious,
            LabelTheme::Action,
            crate::app::ContentElement::Text("Campaign".to_string(), Alignment::Center),
        );

        let button_skirmish = ButtonElement::new(
            (200, 68 + 32),
            (96, 20),
            BUTTON_SKIRMISH,
            LabelTrim::Glorious,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Skirmish".to_string(), Alignment::Center),
        );

        let button_editor = ButtonElement::new(
            (208, 68 + 32 * 2 + 4),
            (80, 20),
            BUTTON_EDITOR,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Editor".to_string(), Alignment::Center),
        );

        let button_tutorial = ButtonElement::new(
            (208, 68 + 32 * 3),
            (80, 20),
            BUTTON_TUTORIAL,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Text("Tutorial".to_string(), Alignment::Center),
        );

        let root_element = Interface::new(vec![
            button_arena.boxed(),
            button_editor.boxed(),
            button_tutorial.boxed(),
            button_skirmish.boxed(),
        ]);

        let button_reset = ConfirmButtonElement::new(
            (208 - 164, 64 + 166),
            (24, 20),
            BUTTON_RESET,
            LabelTrim::Round,
            LabelTheme::Default,
            crate::app::ContentElement::Sprite((128, 16), (16, 16)),
        );

        MainMenu {
            interface: root_element,
            button_reset,
            preview_state: Game::new(LobbySettings {
                lobby_sort: LobbySort::Local,
                loadout_method: LoadoutMethod::DefaultBoard(Board::new(6, 7).unwrap()),
                seed: window().performance().unwrap().now() as u64,
                can_stalemate: true,
            }),
        }
    }
}
