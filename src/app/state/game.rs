use std::{cell::RefCell, rc::Rc};

use shared::{
    Board, BoardStyle, GameResult, LoadoutMethod, Lobby, LobbyError, LobbyID, LobbySettings,
    LobbySort, Mage, Mages, Message, Position, PowerUp, Team, Turn, TurnLeaf,
};
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};

use super::{ArenaMenu, Editor, SkirmishMenu, State};
use crate::{
    app::{
        Alignment, App, AppContext, ButtonElement, ClipId, ConfirmButtonElement, Interface,
        LabelTheme, LabelTrim, Particle, ParticleSort, ParticleSystem, Pointer, StateSort,
        ToggleButtonElement, UIElement, UIEvent, BOARD_SCALE,
    },
    draw::{
        draw_board, draw_crosshair, draw_mage, draw_mana, draw_powerup, draw_sprite,
        rotation_from_position,
    },
    net::{
        create_new_lobby, fetch, request_state, request_turns_since, send_message, send_ready,
        send_rematch, MessagePool,
    },
    tuple_as, window,
};

const BUTTON_REMATCH: usize = 1;
const BUTTON_LEAVE: usize = 2;
const BUTTON_MENU: usize = 10;
const BUTTON_UNDO: usize = 20;

pub struct Game {
    interface: Interface,
    button_menu: ToggleButtonElement,
    button_undo: ButtonElement,
    lobby: Lobby,
    last_move_frame: u64,
    last_hits: Vec<Position>,
    active_mage: Option<usize>,
    particle_system: ParticleSystem,
    message_pool: Rc<RefCell<MessagePool>>,
    message_closure: Closure<dyn FnMut(JsValue)>,
    board_dirty: bool,
    shake_frame: (u64, usize),
    recorded_result: bool,
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

