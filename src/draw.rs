use std::f64::consts::PI;

use shared::{Board, BoulderStyle, GameResult, Mage, Position, PowerUp, Team};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    app::{ContentElement, LabelTrim, Particle, ParticleSort, Pointer, UIElement, BOARD_SCALE},
    tuple_as,
};

pub fn rotation_from_position(position: Position) -> i8 {
    let (sx, sy) = (position.0.signum(), position.1.signum());

    match (sx, sy) {
        (1, 0) => 0,
        (1, 1) => 1,
        (0, 1) => 2,
        (-1, 1) => 3,
        (-1, 0) => 4,
        (-1, -1) => 5,
        (0, -1) => 6,
        (1, -1) => 7,
        _ => 0,
    }
}

pub fn draw_sprite(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    sx: f64,
    sy: f64,
    sw: f64,
    sh: f64,
    dx: f64,
    dy: f64,
) -> Result<(), JsValue> {
    context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        atlas, sx, sy, sw, sh, dx, dy, sw, sh,
    )?;

    Ok(())
}

fn kerning(char: char) -> (isize, isize) {
    match char {
        'i' => (-2, -2),
        'l' => (-2, -1),
        't' => (-2, -1),
        'f' => (0, -1),
        'a' => (-1, 0),
        'c' => (-1, -1),
        'o' => (-1, -1),
        'p' => (-1, 0),
        ' ' => (-2, -2),
        'I' => (-1, -2),
        _ => (0, 0),
    }
}

pub fn text_length(text: &str) -> isize {
    text.chars()
        .map(|char| {
            let kern = kerning(char);
            (kern.0 + kern.1) + 8
        })
        .sum()
}

pub fn draw_text(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    dx: f64,
    dy: f64,
    text: &str,
) -> Result<(), JsValue> {
    let mut kerning_acc: isize = 0;

    for (i, char) in text.chars().enumerate() {
        let kern = kerning(char);
        kerning_acc += kern.0;

        draw_sprite(
            context,
            atlas,
            ((char as u8 % 32) * 8) as f64,
            216.0 + ((char as u8 / 32) * 8) as f64,
            8.0,
            8.0,
            dx + (i * 8) as f64 + kerning_acc as f64,
            dy + 1.0,
        )
        .unwrap();

        kerning_acc += kern.1;
    }

    Ok(())
}

pub fn draw_text_centered(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    dx: f64,
    dy: f64,
    text: &str,
) -> Result<(), JsValue> {
    draw_text(
        context,
        atlas,
        dx + (-text_length(text) / 2) as f64,
        dy - 4.0,
        text,
    )
}

pub fn draw_crosshair(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    position: &Position,
    offset: (f64, f64),
    frame: u64,
) -> Result<(), JsValue> {
    let board_scale = tuple_as!(BOARD_SCALE, f64);

    draw_sprite(
        context,
        atlas,
        offset.0,
        offset.1,
        8.0,
        8.0,
        position.0 as f64 * board_scale.0 + (frame / 6 % 4) as f64,
        position.1 as f64 * board_scale.1 + (frame / 6 % 4) as f64,
    )?;

    draw_sprite(
        context,
        atlas,
        offset.0 + 8.0,
        offset.1,
        8.0,
        8.0,
        position.0 as f64 * board_scale.0 - (frame / 6 % 4) as f64 + board_scale.0 - 8.0,
        position.1 as f64 * board_scale.1 + (frame / 6 % 4) as f64,
    )?;

    draw_sprite(
        context,
        atlas,
        offset.0,
        offset.1 + 8.0,
        8.0,
        8.0,
        position.0 as f64 * board_scale.0 + (frame / 6 % 4) as f64,
        position.1 as f64 * board_scale.1 - (frame / 6 % 4) as f64 + board_scale.1 - 8.0,
    )?;

    draw_sprite(
        context,
        atlas,
        offset.0 + 8.0,
        offset.1 + 8.0,
        8.0,
        8.0,
        position.0 as f64 * board_scale.0 - (frame / 6 % 4) as f64 + board_scale.0 - 8.0,
        position.1 as f64 * board_scale.1 - (frame / 6 % 4) as f64 + board_scale.1 - 8.0,
    )?;

    Ok(())
}

