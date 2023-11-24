use itertools::Itertools;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::draw::draw_particle;

#[derive(Clone, Default)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    last_tick_at: usize,
    last_index: usize,
}

impl ParticleSystem {
    pub fn tick_and_draw(
        &mut self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        frame: usize,
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

    pub fn spawn<F>(&mut self, count: usize, emitter: F)
    where
        F: Fn(usize) -> Particle,
    {
        self.particles.append(
            &mut (0..count)
                .into_iter()
                .map(|i| {
                    let mut particle = emitter(i);

                    particle.index = self.last_index;
                    self.last_index += 1;

                    particle
                })
                .collect_vec(),
        );
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

#[derive(Copy, Clone)]
pub struct Particle {
    pub index: usize,
    pub position: (f64, f64),
    velocity: (f64, f64),
    pub lifetime: usize,
    pub sort: ParticleSort,
}

impl Particle {
    pub fn new(
        position: (f64, f64),
        velocity: (f64, f64),
        lifetime: usize,
        sort: ParticleSort,
    ) -> Particle {
        Particle {
            index: 0,
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