        if let shared::LobbySort::Online(0) = lobby_settings.lobby_sort {
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
            button_menu,
            button_undo,
            lobby: Lobby::new(lobby_settings),
            last_move_frame: 0,
            last_hits: Vec::new(),
            active_mage: None,
            particle_system: ParticleSystem::default(),
            message_pool,
            message_closure,
            board_dirty: true,
            recorded_result: false,
            shake_frame: (0, 0),
        }
    }

    pub fn particle_system(&mut self) -> &mut ParticleSystem {
        &mut self.particle_system
    }

    pub fn lobby(&self) -> &Lobby {
        &self.lobby
    }

    pub fn lobby_id(&self) -> Result<LobbyID, LobbyError> {
        self.lobby
            .settings
            .lobby_sort
            .lobby_id()
            .ok_or(LobbyError("lobby has no ID".to_string()))
    }

    /// Converts a canvas location to a board [`Position`].
    pub fn location_as_position(
        &self,
        location: (i32, i32),
        offset: (i32, i32),
        scale: (i32, i32),
    ) -> Option<Position> {
        let position = Position(
            ((location.0 - offset.0) / scale.0) as i8,
            ((location.1 - offset.1) / scale.1) as i8,
        );

        let (board_width, board_height) = self.lobby.game.board_size();

        if (location.0 - offset.0) >= 0
            && position.0 < board_width as i8
            && (location.1 - offset.1) >= 0
            && position.1 < board_height as i8
        {
            Some(position)
        } else {
            None
        }
    }

    pub fn live_occupied(&self, position: Position) -> bool {
        self.lobby.game.live_occupied(&position)
    }

    fn is_mage_active(&self, mage: &Mage) -> bool {
        match self.active_mage {
            Some(active_mage) => active_mage == mage.index,
            None => false,
        }
    }

    fn get_active_mage(&self) -> Option<&Mage> {
        if let Some(index) = self.active_mage {
            if let Some(mage) = self.lobby.game.get_mage(index) {
                return Some(mage);
            }
        }

        None
    }

    pub fn select_mage_at(&mut self, session_id: Option<&String>, selected_tile: &Position) {
        if self.lobby.is_active_player(session_id) {
            self.active_mage = if let Some(occupant) = self.lobby.game.live_occupant(selected_tile)
            {
                if occupant.team == self.lobby.game.turn_for() {
                    Some(occupant.index)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn deselect_mage(&mut self) {
        self.active_mage = None;
    }

    pub fn play_mage_selection_sound(&self, app_context: &AppContext) {
        match self.active_mage {
            Some(_) => app_context.audio_system.play_clip(ClipId::MageSelect),
            None => app_context.audio_system.play_clip(ClipId::MageDeselect),
        }
    }

    pub fn take_best_turn_quick(&mut self) {
        let turn = self
            .lobby
            .game
            .best_turn(3, window().performance().unwrap().now().to_bits());

        if let Some(TurnLeaf(turn, _)) = turn {
            self.message_pool.borrow_mut().push(Message::Move(turn));
        }
    }

    pub fn take_best_turn(&mut self) {
        let turn = self
            .lobby
            .game
            .best_turn_auto(window().performance().unwrap().now().to_bits());

        if let Some(TurnLeaf(turn, _)) = turn {
            self.message_pool.borrow_mut().push(Message::Move(turn));
        }
    }

    pub fn board_offset(&self) -> (i32, i32) {
        let board_size = self.lobby().game.board_size();

        (
            ((8 - board_size.0) as i32 * BOARD_SCALE.0) / 2,
            ((8 - board_size.1) as i32 * BOARD_SCALE.1) / 2,
        )
    }

    pub fn frames_since_last_move(&self, frame: u64) -> u64 {
        frame.saturating_sub(self.last_move_frame)
    }

    pub fn draw_game(
        &mut self,
        context: &CanvasRenderingContext2d,
        _interface_context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        frame: u64,
        pointer: &Pointer,
    ) -> Result<(), JsValue> {
        let board_scale = tuple_as!(BOARD_SCALE, f64);
        let board_offset = tuple_as!(self.board_offset(), f64);

        // let (board_width, board_height) = self.lobby.game.board_size();

        if self.board_dirty {
            self.board_dirty = false;
            draw_board(atlas, 256.0, 0.0, self.lobby.game.board(), 8, 8).unwrap();
            draw_board(
                atlas,
                384.0,
                256.0,
                &Board::with_style(4, 4, BoardStyle::Teleport).unwrap(),
                4,
                4,
            )
            .unwrap();
        }

        // DRAW background layer (board + UI block)

        // DRAW board

        context.save();

        if frame.saturating_sub(self.shake_frame.0) < 20 {
            let magnitude = self.shake_frame.1.saturating_sub(1) as f64 * 0.65;
            context.translate(
                ((frame as f64 * 0.15 * self.shake_frame.1 as f64).sin() * (magnitude)).round(),
                ((frame as f64 * 0.3 * self.shake_frame.1 as f64).sin() * (magnitude)).round(),
            )?;
        }

        {
            context.save();

            draw_sprite(context, atlas, 256.0, 0.0, 256.0, 256.0, 0.0, 0.0)?;

            context.translate(board_offset.0, board_offset.1)?;

            // DRAW particles

            self.particle_system()
                .tick_and_draw(context, atlas, frame)?;

            // DRAW powerups
            for (position, powerup) in self.lobby.game.powerups() {
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

            {
                let board_offset = self.board_offset();

                // DRAW markers
                context.save();

                for mage in self.lobby.game.iter_mages() {
                    if mage.is_alive() && mage.is_defensive() {
                        for (_, position) in self.lobby.game.targets(mage, mage.position) {
                            match mage.team {
                                Team::Red => {
                                    draw_sprite(
                                        context,
                                        atlas,
                                        32.0,
                                        16.0,
                                        16.0,
                                        16.0,
                                        position.0 as f64 * 32.0 + 8.0,
                                        position.1 as f64 * 32.0 + 8.0,
                                    )?;
                                    // draw_crosshair(context, atlas, &position, (32.0, 16.0), 1)?;
                                }
                                Team::Blue => {
                                    draw_sprite(
                                        context,
                                        atlas,
                                        48.0,
                                        16.0,
                                        16.0,
                                        16.0,
                                        position.0 as f64 * 32.0 + 8.0,
                                        position.1 as f64 * 32.0 + 8.0,
                                    )?;
                                    // draw_crosshair(context, atlas, &position, (48.0, 16.0), 1)?;
                                }
                            }
                        }
                    }
                }

                if let Some(mage) = self.get_active_mage() {
                    let available_moves = self.lobby.game.available_moves(mage);
                    for (position, dir, _) in &available_moves {
                        let ri = rotation_from_position(*dir);
                        let is_diagonal = ri % 2 == 1;
                        context.save();
                        context.translate(
                            (position.0 as f64 + 0.5) * board_scale.0,
                            (position.1 as f64 + 0.5) * board_scale.1,
                        )?;
                        context.rotate((ri / 2) as f64 * std::f64::consts::PI / 2.0)?;
                        let bop = (frame / 10 % 3) as f64;
                        context.translate(bop - 4.0, if is_diagonal { bop - 4.0 } else { 0.0 })?;
                        draw_sprite(
                            context,
                            atlas,
                            if is_diagonal { 16.0 } else { 0.0 },
                            32.0,
                            16.0,
                            16.0,
                            -8.0,
                            -8.0,
                        )?;
                        context.restore();
                    }

                    if let Some(selected_tile) = self.lobby.game.location_as_position(
                        pointer.location,
                        board_offset,
                        BOARD_SCALE,
                    ) {
                        if available_moves
                            .iter()
                            .any(|(position, _, _)| position == &selected_tile)
                        {
                            for (enemy_occupied, position) in
                                &self.lobby.game.targets(mage, selected_tile)
                            {
                                if *enemy_occupied {
                                    draw_sprite(
                                        context,
                                        atlas,
                                        32.0,
                                        256.0,
                                        32.0,
                                        32.0,
                                        position.0 as f64 * 32.0,
                                        position.1 as f64 * 32.0,
                                    )?;
                                    draw_crosshair(context, atlas, position, (64.0, 32.0), frame)?;
                                } else {
                                    draw_sprite(
                                        context,
                                        atlas,
                                        64.0,
                                        256.0,
                                        32.0,
                                        32.0,
                                        position.0 as f64 * 32.0,
                                        position.1 as f64 * 32.0,
                                    )?;
                                    // draw_crosshair(context, atlas, position, (48.0, 32.0), 0)?;
                                }
                            }
                        }
                    }
                }

                if let Some(selected_tile) = self.lobby.game.location_as_position(
                    pointer.location,
                    board_offset,
                    BOARD_SCALE,
                ) {
                    if let Some(occupant) = self.lobby.game.live_occupant(&selected_tile) {
                        if let Some(selected_tile) = self.lobby.game.location_as_position(
                            pointer.location,
                            board_offset,
                            BOARD_SCALE,
                        ) {
                            for (_, position) in &self.lobby.game.targets(occupant, selected_tile) {
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
                    }
                    draw_crosshair(context, atlas, &selected_tile, (32.0, 32.0), frame)?;
                }

                context.restore();
            }

            {
                let game_started = self.lobby.all_ready() | self.lobby.is_local();

                self.lobby.game.sort_mages();

                // DRAW mages
                for mage in self.lobby.game.iter_mages() {
                    context.save();

                    context.translate(
                        16.0 + mage.position.0 as f64 * board_scale.0,
                        16.0 + mage.position.1 as f64 * board_scale.1,
                    )?;

                    context.save();

                    if self.frames_since_last_move(frame) < 32
                        && self.frames_since_last_move(frame) % 16 < 8
                        && self.last_hits.contains(&mage.position)
                    {
                        context.set_global_composite_operation("lighter")?;
                    }

                    draw_mage(
                        context,
                        atlas,
                        mage,
                        frame,
                        self.lobby.game.turn_for(),
                        game_started,
                        self.lobby.game.result(),
                    )?;

                    context.restore();

                    if mage.is_alive() {
                        if mage.has_diagonals() {
                            for _ in 0..(frame / 3 % 2) {
                                let d = js_sys::Math::random() * -std::f64::consts::PI * 0.9;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.05;
                                self.particle_system.add(Particle::new(
                                    (
                                        mage.position.0 as f64 + d.cos() * 0.4,
                                        mage.position.1 as f64 - 0.15 + d.sin() * 0.4,
                                    ),
                                    (d.cos() * v, d.sin() * v),
                                    (js_sys::Math::random() * 30.0) as u64,
                                    ParticleSort::Diagonals,
                                ));
                            }
                        } else if mage.is_defensive() {
                            for _ in 0..(frame / 3 % 2) {
                                let d = js_sys::Math::random() * -std::f64::consts::PI * 0.9;
                                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.05;
                                self.particle_system.add(Particle::new(
                                    (
                                        mage.position.0 as f64 + d.cos() * 0.4,
                                        mage.position.1 as f64 - 0.15 + d.sin() * 0.4,
                                    ),
                                    (d.cos() * v, d.sin() * v),
                                    (js_sys::Math::random() * 30.0) as u64,
                                    ParticleSort::Shield,
                                ));
                            }
                        }
                    }

                    if self.is_mage_active(mage) {
                        draw_sprite(
                            context,
                            atlas,
                            72.0,
                            0.0,
                            8.0,
                            5.0,
                            -3.0,
                            -17.0 - (frame / 6 % 6) as f64,
                        )?;
                    }

                    context.restore();
                }

                // DRAW mana bars for all mages
                for mage in self.lobby.game.iter_mages() {
                    context.save();

                    context.translate(
                        16.0 + mage.position.0 as f64 * board_scale.0,
                        16.0 + mage.position.1 as f64 * board_scale.1,
                    )?;
                    draw_mana(context, atlas, mage)?;

                    context.restore();
                }
            }

            context.translate(-board_offset.0, -board_offset.1)?;

            if !self.lobby.all_ready() && !self.lobby.is_local() {
                draw_sprite(context, atlas, 384.0, 256.0, 128.0, 128.0, 64.0, 64.0)?;

                let mut lid = self.lobby_id().unwrap_or(0);

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
                        x as f64 * board_scale.0 + 72.0,
                        y as f64 * board_scale.1 + 72.0,
                    )?;
                }
            }

            context.restore();
        }

        context.restore();

        context.save();

        context.translate(6.0 - self.board_offset().0 as f64 + 128.0, -40.0 + 128.0)?;

        if self.lobby.game.can_stalemate() {
            let (_, gap) = self.lobby.game.stalemate();
            for i in 1..9 {
                if gap > i {
                    if i % 2 == 1 {
                        draw_sprite(
                            context,
                            atlas,
                            128.0,
                            8.0,
                            8.0,
                            8.0,
                            128.0,
                            0.0 + (i * 8) as f64,
                        )?;
                    } else {
                        draw_sprite(
                            context,
                            atlas,
                            136.0,
                            0.0,
                            8.0,
                            16.0,
                            128.0,
                            0.0 + (i * 8 - 8) as f64,
                        )?;
                    }
                } else {
                    draw_sprite(
                        context,
                        atlas,
                        128.0,
                        0.0,
                        8.0,
                        8.0,
                        128.0,
                        0.0 + (i * 8) as f64,
                    )?;
                }
            }
        }

        context.restore();

        Ok(())
    }

    pub fn tick_game(&mut self, frame: u64, app_context: &AppContext) {
        if self.last_move_frame == 0 {
            self.last_move_frame = frame;
        }

        let session_id = &app_context.session_id;

        let mut target_positions = Vec::new();

        let all_ready = self.lobby.all_ready();

        let mut message_pool = self.message_pool.borrow_mut();

        if let Some(lobby_id) = self.lobby.settings.lobby_sort.lobby_id() {
            if message_pool.available(frame) {
                if all_ready {
                    if self.is_interface_active() {
                        let _ = fetch(&request_state(lobby_id)).then(&self.message_closure);
                    } else {
                        let _ = fetch(&request_turns_since(lobby_id, self.lobby.game.turns()))
                            .then(&self.message_closure);
                    }
                } else if self.lobby.settings.lobby_sort != LobbySort::Online(0) {
                    let _ = fetch(&request_state(lobby_id)).then(&self.message_closure);
                }

                message_pool.block(frame);
            }
        }

        if self.lobby.has_ai()
            && self.lobby.game.turn_for() == Team::Blue
            && frame - self.last_move_frame > 45
            && !self.lobby.finished()
        {
            let turn = self
                .lobby
                .game
                .best_turn_auto(window().performance().unwrap().now().to_bits());

            if let Some(TurnLeaf(turn, _)) = turn {
                message_pool.messages.append(&mut vec![Message::Move(turn)]);
            }
        }

        for message in &message_pool.messages {
            match message {
                Message::Moves(turns) => {
                    for Turn(from, to) in turns {
                        if let Some(move_targets) = self.lobby.game.take_move(*from, *to) {
                            target_positions.append(&mut move_targets.clone());

                            self.last_move_frame = frame;
                            self.last_hits = move_targets;
                        }
                    }
                }
                Message::Move(Turn(from, to)) => {
                    let to_powerup = self.lobby.game.powerups().get(to).cloned();

                    if let Some(move_targets) = self.lobby.game.take_move(*from, *to) {
                        app_context.audio_system.play_clip(ClipId::MageMove);

                        target_positions.append(&mut move_targets.clone());

                        if let Some(moved_mage) = self.lobby.game.occupant(to) {
                            if let Some(to_powerup) = to_powerup {
                                app_context.audio_system.play_powerup(to_powerup);

                                if to_powerup == PowerUp::Beam {
                                    let particle_sort = match moved_mage.team {
                                        Team::Red => ParticleSort::RedWin,
                                        Team::Blue => ParticleSort::BlueWin,
                                    };

                                    for x in 0..self.lobby.game.board_size().0 {
                                        for _ in 0..40 {
                                            let d = js_sys::Math::random() * std::f64::consts::TAU;
                                            let v = (js_sys::Math::random()
                                                + js_sys::Math::random())
                                                * 0.1;

                                            self.particle_system.add(Particle::new(
                                                (x as f64, to.1 as f64),
                                                (d.cos() * v * 2.0, d.sin() * v * 0.5),
                                                (js_sys::Math::random() * 50.0) as u64,
                                                particle_sort,
                                            ));
                                        }
                                    }

                                    for y in 0..self.lobby.game.board_size().1 {
                                        for _ in 0..40 {
                                            let d = js_sys::Math::random() * std::f64::consts::TAU;
                                            let v = (js_sys::Math::random()
                                                + js_sys::Math::random())
                                                * 0.1;

                                            self.particle_system.add(Particle::new(
                                                (to.0 as f64, y as f64),
                                                (d.cos() * v * 0.5, d.sin() * v * 2.0),
                                                (js_sys::Math::random() * 50.0) as u64,
                                                particle_sort,
                                            ));
                                        }
                                    }
                                }
                            }
                        }

                        self.last_move_frame = frame;
                        self.last_hits = move_targets;
                    }
                }
                Message::Lobby(lobby) => {
                    self.lobby = *lobby.clone();
                    self.board_dirty = true;

                    if let Ok(lobby_id) = self.lobby_id() {
                        if !lobby.all_ready() {
                            send_ready(lobby_id, session_id.clone().unwrap());
                        }
                    }
                }
                _ => (),
            }
        }

        message_pool.clear();

        for tile in &target_positions {
            for _ in 0..40 {
                let d = js_sys::Math::random() * std::f64::consts::TAU;
                let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;
                self.particle_system.add(Particle::new(
                    (tile.0 as f64, tile.1 as f64),
                    (d.cos() * v, d.sin() * v),
                    (js_sys::Math::random() * 50.0) as u64,
                    ParticleSort::Missile,
                ));
            }
        }

        if !target_positions.is_empty() {
            self.shake_frame = (frame, target_positions.len());

            app_context
                .audio_system
                .play_random_zap(target_positions.len());
        }
    }

    pub fn is_interface_active(&self) -> bool {
        self.button_menu.selected()
    }
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

        self.draw_game(context, interface_context, atlas, frame, pointer)?;

        {
            let interface_pointer =
                pointer.teleport(app_context.canvas_settings.inverse_interface_center());

            interface_context.save();
            interface_context.translate(
                (app_context.canvas_settings.interface_width / 2) as f64,
                (app_context.canvas_settings.interface_height / 2) as f64,
            )?;

            self.button_menu
                .draw(interface_context, atlas, &interface_pointer, frame)?;

            if self.lobby.is_local() {
                self.button_undo
                    .draw(interface_context, atlas, &interface_pointer, frame)?;
            }

            if self.is_interface_active() {
                self.interface
                    .draw(interface_context, atlas, &interface_pointer, frame)?;

                for player in self
                    .lobby
                    .players()
                    .values()
                    .filter(|player| player.rematch)
                {
                    let first_mage = self
                        .lobby
                        .game
                        .iter_mages()
                        .find(|mage| mage.team == player.team);

                    if let Some(first_mage) = first_mage {
                        interface_context.save();

                        match player.team {
                            Team::Red => {
                                interface_context.translate(-40.0, -8.0)?;
                            }
                            Team::Blue => {
                                interface_context.translate(40.0, -8.0)?;
                            }
                        }

                        draw_mage(
                            interface_context,
                            atlas,
                            first_mage,
                            frame,
                            player.team,
                            true,
                            None,
                        )?;

                        interface_context.restore();
                    }
                }
            }

            interface_context.restore();
        }

        // draw_text(
        //     interface_context,
        //     atlas,
        //     16.0,
        //     16.0,
        //     &format!("{:?}", self.lobby().game.evaluate()),
        // )?;

        Ok(())
    }

    fn tick(
        &mut self,
        _text_input: &HtmlInputElement,
        app_context: &AppContext,
    ) -> Option<StateSort> {
        let board_offset = self.board_offset();
        let frame = app_context.frame;
        let pointer = &app_context.pointer;
        let session_id = &app_context.session_id;

        let message_pool = self.message_pool.clone();

        if self.lobby.finished() && self.frames_since_last_move(frame) == 120 {
            self.button_menu.set_selected(true);
        }

        if self.lobby.finished() {
            if let Some(GameResult::Win(team)) = self.lobby.game.result() {
                // Did not record the result in the KV-store yet...
                if !self.recorded_result {
                    App::kv_set(
                        &self.lobby.game.prototype_code(),
                        match team {
                            Team::Red => "win",
                            Team::Blue => "loss",
                        },
                    );

                    self.recorded_result = true;
                }

                let board_size = self.lobby().game.board_size();

                let particle_sort = match team {
                    Team::Red => ParticleSort::RedWin,
                    Team::Blue => ParticleSort::BlueWin,
                };

                for _ in 0..(board_size.0 + board_size.1) / 5 {
                    let d = js_sys::Math::random() * std::f64::consts::TAU;
                    let v = (js_sys::Math::random() + js_sys::Math::random()) * 0.1;

                    self.particle_system.add(Particle::new(
                        (js_sys::Math::random() * board_size.0 as f64 - 0.5, -0.5),
                        (d.sin() * v * 0.5, -v),
                        (js_sys::Math::random() * 40.0) as u64,
                        particle_sort,
                    ));

                    self.particle_system().add(Particle::new(
                        (
                            js_sys::Math::random() * board_size.0 as f64 - 0.5,
                            board_size.1 as f64 - 0.5,
                        ),
                        (d.sin() * v * 0.5, v),
                        (js_sys::Math::random() * 40.0) as u64,
                        particle_sort,
                    ));

                    self.particle_system().add(Particle::new(
                        (-0.5, js_sys::Math::random() * board_size.1 as f64 - 0.5),
                        (-v, d.sin() * v * 0.5),
                        (js_sys::Math::random() * 40.0) as u64,
                        particle_sort,
                    ));

                    self.particle_system().add(Particle::new(
                        (
                            board_size.0 as f64 - 0.5,
                            js_sys::Math::random() * board_size.1 as f64 - 0.5,
                        ),
                        (v, d.sin() * v * 0.5),
                        (js_sys::Math::random() * 40.0) as u64,
                        particle_sort,
                    ));
                }
            }
        }

        let interface_pointer =
            pointer.teleport(app_context.canvas_settings.inverse_interface_center());

        self.button_menu.tick(&interface_pointer);

        if self.lobby.is_local() && self.button_undo.tick(&interface_pointer).is_some() {
            self.lobby.rewind(2);

            self.last_move_frame = frame;
            self.last_hits = Vec::new();

            self.button_menu.set_selected(false);
        }

        if self.is_interface_active() {
            if let Some(UIEvent::ButtonClick(value, clip_id)) =
                self.interface.tick(&interface_pointer)
            {
                app_context.audio_system.play_clip_option(clip_id);

                match value {
                    BUTTON_REMATCH => {
                        if self.lobby.is_local() {
                            return Some(StateSort::Game(Game::new(self.lobby.settings.clone())));
                        } else if let Ok(lobby_id) = self.lobby_id() {
                            let session_id = app_context.session_id.clone().unwrap();
                            let _ = send_rematch(lobby_id, session_id)
                                .unwrap()
                                .then(&self.message_closure);
                        }
                    }
                    BUTTON_LEAVE => match &self.lobby.settings {
                        LobbySettings {
                            loadout_method: LoadoutMethod::EditorPrefab(level),
                            ..
                        } => {
                            return Some(StateSort::Editor(Editor::new(level.clone())));
                        }
                        LobbySettings {
                            loadout_method: LoadoutMethod::Arena(_, position),
                            ..
                        } => {
                            return Some(StateSort::ArenaMenu(ArenaMenu::at_position(*position)));
                        }
                        _ => return Some(StateSort::SkirmishMenu(SkirmishMenu::default())),
                    },
                    _ => (),
                }
            }
        } else {
            if pointer.alt_clicked() {
                self.deselect_mage();
            }

            if pointer.clicked() {
                if let Some(selected_tile) = self.lobby.game.location_as_position(
                    pointer.location,
                    board_offset,
                    BOARD_SCALE,
                ) {
                    if let Some(active_mage) = self.get_active_mage() {
                        let from = active_mage.position;

                        if self.lobby.game.try_move(from, selected_tile) {
                            if !self.lobby.is_local() && session_id.is_some() {
                                send_message(
                                    self.lobby_id().unwrap(),
                                    session_id.clone().unwrap(),
                                    Message::Move(Turn(from, selected_tile)),
                                );
                            }

                            let mut message_pool = message_pool.borrow_mut();

                            message_pool
                                .messages
                                .push(Message::Move(Turn(from, selected_tile)));

                            self.active_mage = None;
                            self.last_move_frame = frame;
                        } else {
                            self.select_mage_at(session_id.as_ref(), &selected_tile);
                            self.play_mage_selection_sound(app_context);
                        }
                    } else {
                        self.select_mage_at(session_id.as_ref(), &selected_tile);
                        self.play_mage_selection_sound(app_context);
                    }
                }
            }
        }

        self.tick_game(frame, app_context);

        None
    }
}
