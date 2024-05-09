use std::{ops::Sub, sync::Arc};

use eframe::egui::{self, pos2, vec2, Pos2, Vec2};

mod gfx;
use gfx::{CircleInstance, GraphicsState};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        follow_system_theme: false,
        ..Default::default()
    };
    eframe::run_native(
        "Pentagon Gardening Simulator",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

struct App {
    gfx: Arc<GraphicsState>,
    a_rad: f64,
    b_rad: f64,
    a_step: u32,
    b_step: u32,
    scale: f32,
    depth: u32,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gfx: Arc::new(GraphicsState::new(
                cc.wgpu_render_state.as_ref().expect("No render state"),
            )),
            a_rad: 0.5,
            b_rad: 0.5,
            a_step: 5,
            b_step: 5,
            scale: 1.,
            depth: 100,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut clear = false;
        egui::TopBottomPanel::bottom("Sliders").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    clear |= ui
                        .add(egui::Slider::new(&mut self.a_rad, (0.)..=(2.)))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.b_rad, (0.)..=(2.)))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.scale, (0.)..=(10.)))
                        .changed();
                });
                ui.vertical(|ui| {
                    clear |= ui
                        .add(egui::Slider::new(&mut self.a_step, 2..=10))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.b_step, 2..=10))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.depth, 1..=10000))
                        .changed();
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let (cen, size) = (rect.center(), rect.size());
            let unit = size.min_elem() * self.scale / 2.;

            // Allocate space in the UI.
            let (egui_rect, target_size) =
                rounded_pixel_rect(ui, ui.available_rect_before_wrap(), 1);
            let r = ui.allocate_rect(egui_rect, egui::Sense::click_and_drag());

            let scale = egui_rect.size() / egui_rect.height();
            let scale = [scale.x.recip() * self.scale, scale.y.recip() * self.scale];

            // let trans_tup = (unit, cen.to_vec2() - (unit / 2. * vec2(1., 0.)));
            let trans_tup = (unit, cen.to_vec2());
            let trans = |pos| transform(pos, trans_tup);
            let itrans = |pos| inv_transform(pos, trans_tup);

            // let a_cen = cen - (unit / 2. * vec2(1., 0.));
            // let b_cen = cen + (unit / 2. * vec2(1., 0.));
            let a_cen = Pos::new(-0.5, 0.);
            let b_cen = Pos::new(0.5, 0.);
            let a_cen_screen = trans(a_cen.into());
            let b_cen_screen = trans(b_cen.into());
            let mut circles = vec![];
            if r.is_pointer_button_down_on() {
                if let Some(mpos) = ctx.pointer_latest_pos() {
                    //let mpos = itrans(mpos);
                    let seed: Pos = itrans(mpos).into();
                    let seed = Pos::new(seed.x, -seed.y);
                    let point_max_rad = |point: Pos| {
                        f64::min(
                            (a_cen.dist(point) - self.a_rad).abs(),
                            (b_cen.dist(point) - self.b_rad).abs(),
                        )
                    };
                    let mut max_rad = point_max_rad(seed);
                    let mut points = vec![(seed, 0)];
                    let mut pointset: hypermath::collections::ApproxHashMap<Pos, ()> =
                        hypermath::collections::approx_hashmap::ApproxHashMap::new();
                    pointset.insert(&seed, ());
                    for i in 0..self.depth as usize {
                        if i >= points.len() {
                            break;
                        }
                        if in_circle(points[i].0, a_cen, self.a_rad) {
                            let new = points[i].0.rotate(a_cen, self.a_step);
                            if pointset.insert(&new.into(), ()).is_none() {
                                points.push((new, i));
                                max_rad = max_rad.min(point_max_rad(new));
                            }
                        }
                        if in_circle(points[i].0, b_cen, self.b_rad) {
                            let new = points[i].0.rotate(b_cen, self.b_step);
                            if pointset.insert(&new.into(), ()).is_none() {
                                points.push((new, i));
                                max_rad = max_rad.min(point_max_rad(new));
                            }
                        }
                    }
                    for point in &points {
                        let col = colorous::SINEBOW.eval_rational(points.len() % 20, 21);
                        let col = [
                            col.r as f32 / 255.,
                            col.g as f32 / 255.,
                            col.b as f32 / 255.,
                            1.,
                        ];
                        circles.push(Circle {
                            centre: point.0.into(),
                            radius: max_rad as f32,
                            col,
                        })
                    }
                }
            }

            let out_circles = if circles.len() > 0 {
                circles.iter().map(|c| c.get_instance(scale)).collect()
            } else {
                vec![Circle {
                    centre: [f32::NAN; 2],
                    radius: f32::NAN,
                    col: [f32::NAN; 4],
                }
                .get_instance(scale)]
            };
            let painter = ui.painter_at(egui_rect);
            painter.add(eframe::egui_wgpu::Callback::new_paint_callback(
                egui_rect,
                gfx::RenderResources {
                    gfx: Arc::clone(&self.gfx),
                    circles: out_circles,
                    texture_size: eframe::wgpu::Extent3d {
                        width: target_size[0],
                        height: target_size[1],
                        depth_or_array_layers: 1,
                    },
                    clear,
                },
            ));
            ctx.request_repaint();
            ui.painter().circle_stroke(
                a_cen_screen,
                self.a_rad as f32 * unit,
                (4., egui::Color32::RED),
            );
            ui.painter().circle_stroke(
                b_cen_screen,
                self.b_rad as f32 * unit,
                (4., egui::Color32::BLUE),
            );
        });
    }
}

