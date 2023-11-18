use std::f64::consts::PI;

use nalgebra::Vector2;
use rapier2d::dynamics::RigidBody;
use shared::BugData;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    app::{ContentElement, LabelTrim, Particle, ParticleSort, Pointer, UIElement, BOARD_SCALE},
    tuple_as,
};

pub fn draw_image(
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
        atlas,
        sx,
        sy,
        sw,
        sh,
        dx.floor(),
        dy.floor(),
        sw,
        sh,
    )?;

    Ok(())
}

pub fn draw_image_centered(
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
        atlas,
        sx,
        sy,
        sw,
        sh,
        (dx - sw / 2.0).floor(),
        (dy - sh / 2.0).floor(),
        sw,
        sh,
    )?;

    Ok(())
}

const LOCAL_SCALE: f64 = 16.0;

pub fn local_to_screen(local: &Vector2<f32>) -> (f64, f64) {
    (
        local.x as f64 * LOCAL_SCALE + 384.0 / 2.0,
        local.y as f64 * LOCAL_SCALE + 256.0 / 2.0,
    )
}
pub fn screen_to_local(screen: (f64, f64)) -> (f64, f64) {
    (
        (screen.0 - 384.0 / 2.0) / LOCAL_SCALE,
        (screen.1 - 256.0 / 2.0) / LOCAL_SCALE,
    )
}

pub fn draw_bug(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    (rigid_body, bug_data): (&RigidBody, &BugData),
    index: usize,
    frame: usize,
) -> Result<(), JsValue> {
    let (dx, dy) = local_to_screen(rigid_body.translation());
    let direction = rigid_body.linvel().x.signum() as f64;

    context.save();
    context.translate(dx.round(), dy.round())?;
    context.scale(direction, 1.0)?;
    draw_bugdata(context, atlas, bug_data, index, frame)?;
    context.restore();

    Ok(())
}

pub fn draw_bugdata(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    bug_data: &BugData,
    index: usize,
    frame: usize,
) -> Result<(), JsValue> {
    draw_image_centered(
        context,
        atlas,
        16.0 * ((index % 2) as f64),
        16.0 * (((frame / (6 + (index % 3)) + (index % 3)) % 2) as f64),
        16.0,
        16.0,
        0.0,
        0.0,
    )?;

    Ok(())
}

pub fn draw_bug_impulse(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    (rigid_body, bug_data): (&RigidBody, &BugData),
    index: usize,
    frame: usize,
) -> Result<(), JsValue> {
    let (ox, oy) = local_to_screen(rigid_body.translation());
    let (dx, dy) = local_to_screen(&(rigid_body.translation() + bug_data.impulse_intent()));

    let length = (dy - oy).hypot(dx - ox);

    if length > 16.0 {
        let (nx, ny) = ((dx - ox) / length, (dy - oy) / length);
        const STEP: f64 = 6.0;
        let increments = (length / STEP) as usize;

        for t in 0..increments {
            let (qx, qy) = (nx * STEP * t as f64, ny * STEP * t as f64);
            draw_image_centered(context, atlas, 40.0, 184.0, 8.0, 8.0, ox + qx, oy + qy)?;
        }

        draw_image_centered(context, atlas, 32.0, 184.0, 8.0, 8.0, dx, dy)?;
    }

    Ok(())
}

// pub struct Sprite {
//     sx: u16,
//     sy: u16,
//     sw: u16,
//     sh: u16,
//     dx: f64,
//     dy: f64,
// }

// pub fn draw_sprite(
//     context: &CanvasRenderingContext2d,
//     atlas: &HtmlCanvasElement,
//     sprite: &Sprite,
// ) -> Result<(), JsValue> {
//     draw_image(
//         context,
//         atlas,
//         sprite.sx as f64,
//         sprite.sy as f64,
//         sprite.sw as f64,
//         sprite.sh as f64,
//         sprite.dx - sprite.sw as f64 * 0.5,
//         sprite.dy - sprite.sh as f64 * 0.5,
//     )
// }

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

        draw_image(
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

pub fn draw_particle(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    particle: &Particle,
    frame: usize,
) -> Result<(), JsValue> {
    let board_scale = tuple_as!(BOARD_SCALE, f64);

    context.save();
    context.translate(
        ((particle.position.0 + 0.5) * board_scale.0).floor(),
        ((particle.position.1 + 0.5) * board_scale.1).floor(),
    )?;

    let spin = particle.lifetime;
    let cycle = frame
        + (particle.position.0 * 16.0) as usize
        + (particle.position.1 * 16.0) as usize
        + spin;

    context.rotate((spin / 5) as f64 * std::f64::consts::PI / 2.0)?;
    // context.rotate(frame as f64 * 0.1)?;
    draw_image(
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

pub fn draw_label(
    context: &CanvasRenderingContext2d,
    atlas: &HtmlCanvasElement,
    position: (i32, i32),
    size: (i32, i32),
    color: &str,
    content: &ContentElement,
    pointer: &Pointer,
    frame: usize,
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
        LabelTrim::Round => (232.0, 248.0),
        LabelTrim::Glorious => (240.0, 248.0),
        LabelTrim::Return => (248.0, 248.0),
    };

    draw_image(
        context,
        atlas,
        trim_position.0,
        trim_position.1,
        4.0,
        4.0,
        -size.0 as f64 / 2.0,
        -size.1 as f64 / 2.0,
    )?;
    draw_image(
        context,
        atlas,
        trim_position.0 + 4.0,
        trim_position.1,
        4.0,
        4.0,
        size.0 as f64 / 2.0 - 4.0,
        -size.1 as f64 / 2.0,
    )?;
    draw_image(
        context,
        atlas,
        trim_position.0,
        trim_position.1 + 4.0,
        4.0,
        4.0,
        -size.0 as f64 / 2.0,
        size.1 as f64 / 2.0 - 4.0,
    )?;
    draw_image(
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