pub fn draw_mage(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    mage: &Mage,
    frame: u64,
    team: Team,
    game_started: bool,
    game_result: Option<GameResult>,
) -> Result<(), JsValue> {
    let bounce = (if mage.is_alive() && (mage.team == team && game_started || game_result.is_some())
    {
        -((frame as i64 / 6 + mage.index as i64 / 2) % 4 - 2).abs()
    } else {
        0
    }) as f64;

    let sleeping_offset = if mage.is_alive() && game_started {
        (0.0, 40.0)
    } else {
        (80.0, 32.0)
    };

    context.save();

    if mage.is_alive() {
        draw_sprite(context, atlas, 0.0, 208.0, 32.0, 16.0, -16.0, -4.0)?;

        if let Some(GameResult::Win(team)) = game_result {
            if team == mage.team {
                context.translate(
                    0.0,
                    ((frame as i64 % 80 - 40).max(0) - 20).abs() as f64 - 20.0,
                )?;
                context.rotate(
                    ((frame as i64 % 80 - 35).max(0) / 5) as f64 * std::f64::consts::PI / 2.0,
                )?;
            }
        }
    } else {
        context.translate(0.0, 4.0)?;

        draw_sprite(context, atlas, 32.0, 208.0, 32.0, 16.0, -16.0, -4.0)?;
    }

    let sprite_x = match mage.sort {
        shared::MageSort::Diamond => 0.0,
        shared::MageSort::Spike => 32.0,
        shared::MageSort::Knight => 64.0,
        shared::MageSort::Cross => 96.0,
        shared::MageSort::Plus => 128.0,
    };

    match mage.team {
        Team::Red => context
            .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                atlas,
                sprite_x,
                64.0 + sleeping_offset.0,
                32.0,
                sleeping_offset.1,
                -19.0,
                -28.0 + bounce + 40.0 - sleeping_offset.1,
                32.0,
                sleeping_offset.1,
            )?,
        Team::Blue => {
            context.scale(-1.0, 1.0)?;
            context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                atlas,
                sprite_x,
                64.0 + sleeping_offset.1 + sleeping_offset.0,
                32.0,
                sleeping_offset.1,
                -19.0,
                -28.0 + bounce + 40.0 - sleeping_offset.1,
                32.0,
                sleeping_offset.1,
            )?
        }
    }

    context.restore();

    Ok(())
}

pub fn draw_mana(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    mage: &Mage,
) -> Result<(), JsValue> {
    for i in 0..mage.mana.0 {
        draw_sprite(
            context,
            atlas,
            80.0,
            12.0,
            4.0,
            4.0,
            i as f64 * 6.0 - mage.mana.0 as f64 * 3.0 + 1.0,
            10.0,
        )?;
    }

    Ok(())
}

pub fn draw_spell_pattern(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    mage: &Mage,
) -> Result<(), JsValue> {
    for x in 0..5 {
        for y in 0..5 {
            if x == 2 && y == 2 {
                draw_sprite(
                    context,
                    atlas,
                    104.0,
                    16.0,
                    8.0,
                    8.0,
                    x as f64 * 8.0,
                    y as f64 * 8.0,
                )?;
            } else {
                draw_sprite(
                    context,
                    atlas,
                    96.0,
                    16.0,
                    8.0,
                    8.0,
                    x as f64 * 8.0,
                    y as f64 * 8.0,
                )?;
            }
        }
    }

    for position in &mage.spell.pattern {
        draw_sprite(
            context,
            atlas,
            96.0,
            24.0,
            8.0,
            8.0,
            position.0 as f64 * 8.0 + 16.0,
            position.1 as f64 * 8.0 + 16.0,
        )?;
    }

    Ok(())
}

pub fn draw_powerup(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    position: &Position,
    powerup: &PowerUp,
    frame: u64,
) -> Result<(), JsValue> {
    context.save();

    // draw_sprite(context, atlas, 0.0, 208.0, 32.0, 16.0, -16.0, -4.0)?;

    let sprite = match powerup {
        PowerUp::Shield => (32.0, 288.0, 32.0, 32.0),
        PowerUp::Beam => (96.0, 288.0, 32.0, 32.0),
        PowerUp::Diagonal => (64.0, 288.0, 32.0, 32.0),
        PowerUp::Boulder(BoulderStyle::Rock) => (0.0, 304.0, 32.0, 48.0),
        PowerUp::Boulder(BoulderStyle::Pedestal) => (0.0, 352.0, 32.0, 48.0),
        PowerUp::Boulder(BoulderStyle::Tentacle) => (32.0, 352.0, 32.0, 48.0),
    };

    let t = (frame as f64) / 10.0 + position.0 as f64 * 9.0 + position.1 as f64;

    let bounce = match powerup {
        PowerUp::Shield => {
            let q = ((t).sin(), -(t).sin().abs());
            ((q.0 * 3.0).round(), (q.1 * 3.0).round())
        }
        PowerUp::Beam => {
            let q = (
                (1.0 - (t).sin().abs()) * (t).cos().signum(),
                (1.0 - (t + PI / 2.0).sin().abs()) * (t + PI / 2.0).cos().signum().min(0.0),
            );
            ((q.0 * 3.0).round(), (q.1 * 5.0).round())
        }
        PowerUp::Diagonal => {
            let t = t + PI / 2.0;
            let q = (
                (1.0 - (t).sin().abs()) * (t).cos().signum(),
                (1.0 - (t + PI / 2.0).sin().abs()) * (t + PI / 2.0).cos().signum(),
            );
            ((q.0 * 4.0).round(), (q.1 * 4.0).round())
        }
        PowerUp::Boulder(_) => (0.0, -16.0),
    };

    draw_sprite(
        context,
        atlas,
        sprite.0,
        sprite.1,
        sprite.2,
        sprite.3,
        -16.0 + bounce.0,
        -16.0 + bounce.1,
    )?;

    context.restore();

    Ok(())
}