fn in_circle(pos: Pos, cen: Pos, rad: f64) -> bool {
    cen.dist_sq(pos) < rad * rad
}

fn transform(pos: Pos2, transform: (f32, Vec2)) -> Pos2 {
    (pos.to_vec2() * transform.0).to_pos2() + transform.1
}
fn inv_transform(pos: Pos2, transform: (f32, Vec2)) -> Pos2 {
    ((pos - transform.1).to_vec2() / transform.0).to_pos2()
}

#[derive(Debug, Default, Copy, Clone)]
struct Pos {
    x: f64,
    y: f64,
}
impl From<Pos2> for Pos {
    fn from(value: Pos2) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}
impl From<Pos> for Pos2 {
    fn from(value: Pos) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}
impl From<Pos> for [f32; 2] {
    fn from(value: Pos) -> Self {
        [value.x as f32, value.y as f32]
    }
}
impl Pos {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn rotate(self, cen: Pos, step: u32) -> Self {
        let theta = std::f64::consts::TAU / step as f64;
        let x = self.x - cen.x;
        let y = self.y - cen.y;
        let x2 = theta.cos() * x + theta.sin() * y;
        let y2 = theta.cos() * y - theta.sin() * x;
        let x = x2 + cen.x;
        let y = y2 + cen.y;

        Self { x, y }
    }
    fn dist_sq(self, other: Pos) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
    fn dist(self, other: Pos) -> f64 {
        self.dist_sq(other).sqrt()
    }
}

impl hypermath::collections::approx_hashmap::ApproxHashMapKey for Pos {
    type Hash = [hypermath::collections::approx_hashmap::FloatHash; 2];

    fn approx_hash(
        &self,
        float_hash_fn: impl FnMut(
            hypermath::prelude::Float,
        ) -> hypermath::collections::approx_hashmap::FloatHash,
    ) -> Self::Hash {
        [self.x as f64, self.y as f64].map(float_hash_fn)
    }
}

/// Rounds an egui rectangle to the nearest pixel boundary and returns the
/// rounded egui rectangle, along with its width & height in pixels.
pub fn rounded_pixel_rect(
    ui: &egui::Ui,
    rect: egui::Rect,
    downscale_rate: u32,
) -> (egui::Rect, [u32; 2]) {
    let dpi = ui.ctx().pixels_per_point();

    // Round rectangle to pixel boundary for crisp image.
    let mut pixels_rect = rect;
    pixels_rect.set_left((dpi * pixels_rect.left()).ceil());
    pixels_rect.set_bottom((dpi * pixels_rect.bottom()).floor());
    pixels_rect.set_right((dpi * pixels_rect.right()).floor());
    pixels_rect.set_top((dpi * pixels_rect.top()).ceil());

    // Convert back from pixel coordinates to egui coordinates.
    let mut egui_rect = pixels_rect;
    *egui_rect.left_mut() /= dpi;
    *egui_rect.bottom_mut() /= dpi;
    *egui_rect.right_mut() /= dpi;
    *egui_rect.top_mut() /= dpi;

    let pixel_size = [
        pixels_rect.width() as u32 / downscale_rate,
        pixels_rect.height() as u32 / downscale_rate,
    ];
    (egui_rect, pixel_size)
}

struct Circle {
    centre: [f32; 2],
    radius: f32,
    col: [f32; 4],
}
impl Circle {
    fn get_instance(&self, scale: [f32; 2]) -> CircleInstance {
        CircleInstance {
            col: self.col,
            centre: [self.centre[0] * scale[0], self.centre[1] * scale[1]],
            scale: [scale[0] * self.radius, scale[1] * self.radius],
        }
    }
}
