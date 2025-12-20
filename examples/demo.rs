use eframe::App;
use egui::{Color32, Vec2, ViewportBuilder};
use egui_snow::Snow;

/// Enum for readable layer selection
#[derive(PartialEq, Clone, Copy)]
enum SnowLayer {
    Background,
    Foreground,
    Top,
}

struct SnowDemo {
    // Settings
    active: bool,
    color: Color32,
    density: usize,

    // Ranges are split into min/max for easier UI slider manipulation
    size_min: f32,
    size_max: f32,

    speed_min: f32,
    speed_max: f32,

    wind_x: f32,
    wind_y: f32,

    // Layout / Rendering
    layer: SnowLayer,
    draw_in_window: bool,
}

impl Default for SnowDemo {
    fn default() -> Self {
        Self {
            active: true,
            color: Color32::from_white_alpha(200),
            density: 400,
            size_min: 0.3,
            size_max: 1.0,
            speed_min: 40.0,
            speed_max: 120.0,
            wind_x: 0.0,
            wind_y: 0.0,
            layer: SnowLayer::Foreground,
            draw_in_window: false,
        }
    }
}

impl App for SnowDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("❄ egui-snow ❄");
            ui.label("High performance particle system for egui.");
            ui.label("This application demonstrates the capabilities of the snowfall effect widget.");
            ui.add_space(20.0);
            ui.label("Try moving the 'Settings' window around to see how snow particles interact with layers.");
            ui.label("Content inside the CentralPanel is usually on the Background layer.");

            ui.add_space(20.0);
            if ui.button("Toggle Dark/Light Mode").clicked() {
                let visuals = if ctx.style().visuals.dark_mode {
                    egui::Visuals::light()
                } else {
                    egui::Visuals::dark()
                };
                ctx.set_visuals(visuals);
            }
        });

        // Render the Settings Window
        let mut window_rect = ctx.content_rect();
        let window_response = egui::Window::new("Settings")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("General");
                ui.checkbox(&mut self.active, "Effect Active");
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_srgba(&mut self.color);
                });

                ui.separator();
                ui.heading("Particles");

                ui.label("Density (Count):");
                ui.add(egui::Slider::new(&mut self.density, 0..=20000).logarithmic(true));

                ui.add_space(5.0);
                ui.label("Size Range (px):");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.size_min)
                            .speed(0.1)
                            .range(0.1..=20.0)
                            .prefix("Min: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.size_max)
                            .speed(0.1)
                            .range(0.1..=20.0)
                            .prefix("Max: "),
                    );
                });
                // Ensure min does not exceed max
                if self.size_min > self.size_max {
                    self.size_min = self.size_max;
                }

                ui.separator();
                ui.heading("Physics");

                ui.label("Fall Speed Range (px/s):");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.speed_min)
                            .speed(1.0)
                            .range(0.0..=500.0)
                            .prefix("Min: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.speed_max)
                            .speed(1.0)
                            .range(0.0..=500.0)
                            .prefix("Max: "),
                    );
                });
                if self.speed_min > self.speed_max {
                    self.speed_min = self.speed_max;
                }

                ui.label("Wind Vector:");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.wind_x, -200.0..=200.0).text("Horizontal"));
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.wind_y, -100.0..=100.0).text("Vertical"));
                });

                ui.separator();
                ui.heading("Rendering");
                ui.horizontal(|ui| {
                    ui.label("Layer:");
                    ui.selectable_value(&mut self.layer, SnowLayer::Background, "Background")
                        .on_hover_text("Behind everything (Order::Background)");
                    ui.selectable_value(&mut self.layer, SnowLayer::Foreground, "Foreground")
                        .on_hover_text("In front of windows (Order::Foreground)");
                    ui.selectable_value(&mut self.layer, SnowLayer::Top, "Top")
                        .on_hover_text(
                            "On top of everything including debug/tooltips (Order::Debug)",
                        );
                });

                ui.checkbox(&mut self.draw_in_window, "Draw in this window only");

                ui.add_space(10.0);
                if ui.button("Reset Defaults").clicked() {
                    *self = Self::default();
                }
            });

        // Capture window rect if we want to confine snow to it
        if self.draw_in_window {
            if let Some(resp) = window_response {
                window_rect = resp.response.rect;
            }
        }

        // Render Snow
        if self.active {
            let mut snow = Snow::new("demo_snow")
                .color(self.color)
                .density(self.density)
                .size(self.size_min..=self.size_max)
                .speed(self.speed_min..=self.speed_max)
                .wind(Vec2::new(self.wind_x, self.wind_y));

            // Apply Layer settings
            snow = match self.layer {
                SnowLayer::Background => snow.on_background(),
                SnowLayer::Foreground => snow.on_foreground(),
                SnowLayer::Top => snow.on_top(),
            };

            snow = snow.area(window_rect);
            snow.show(ctx);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 220.0])
            .with_title("egui-snow Interactive Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "egui-snow demo",
        native_options,
        Box::new(|_| Ok(Box::new(SnowDemo::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn get_canvas_element() -> Option<web_sys::HtmlCanvasElement> {
    use eframe::wasm_bindgen::JsCast;

    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id("egui_snow_canvas")?;
    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Redirect `log` message to `console.log`
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    let web_options = eframe::WebOptions::default();
    let canvas = get_canvas_element().expect("Failed to find canvas with id 'egui_snow_canvas'");

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_| Ok(Box::new(SnowDemo::default()))),
            )
            .await
            .expect("failed to start eframe");
    });
}
