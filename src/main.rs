use std::{ops::Sub, sync::Arc};

use eframe::egui::{self, pos2, vec2, Pos2, Vec2};

mod geom;
use geom::{Circle, Pos, RotCircle};

mod gfx;
use gfx::GraphicsState;
use itertools::Itertools;

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

fn gen_circles(N: usize) -> Vec<RotCircle> {
    let ang = std::f64::consts::TAU / N as f64;
    let angs = (0..N).map(|n| n as f64 * ang).collect_vec();
    angs.iter()
        .map(|ang| RotCircle::new(Pos::new(-ang.cos() / 2., ang.sin() / 2.), 0.5, 5, false))
        .collect_vec()
}
fn gen_colors(i: usize) -> egui::Color32 {
    if let Some(col) = colorous::SET1.get(i) {
        return egui::Color32::from_rgb(col.r, col.g, col.b);
    };
    return egui::Color32::GOLD;
}

struct App {
    gfx: Arc<GraphicsState>,
    circles: Vec<RotCircle>,
    scale: f32,
    depth: u32,
    grip_rad: f32,
    grip_cuts: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gfx: Arc::new(GraphicsState::new(
                cc.wgpu_render_state.as_ref().expect("No render state"),
            )),
            circles: gen_circles(2),
            scale: 0.5,
            depth: 500,
            grip_rad: 0.025,
            grip_cuts: false,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut clear = false;
        egui::TopBottomPanel::bottom("Sliders").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if ui.button("+").clicked() {
                        self.circles = gen_circles(self.circles.len() + 1);
                        clear = true;
                    }
                    if ui.button("-").clicked() {
                        if self.circles.len() > 1 {
                            self.circles = gen_circles(self.circles.len() - 1);
                            clear = true;
                        }
                    }
                    ui.checkbox(&mut self.grip_cuts, "All Cuts");
                });
                ui.vertical(|ui| {
                    clear |= ui
                        .add(egui::Slider::new(&mut self.scale, (0.1)..=(100.)).logarithmic(true))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.depth, 100..=100000).logarithmic(true))
                        .changed();
                    clear |= ui
                        .add(egui::Slider::new(&mut self.grip_rad, (0.)..=(0.1)))
                        .changed();
                });

                for circle in &mut self.circles {
                    ui.vertical(|ui| {
                        clear |= ui
                            .add(
                                egui::Slider::new(&mut circle.rad, (0.)..=(2.))
                                    .clamp_to_range(false),
                            )
                            .changed();
                        clear |= ui
                            .add(egui::Slider::new(&mut circle.step, 2..=16).clamp_to_range(false))
                            .changed();
                        clear |= ui.checkbox(&mut circle.inverted, "Invert").clicked()
                    });
                }
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

            let mut circles = vec![];
            let mut grips = vec![];
            if r.is_pointer_button_down_on() {
                if let Some(mpos) = ctx.pointer_latest_pos() {
                    //let mpos = itrans(mpos);
                    let seed = itrans(mpos);
                    let seed = Pos::new(seed.x as f64, -seed.y as f64);

                    // Fill regions
                    if ui.input(|i| i.pointer.primary_down()) {
                        let point_max_rad = |point: Pos| {
                            self.circles
                                .iter()
                                .map(|c| (c.cen.dist(&point) - c.rad).abs())
                                .reduce(f64::min)
                                .expect("Oops, no circles")
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
                            for circle in &self.circles {
                                if circle.contains(&points[i].0) {
                                    let new = circle.rotate_point(points[i].0);
                                    if pointset.insert(&new.into(), ()).is_none() {
                                        points.push((new, i));
                                        max_rad = max_rad.min(point_max_rad(new));
                                    }
                                }
                            }
                        }
                        for point in &points {
                            let col = if points.len() as u32 > self.depth {
                                [0.5, 0.5, 0.5, 1.]
                            } else {
                                let col = colorous::SINEBOW.eval_rational(
                                    (calculate_hash(&(points.len() + 1))) as u32 as usize,
                                    u32::MAX as usize + 1,
                                );
                                [
                                    col.r as f32 / 255.,
                                    col.g as f32 / 255.,
                                    col.b as f32 / 255.,
                                    1.,
                                ]
                            };
                            circles.push(Circle {
                                centre: point.0.into(),
                                radius: max_rad as f32,
                                col,
                            })
                        }
                    }

                    // Calculate grips
                    if ui.input(|i| i.pointer.secondary_down()) {
                        let mut points = vec![];
                        let mut pointset: hypermath::collections::ApproxHashMap<RotCircle, ()> =
                            hypermath::collections::approx_hashmap::ApproxHashMap::new();

                        let base_grips = GripSet {
                            circles: self.circles.clone(),
                        };
                        let mut gripsets = vec![base_grips.clone()];
                        let mut gripsetset: hypermath::collections::ApproxHashMap<GripSet, ()> =
                            hypermath::collections::approx_hashmap::ApproxHashMap::new();
                        gripsetset.insert(&base_grips, ());
                        for (i, g) in self
                            .circles
                            .iter()
                            .enumerate()
                            .filter(|(_, c)| c.contains(&seed))
                        {
                            pointset.insert(&g, ());
                            points.push((g.cen.clone(), i));
                        }

                        for i in 0..self.depth as usize {
                            if i >= gripsets.len() {
                                break;
                            }
                            for j in 0..gripsets[i].circles.len() {
                                if gripsets[i].circles[j].contains(&seed) {
                                    let new_set = gripsets[i].rotate_by(j);
                                    if gripsetset.insert(&new_set, ()).is_none() {
                                        for (i, grip) in new_set.circles.iter().enumerate() {
                                            if grip.contains(&seed) {
                                                if pointset.insert(&grip, ()).is_none() {
                                                    points.push((grip.cen, i));
                                                }
                                            }
                                        }
                                        gripsets.push(new_set);
                                    }
                                }
                            }
                        }
                        grips = points;
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
            for (i, circle) in self.circles.iter().enumerate() {
                let cen: Pos2 = circle.cen.into();
                painter.circle_stroke(
                    trans(pos2(cen.x, -cen.y)),
                    circle.rad as f32 * unit,
                    (4., gen_colors(i)),
                );
            }
            for (grip, i) in grips {
                // let cen: Pos2 = grip.cen.into();
                let cen: Pos2 = grip.into();
                let cen = trans(pos2(cen.x, -cen.y));
                painter.circle(
                    cen,
                    self.grip_rad * unit,
                    gen_colors(i),
                    (2., egui::Color32::LIGHT_GRAY),
                );
                if self.grip_cuts {
                    painter.circle_stroke(
                        cen,
                        self.circles[i].rad as f32 * unit,
                        (2., egui::Color32::LIGHT_GRAY),
                    );
                }
            }
            ctx.request_repaint();
        });
    }
}

