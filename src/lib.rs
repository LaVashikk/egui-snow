use egui::{Color32, Context, Id, LayerId, Order, Pos2, Rect, Vec2};
use rand::Rng;
use std::ops::RangeInclusive;

#[derive(Default, Clone)]
struct SnowState {
    flakes: Vec<Snowflake>,
}

#[derive(Clone, Copy)]
struct Snowflake {
    /// Normalized coordinates (0.0..1.0)
    normalized_pos: Pos2,
    /// Fall speed (px/sec), unique for each particle
    fall_speed: f32,
    /// Random turbulence on X axis, unique for each particle
    turbulence: f32,
    /// Size of the snowflake
    size: f32,
    /// Oscillation phase
    phase: f32,
}

/// A configurable snow effect for egui.
///
/// `Snow` allows you to add a falling snow animation to any egui layer or area.
/// It maintains its state between frames using egui's temporary data storage.
#[derive(Debug)]
pub struct Snow {
    id: Id,
    color: Color32,
    density: usize,
    layer_order: Order,
    custom_layer: Option<LayerId>,
    custom_area: Option<Rect>,

    size_range: RangeInclusive<f32>,
    speed_range: RangeInclusive<f32>,
    wind: Vec2,
}

impl Snow {
    /// Creates a new snow effect with a unique ID.
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(id_source),
            color: Color32::WHITE,
            density: 50,
            layer_order: Order::Foreground,
            custom_layer: None,
            custom_area: None,
            // Default values
            size_range: 0.3..=1.5,
            speed_range: 40.0..=100.0,
            wind: Vec2::ZERO,
        }
    }

    /// Sets the color of the snowflakes.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// Sets the number of snowflakes.
    pub fn density(mut self, density: usize) -> Self {
        self.density = density;
        self
    }

    /// Sets the falling speed range (min..=max) in pixels per second.
    pub fn speed(mut self, range: RangeInclusive<f32>) -> Self {
        self.speed_range = range;
        self
    }

    /// Sets the snowflake size range (min..=max).
    pub fn size(mut self, range: RangeInclusive<f32>) -> Self {
        self.size_range = range;
        self
    }

    /// Sets the wind vector.
    pub fn wind(mut self, wind: impl Into<Vec2>) -> Self {
        self.wind = wind.into();
        self
    }

    /// Places the snow on the Debug layer (on top of everything).
    pub fn on_top(mut self) -> Self {
        self.layer_order = Order::Debug;
        self
    }

    /// Places the snow on the Foreground layer.
    pub fn on_foreground(mut self) -> Self {
        self.layer_order = Order::Foreground;
        self
    }

    /// Places the snow on the Background layer.
    pub fn on_background(mut self) -> Self {
        self.layer_order = Order::Background;
        self
    }

    /// Provides full control over the rendering layer.
    ///
    /// This overrides `on_top`, `on_foreground`, and `on_background`.
    pub fn layer(mut self, layer_id: LayerId) -> Self {
        self.custom_layer = Some(layer_id);
        self
    }

    /// Restricts the snow effect to a specific area.
    ///
    /// If not set, it defaults to `ctx.content_rect()`.
    pub fn area(mut self, area: Rect) -> Self {
        self.custom_area = Some(area);
        self
    }

    /// Updates and renders the snow effect.
    ///
    /// This should be called every frame. It automatically requests a repaint.
    pub fn show(self, ctx: &Context) {
        let screen_rect = self.custom_area.unwrap_or_else(|| ctx.content_rect());
        if screen_rect.width() <= 0.0 || screen_rect.height() <= 0.0 {
            return;
        }

        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        let time = ctx.input(|i| i.time);

        let mut snowflakes = ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with(self.id, SnowState::default);
            std::mem::take(&mut state.flakes)
        });

        // Spawn logic
        let mut rng = rand::rng();
        if snowflakes.len() < self.density {
            let needed = self.density - snowflakes.len();
            for _ in 0..needed {
                snowflakes.push(Snowflake::spawn(
                    &mut rng,
                    true,
                    &self.size_range,
                    &self.speed_range,
                ));
            }
        } else if snowflakes.len() > self.density {
            snowflakes.truncate(self.density);
        }

        let layer_id = self
            .custom_layer
            .unwrap_or_else(|| LayerId::new(self.layer_order, self.id));
        let painter = ctx.layer_painter(layer_id);

        // Cache values for alpha calculation
        let min_size = *self.size_range.start();
        let size_diff = (*self.size_range.end() - min_size).max(0.0001);

        // Update & Render logic
        for flake in &mut snowflakes {
            // --- Update ---
            // Y: Individual speed + vertical wind
            let pixel_dy = (flake.fall_speed + self.wind.y) * dt;

            // X: Individual turbulence + global wind + sine wave
            let sway = ((time as f32 + flake.phase).sin()) * 5.0;
            let pixel_dx = (flake.turbulence + self.wind.x + sway) * dt;

            flake.normalized_pos.x += pixel_dx / screen_rect.width();
            flake.normalized_pos.y += pixel_dy / screen_rect.height();

            // Respawn (Wrap Y)
            // If it flew down
            if flake.normalized_pos.y > 1.0 {
                *flake = Snowflake::spawn(&mut rng, false, &self.size_range, &self.speed_range);
            }
            // If it flew up (strong upward wind)
            else if flake.normalized_pos.y < -0.05 {
                flake.normalized_pos.y = 1.0;
                flake.normalized_pos.x = rng.random_range(0.0..1.0);
                flake.size = rng.random_range(self.size_range.clone());
                flake.fall_speed = rng.random_range(self.speed_range.clone());
            }

            // Wrap X (seamless horizontal transition)
            if flake.normalized_pos.x > 1.0 {
                flake.normalized_pos.x -= 1.0;
            } else if flake.normalized_pos.x < 0.0 {
                flake.normalized_pos.x += 1.0;
            }

            // --- Render ---
            let pixel_pos = Pos2::new(
                screen_rect.min.x + flake.normalized_pos.x * screen_rect.width(),
                screen_rect.min.y + flake.normalized_pos.y * screen_rect.height(),
            );

            // Calculate alpha based on size (simulating depth)
            // Smaller particles are more transparent
            let depth_factor = (flake.size - min_size) / size_diff;
            let alpha_mult = 0.4 + (0.6 * depth_factor); // Min. transparency 40%
            let final_color = self.color.gamma_multiply(alpha_mult);

            painter.circle_filled(pixel_pos, flake.size, final_color);
        }

        // Return snowflakes to state
        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with(self.id, SnowState::default);
            state.flakes = snowflakes;
        });

        ctx.request_repaint();
    }
}

impl Snowflake {
    fn spawn(
        rng: &mut impl rand::Rng,
        random_y: bool,
        size_range: &RangeInclusive<f32>,
        speed_range: &RangeInclusive<f32>,
    ) -> Self {
        Self {
            normalized_pos: Pos2::new(
                rng.random_range(0.0..1.0),
                if random_y {
                    rng.random_range(0.0..1.0)
                } else {
                    -0.05
                },
            ),
            fall_speed: rng.random_range(speed_range.clone()),
            turbulence: rng.random_range(-20.0..20.0),
            size: rng.random_range(size_range.clone()),
            phase: rng.random_range(0.0..std::f32::consts::PI * 2.0),
        }
    }
}
