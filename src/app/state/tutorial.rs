use shared::{GameResult, Level, LoadoutMethod, LobbySettings, LobbySort, Team};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{Game, MainMenu, State};
use crate::{
    app::{
        Alignment::Center, AppContext, ContentElement::Text, LabelTrim, Particle, ParticleSort,
        StateSort,
    },
    draw::{draw_label, draw_text_centered},
    window,
};

#[derive(PartialEq)]
enum TutorialStage {
    Movement,
    Attacking,
    Charging,
    FinalBlow,
    Victory,
}

pub struct Tutorial {
    pub game_state: Game,
    tutorial_stage: TutorialStage,
}

impl Tutorial {
    pub fn spark_board(&mut self) {
        let board_size = self.game_state.lobby().game.board_size();

        for _ in 0..board_size.0 * 8 {
            let d = js_sys::Math::random() * std::f64::consts::TAU;
            let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;

            self.game_state.particle_system().add(Particle::new(
                (js_sys::Math::random() * board_size.0 as f64 - 0.5, -0.5),
                (d.sin() * v * 0.2, -v),
                (js_sys::Math::random() * 40.0) as u64,
                ParticleSort::Diagonals,
            ));
        }
    }
}

impl State for Tutorial {
    fn draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        app_context: &AppContext,
    ) -> Result<(), JsValue> {
        match self.tutorial_stage {
            TutorialStage::Movement => {
                draw_label(
                    context,
                    atlas,
                    (80, 24),
                    (96, 16),
                    "#557F55",
                    &Text("Movement".to_string(), Center),
                    &app_context.pointer,
                    app_context.frame,
                    &LabelTrim::Glorious,
                    false,
                )?;

                draw_text_centered(interface_context, atlas, 128.0, 224.0, "Click the Red Mage")?;
                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    240.0,
                    "Then pick a square to move to",
                )?;
            }
            TutorialStage::Attacking => {
                draw_label(
                    context,
                    atlas,
                    (80, 24),
                    (96, 16),
                    "#557F55",
                    &Text("Attacking".to_string(), Center),
                    &app_context.pointer,
                    app_context.frame,
                    &LabelTrim::Glorious,
                    false,
                )?;

                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    224.0,
                    "Mages attack when they move",
                )?;
                draw_text_centered(interface_context, atlas, 128.0, 240.0, "Zap the Blue Mage!")?;
            }
            TutorialStage::Charging => {
                draw_label(
                    context,
                    atlas,
                    (80, 24),
                    (96, 16),
                    "#557F55",
                    &Text("Charging".to_string(), Center),
                    &app_context.pointer,
                    app_context.frame,
                    &LabelTrim::Glorious,
                    false,
                )?;

                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    224.0,
                    "Mages charge with powerups",
                )?;
                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    240.0,
                    "This one allows you to move diagonally!",
                )?;
            }
            TutorialStage::FinalBlow => {
                draw_label(
                    context,
                    atlas,
                    (80, 24),
                    (96, 16),
                    "#557F55",
                    &Text("Final Blow".to_string(), Center),
                    &app_context.pointer,
                    app_context.frame,
                    &LabelTrim::Glorious,
                    false,
                )?;
                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    232.0,
                    "Deal the final blow!",
                )?;
            }
            TutorialStage::Victory => {
                draw_label(
                    context,
                    atlas,
                    (80, 24),
                    (96, 16),
                    "#557F55",
                    &Text("Victory!".to_string(), Center),
                    &app_context.pointer,
                    app_context.frame,
                    &LabelTrim::Glorious,
                    false,
                )?;
                draw_text_centered(interface_context, atlas, 128.0, 224.0, "Congratulations!")?;
                draw_text_centered(
                    interface_context,
                    atlas,
                    128.0,
                    240.0,
                    "You won your first battle",
                )?;
            }
        }

        self.game_state
            .draw(context, interface_context, atlas, app_context)
    }

    fn tick(
        &mut self,
        text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        match self.tutorial_stage {
            TutorialStage::Movement => {
                if self.game_state.lobby().game.turns() > 0 {
                    self.tutorial_stage = TutorialStage::Attacking;

                    self.spark_board();
                }
            }
            TutorialStage::Attacking => {
                if self
                    .game_state
                    .lobby()
                    .game
                    .iter_mages()
                    .any(|mage| mage.has_diagonals())
                {
                    self.tutorial_stage = TutorialStage::Charging;

                    self.spark_board();
                }
            }
            TutorialStage::Charging => {
                if let Some(_enemy_mage) = self
                    .game_state
                    .lobby()
                    .game
                    .iter_mages()
                    .find(|mage| mage.mana == 1 && mage.team == Team::Blue)
                {
                    self.tutorial_stage = TutorialStage::FinalBlow;

                    self.spark_board();
                }
            }
            _ => {}
        }

        if self.tutorial_stage != TutorialStage::Victory
            && self.game_state.lobby().game.result() == Some(GameResult::Win(Team::Red))
        {
            self.tutorial_stage = TutorialStage::Victory;

            self.spark_board();
        }

        let next_state = self.game_state.tick(text_input, app_context);

        match next_state {
            Some(StateSort::Game(_)) => Some(StateSort::Tutorial(Tutorial::default())),
            Some(StateSort::SkirmishMenu(_)) => Some(StateSort::MainMenu(MainMenu::default())),
            _ => next_state,
        }
    }
}

impl Default for Tutorial {
    fn default() -> Self {
        let level: Level = "hg18a09m4g0m81g00c4068035g14r0v008".into();

        Tutorial {
            game_state: Game::new(LobbySettings {
                lobby_sort: LobbySort::LocalAI,
                loadout_method: LoadoutMethod::Prefab(level),
                seed: window().performance().unwrap().now() as u64,
                can_stalemate: false,
            }),
            tutorial_stage: TutorialStage::Movement,
        }
    }
}
