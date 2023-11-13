use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::draw::draw_particle;

#[derive(Clone, Default)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    last_tick_at: u64,
}

impl ParticleSystem {
    pub fn tick_and_draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        frame: u64,
    ) -> Result<(), JsValue> {
        if self.last_tick_at != frame {
            self.last_tick_at = frame;

            for particle in self.particles.iter_mut() {
                particle.tick();
            }

            self.particles.retain(|particle| particle.is_alive());
        }

        for particle in &self.particles {
            draw_particle(context, atlas, particle, frame)?;
        }

        Ok(())
    }

    pub fn add(&mut self, particle: Particle) {
        self.particles.push(particle)
    }
}

#[derive(Copy, Clone)]
pub enum ParticleSort {
    Missile,
    Diagonals,
    RedWin,
    BlueWin,
    Shield,
    Beam,
}
impl ParticleSort {
    pub(crate) fn for_powerup(powerup: &shared::PowerUp) -> Option<ParticleSort> {
        match powerup {
            shared::PowerUp::Shield => Some(Self::Shield),
            shared::PowerUp::Beam => Some(Self::Beam),
            shared::PowerUp::Diagonal => Some(Self::Diagonals),
            shared::PowerUp::Boulder(_) => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Particle {
    pub position: (f64, f64),
    velocity: (f64, f64),
    pub lifetime: u64,
    pub sort: ParticleSort,
}

impl Particle {
    pub fn new(
        position: (f64, f64),
        velocity: (f64, f64),
        lifetime: u64,
        sort: ParticleSort,
    ) -> Particle {
        Particle {
            position,
            velocity,
            lifetime,
            sort,
        }
    }

    pub fn tick(&mut self) {
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;
        self.velocity.0 -= self.velocity.0 * 0.1;
        self.velocity.1 -= self.velocity.1 * 0.1;
        self.lifetime = self.lifetime.saturating_sub(1);
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 1
    }
}