fn transform(pos: Pos2, transform: (f32, Vec2)) -> Pos2 {
    (pos.to_vec2() * transform.0).to_pos2() + transform.1
}
fn inv_transform(pos: Pos2, transform: (f32, Vec2)) -> Pos2 {
    ((pos - transform.1).to_vec2() / transform.0).to_pos2()
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    let mut s = std::hash::DefaultHasher::new();
    t.hash(&mut s);
    std::hash::Hasher::finish(&s)
}

#[derive(Debug, Clone)]
struct GripSet {
    circles: Vec<RotCircle>,
}
impl GripSet {
    fn rotate_by(&self, index: usize) -> Self {
        let circles = self
            .circles
            .iter()
            .map(|circle| self.circles[index].rotate_circle(circle))
            .collect_vec();
        Self { circles }
    }
}
impl hypermath::collections::approx_hashmap::ApproxHashMapKey for GripSet {
    type Hash = Vec<<Pos as hypermath::collections::approx_hashmap::ApproxHashMapKey>::Hash>;

    fn approx_hash(
        &self,
        mut float_hash_fn: impl FnMut(
            hypermath::prelude::Float,
        ) -> hypermath::collections::approx_hashmap::FloatHash,
    ) -> Self::Hash {
        self.circles
            .iter()
            .map(|circle| circle.cen.approx_hash(&mut float_hash_fn))
            .collect()
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