pub fn draw_particle(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    particle: &Particle,
    frame: u64,
) -> Result<(), JsValue> {
    let board_scale = tuple_as!(BOARD_SCALE, f64);

    context.save();
    context.translate(
        ((particle.position.0 + 0.5) * board_scale.0).floor(),
        ((particle.position.1 + 0.5) * board_scale.1).floor(),
    )?;

    let spin = particle.lifetime;
    let cycle =
        frame + (particle.position.0 * 16.0) as u64 + (particle.position.1 * 16.0) as u64 + spin;

    context.rotate((spin / 5) as f64 * std::f64::consts::PI / 2.0)?;
    // context.rotate(frame as f64 * 0.1)?;
    draw_sprite(
        context,
        atlas,
        {
            let t = cycle % 24;
            if t > 16 {
                16.0
            } else if t > 8 {
                8.0
            } else {
                0.0
            }
        } + {
            match particle.sort {
                ParticleSort::Missile => 0.0,
                ParticleSort::Diagonals => 24.0,
                ParticleSort::BlueWin => 48.0,
                ParticleSort::RedWin => 72.0,
                ParticleSort::Shield => 96.0,
                ParticleSort::Beam => 120.0,
            }
        },
        56.0,
        8.0,
        8.0,
        -4.0,
        -4.0,
    )?;
    context.restore();

    Ok(())
}

fn quadrant_to_xy(corner: u8) -> (u8, u8) {
    match corner {
        0 => (0, 0),
        1 => (1, 0),
        2 => (1, 1),
        _ => (0, 1),
    }
}

pub fn draw_tile(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    position: &Position,
    sprite_offset: (usize, usize),
) -> Result<(), JsValue> {
    let board_scale = tuple_as!(BOARD_SCALE, f64);
    let sprite_offset = tuple_as!(sprite_offset, f64);

    let offset = if (position.0 + position.1) % 2 == 0 {
        (224.0 + sprite_offset.0, 0.0 + sprite_offset.1)
    } else {
        (224.0 + sprite_offset.0, 32.0 + sprite_offset.1)
    };

    for corner in 0..4 {
        let (x, y) = quadrant_to_xy(corner);

        if (((position.0 + position.1 + position.0 % 3 + position.1 % 5 + corner as i8 * 2) as f64)
            .sin()
            + ((position.0 + position.1 + position.0 % 5 + position.1 % 3 + corner as i8 * 2)
                as f64)
                .cos())
        .abs()
            < 1.0
        {
            // draw_sprite(
            //     context,
            //     atlas,
            //     offset.0 + 8.0,
            //     offset.1 + 8.0,
            //     16.0,
            //     16.0,
            //     (position.0 as f64 + x as f64 / 2.0) * board_scale.0,
            //     (position.1 as f64 + y as f64 / 2.0) * board_scale.1,
            // )?;

            context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                atlas,
                offset.0 + 15.0,
                offset.1 + 15.0,
                2.0,
                2.0,
                (position.0 as f64 + x as f64 / 2.0) * board_scale.0,
                (position.1 as f64 + y as f64 / 2.0) * board_scale.1,
                16.0,
                16.0,
            )?;
        } else {
            context.save();

            draw_sprite(
                context,
                atlas,
                offset.0 + x as f64 * board_scale.0 / 2.0,
                offset.1 + y as f64 * board_scale.1 / 2.0,
                16.0,
                16.0,
                (position.0 as f64 + x as f64 / 2.0) * board_scale.0,
                (position.1 as f64 + y as f64 / 2.0) * board_scale.1,
            )?;

            context.restore();
        }
    }
    Ok(())
}

