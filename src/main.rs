use std::sync::Arc;

use bitvec::prelude::*;
use eframe::egui::{self, pos2, vec2, Pos2, Vec2};

mod geom;
use geom::{Circle, Curvature, GraphicsCircle, Pos, RotCircle};

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

fn gen_circles(N: usize, distance: f64, curvature: Curvature) -> Vec<RotCircle> {
    let ang = std::f64::consts::TAU / N as f64;
    let angs = (0..N).map(|n| n as f64 * ang).collect_vec();
    let distance = match curvature {
        Curvature::Spherical => (distance / 4.).tan(),
        Curvature::Euclidean => distance / 2.,
        Curvature::Hyperbolic => (distance / 4.).tanh(),
    };
    angs.iter()
        .map(|ang| {
            RotCircle::new(
                distance * Pos::new(-ang.cos(), ang.sin()),
                // Pos::new(0., 0.),
                0.5,
                5,
                curvature,
                false,
            )
        })
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
    autofill: bool,
    index: usize,
    pixel_mask: BitBox,
    curvature: Curvature,
    circle_distance: f64,
    circle_count: usize,
    /// Whether drawing parameters have changed
    reset: bool,
    regenerate: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            gfx: Arc::new(GraphicsState::new(
                cc.wgpu_render_state.as_ref().expect("No render state"),
            )),
            circles: vec![],
            scale: 0.5,
            depth: 500,
            grip_rad: 0.05,
            grip_cuts: false,
            autofill: false,
            index: 0,
            pixel_mask: BitVec::EMPTY.into_boxed_bitslice(),
            curvature: Curvature::Euclidean,
            circle_distance: 1.,
            circle_count: 2,
            reset: true,
            regenerate: true,
        }
    }

    fn expand_seed(&self, seed: Pos, circles: &mut Vec<GraphicsCircle>) {
        let point_max_rad = |point: Pos| {
            self.circles
                .iter()
                .map(|c| (c.circle.cen.dist_in_space(&point, self.curvature) - c.circle.rad).abs())
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
            max_rad = max_rad.min(point.0.dist_to_inf(self.curvature));
            let (cen, rad) =
                Circle::new(point.0, max_rad, self.curvature).euclidean_centre_radius();
            circles.push(GraphicsCircle {
                centre: cen.into(),
                radius: rad as f32,
                col,
            });
        }
    }

    fn expand_grips(&self, seed: Pos, grips: &mut Vec<(Pos, usize)>) {
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
            points.push((g.circle.cen.clone(), i));
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
                                    points.push((grip.circle.cen, i));
                                }
                            }
                        }
                        gripsets.push(new_set);
                    }
                }
            }
        }
        *grips = points;
    }

    fn is_pixel_filled(&self, x: usize, y: usize, width: usize) -> bool {
        self.pixel_mask[x + y * width]
    }

    fn fill_pixel_circle(
        &mut self,
        circle: &GraphicsCircle,
        width: usize,
        dpi: f32,
        geom_to_egui: impl Fn(Pos) -> Pos2,
        unit: f32,
    ) {
        let &GraphicsCircle {
            centre: [x, y],
            radius: r,
            ..
        } = circle;
        let Pos2 { x, y } = geom_to_egui(Pos::new(x as f64, y as f64));
        let r = r * unit;
        let r = r * dpi;
        let x = x * dpi;
        let y = y * dpi;
        let circle_top = ((y + r).floor() as usize).min(self.pixel_mask.len() / width - 1);
        let circle_bottom = ((y - r).ceil() as usize).max(0);

        for row in circle_bottom..=circle_top {
            let row_height = row.abs_diff(y as usize);
            let row_width = ((r * r) - (row_height * row_height) as f32).sqrt().floor() as usize;
            let row_centre = x.floor() as isize;
            let row_start = (row_centre - row_width as isize).clamp(0, width as isize) as usize;
            let row_end = (row_centre + row_width as isize).clamp(0, width as isize) as usize;
            self.pixel_mask[(row_start + row * width)..(row_end + row * width)].fill(true);
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("Sliders").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if ui.button("+").clicked() {
                        self.circle_count += 1;
                        self.regenerate = true;
                    }
                    if ui.button("-").clicked() {
                        if self.circle_count > 1 {
                            self.circle_count -= 1;
                            self.regenerate = true;
                        }
                    }
                    ui.checkbox(&mut self.grip_cuts, "All Cuts");
                    ui.checkbox(&mut self.autofill, "Autofill");
                    if ui.button("Reset").clicked() {
                        self.regenerate = true;
                    };
                    if egui::ComboBox::from_label("Curvature")
                        .selected_text(match self.curvature {
                            Curvature::Spherical => "Spherical",
                            Curvature::Euclidean => "Euclidean",
                            Curvature::Hyperbolic => "Hyperbolic",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.curvature,
                                Curvature::Euclidean,
                                "Euclidean",
                            );
                            ui.selectable_value(
                                &mut self.curvature,
                                Curvature::Spherical,
                                "Spherical",
                            );
                            ui.selectable_value(
                                &mut self.curvature,
                                Curvature::Hyperbolic,
                                "Hyperbolic",
                            );
                        })
                        .response
                        .changed()
                    {
                        self.regenerate = true;
                    }
                });
                ui.vertical(|ui| {
                    self.reset |= ui
                        .add(egui::Slider::new(&mut self.scale, (0.1)..=(100.)).logarithmic(true))
                        .changed();
                    self.reset |= ui
                        .add(egui::Slider::new(&mut self.depth, 100..=100000).logarithmic(true))
                        .changed();
                    self.reset |= ui
                        .add(egui::Slider::new(&mut self.grip_rad, (0.)..=(0.1)))
                        .changed();
                    self.regenerate |= ui
                        .add(egui::Slider::new(&mut self.circle_distance, (0.)..=(5.)))
                        .changed();
                });

                for circle in &mut self.circles {
                    ui.vertical(|ui| {
                        self.reset |= ui
                            .add(
                                egui::Slider::new(&mut circle.circle.rad, (0.)..=(2.))
                                    .clamp_to_range(false),
                            )
                            .changed();
                        self.reset |= ui
                            .add(egui::Slider::new(&mut circle.step, 2..=16).clamp_to_range(false))
                            .changed();
                        self.reset |= ui.checkbox(&mut circle.inverted, "Invert").clicked()
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

            let geom_to_egui = |pos: Pos| pos2(pos.x as f32, -pos.y as f32) * unit + cen.to_vec2();
            let egui_to_geom = |pos: Pos2| {
                let pos = (pos - cen.to_vec2()) / unit;
                Pos {
                    x: pos.x as f64,
                    y: -pos.y as f64,
                }
            };

            if self.regenerate {
                self.circles = gen_circles(self.circle_count, self.circle_distance, self.curvature);
                self.reset = true;
            }
            if self.reset {
                self.index = 0;
                self.pixel_mask = bitbox![0; (target_size[0]*target_size[1]) as usize];
            }

            let mut circles = vec![];
            let mut grips = vec![];
            if r.is_pointer_button_down_on() {
                if let Some(mpos) = ctx.pointer_latest_pos() {
                    //let mpos = itrans(mpos);
                    let seed = egui_to_geom(mpos);
                    // let seed = Pos::new(seed.x as f64, -seed.y as f64);

                    // Fill regions
                    if ui.input(|i| i.pointer.primary_down()) {
                        self.expand_seed(seed, &mut circles);
                    }

                    // Calculate grips
                    if ui.input(|i| i.pointer.secondary_down()) {
                        self.expand_grips(seed, &mut grips);
                    }
                }
            }

            if self.autofill {
                if self.pixel_mask.len() != (target_size[0] * target_size[1]) as usize {
                    self.pixel_mask = bitbox![0; (target_size[0]*target_size[1]) as usize];
                }
                let time = std::time::Instant::now();
                // let mut rng = thread_rng();
                while time.elapsed() < std::time::Duration::from_millis(5) {
                    if !self.is_pixel_filled(
                        self.index % target_size[0] as usize,
                        self.index / target_size[0] as usize,
                        target_size[0] as usize,
                    ) {
                        let seed = egui_to_geom(pos2(
                            (self.index % target_size[0] as usize) as f32,
                            (self.index / target_size[0] as usize) as f32,
                        ));
                        self.expand_seed(seed, &mut circles);
                    }
                    self.index =
                        (self.index + 1000000007) % (target_size[0] * target_size[1]) as usize;
                    // self.index = (self.index + 1) % (target_size[0] * target_size[1]) as usize
                }
            }

            for circle in &circles {
                let dpi = ctx.pixels_per_point();
                self.fill_pixel_circle(circle, target_size[0] as usize, dpi, geom_to_egui, unit);
            }

            let out_circles = if circles.len() > 0 {
                circles.iter().map(|c| c.get_instance(scale)).collect()
            } else {
                vec![GraphicsCircle {
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
                    clear: self.reset,
                },
            ));
            if self.curvature == Curvature::Hyperbolic {
                painter.circle_stroke(cen, unit, (1., egui::Color32::LIGHT_GRAY));
            }
            for (i, circle) in self.circles.iter().enumerate() {
                let (cen, rad) = circle.euclidean_centre_radius();
                painter.circle_stroke(geom_to_egui(cen), rad as f32 * unit, (4., gen_colors(i)));
            }
            for (grip, i) in grips {
                let circle = Circle::new(grip, self.grip_rad as f64, self.curvature);
                let (cen, rad) = circle.euclidean_centre_radius();
                let cen = geom_to_egui(cen);
                painter.circle(
                    cen,
                    rad as f32 * unit,
                    gen_colors(i),
                    (2., egui::Color32::LIGHT_GRAY),
                );
                if self.grip_cuts {
                    let circle = Circle::new(grip, self.circles[i].circle.rad, self.curvature);
                    let (cen, rad) = circle.euclidean_centre_radius();
                    let cen = geom_to_egui(cen);
                    painter.circle_stroke(cen, rad as f32 * unit, (2., egui::Color32::LIGHT_GRAY));
                }
            }
            // pixel mask debug visual
            // for i in (0..self.pixel_mask.len()).step_by(100) {
            //     let dpi = ctx.pixels_per_point();
            //     let (x, y) = (i % target_size[0] as usize, i / target_size[0] as usize);
            //     if self.is_pixel_filled(x, y, target_size[0] as usize) {
            //         painter.circle_filled(
            //             pos2(x as f32 / dpi, y as f32 / dpi),
            //             2.,
            //             egui::Color32::GOLD,
            //         );
            //     }
            // }
            ctx.request_repaint();
            self.reset = false;
            self.regenerate = false;
        });
    }
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
            .map(|circle| circle.circle.cen.approx_hash(&mut float_hash_fn))
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