pub fn draw_board(
    atlas: &HtmlCanvasElement,
    dx: f64,
    dy: f64,
    board: &Board,
    clear_width: usize,
    clear_height: usize,
) -> Result<(), JsValue> {
    let board_scale = tuple_as!(BOARD_SCALE, f64);

    let atlas_context = atlas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    atlas_context.save();

    atlas_context.clear_rect(
        dx,
        dy,
        clear_width as f64 * board_scale.0,
        clear_height as f64 * board_scale.1,
    );

    atlas_context.translate(
        dx + ((clear_width - board.width) as f64 * board_scale.0) / 2.0,
        dy + ((clear_height - board.height) as f64 * board_scale.1) / 2.0,
    )?;

    let sprite_offset = board.style.sprite_offset();

    for x in 0..board.width {
        for y in 0..board.height {
            draw_tile(
                &atlas_context,
                atlas,
                &Position(x as i8, y as i8),
                sprite_offset,
            )?;
        }
    }

    let sprite_offset = tuple_as!(sprite_offset, f64);

    atlas_context.set_global_composite_operation("destination-out")?;

    for x in 0..board.width {
        for y in 0..board.height {
            let (edge_l, edge_r, edge_t, edge_b) =
                (x == 0, x == board.width - 1, y == 0, y == board.height - 1);

            let (dx, dy) = (x as f64 * board_scale.0, y as f64 * board_scale.1);

            atlas_context.save();
            atlas_context.translate(dx + 16.0, dy + 16.0)?;
            match (edge_l, edge_r, edge_t, edge_b) {
                (true, false, true, false) => {
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        0.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // TL corner
                (false, true, true, false) => {
                    atlas_context.rotate(std::f64::consts::PI * 0.5)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        0.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // TR corner
                (true, false, false, true) => {
                    atlas_context.rotate(std::f64::consts::PI * 1.5)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        0.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // BL corner
                (false, true, false, true) => {
                    atlas_context.rotate(std::f64::consts::PI)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        0.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // BR corner
                (true, false, false, false) => {
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        32.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // L edge
                (false, true, false, false) => {
                    atlas_context.rotate(std::f64::consts::PI)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        32.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // R edge
                (false, false, true, false) => {
                    atlas_context.rotate(std::f64::consts::PI * 0.5)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        32.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // T edge
                (false, false, false, true) => {
                    atlas_context.rotate(std::f64::consts::PI * 1.5)?;
                    draw_sprite(
                        &atlas_context,
                        atlas,
                        192.0 + sprite_offset.0,
                        32.0 + sprite_offset.1,
                        32.0,
                        32.0,
                        -16.0,
                        -16.0,
                    )?;
                } // B edge
                _ => (),
            }
            atlas_context.restore();
        }
    }

    atlas_context.restore();

    Ok(())
}

pub fn draw_label(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    position: (i32, i32),
    size: (i32, i32),
    color: &str,
    content: &ContentElement,
    pointer: &Pointer,
    frame: u64,
    trim: &LabelTrim,
    snip_content: bool,
) -> Result<(), JsValue> {
    context.save();

    context.translate(position.0 as f64, position.1 as f64)?;

    context.set_fill_style(&color.into());
    context.fill_rect(0.0, 0.0, size.0 as f64, size.1 as f64);

    context.translate(size.0 as f64 / 2.0, size.1 as f64 / 2.0)?;

    if snip_content {
        context.set_global_composite_operation("destination-out")?;
    }

    content.draw(context, atlas, pointer, frame)?;

    context.set_global_composite_operation("destination-out")?;

    let trim_position = match trim {
        LabelTrim::Round => (80.0, 0.0),
        LabelTrim::Glorious => (88.0, 0.0),
        LabelTrim::Return => (96.0, 0.0),
    };

    draw_sprite(
        context,
        atlas,
        trim_position.0,
        trim_position.1,
        4.0,
        4.0,
        -size.0 as f64 / 2.0,
        -size.1 as f64 / 2.0,
    )?;
    draw_sprite(
        context,
        atlas,
        trim_position.0 + 4.0,
        trim_position.1,
        4.0,
        4.0,
        size.0 as f64 / 2.0 - 4.0,
        -size.1 as f64 / 2.0,
    )?;
    draw_sprite(
        context,
        atlas,
        trim_position.0,
        trim_position.1 + 4.0,
        4.0,
        4.0,
        -size.0 as f64 / 2.0,
        size.1 as f64 / 2.0 - 4.0,
    )?;
    draw_sprite(
        context,
        atlas,
        trim_position.0 + 4.0,
        trim_position.1 + 4.0,
        4.0,
        4.0,
        size.0 as f64 / 2.0 - 4.0,
        size.1 as f64 / 2.0 - 4.0,
    )?;

    if *trim == LabelTrim::Glorious {
        context.fill_rect(
            -size.0 as f64 / 2.0,
            -size.1 as f64 / 2.0 + 3.0,
            3.0,
            size.1 as f64 - 6.0,
        );
        context.fill_rect(
            size.0 as f64 / 2.0 - 3.0,
            -size.1 as f64 / 2.0 + 3.0,
            3.0,
            size.1 as f64 - 6.0,
        );
    }

    context.restore();

    Ok(())
}
