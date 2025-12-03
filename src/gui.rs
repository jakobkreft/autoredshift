use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points, HLine};
use crate::config::Config;

pub struct App {
    config: Config,
    dragging_index: Option<usize>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        Self {
            config,
            dragging_index: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Autoredshift Configuration");
            ui.add_space(10.0);

            if ui.button("Save and Exit").clicked() {
                if let Err(e) = self.config.save() {
                    eprintln!("Failed to save config: {}", e);
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            
            ui.add_space(10.0);


            let plot = Plot::new("temperature_curve")
                .view_aspect(1.0)
                .label_formatter(|_name, value| {
                    format!("{:.2}h: {:.0}K", value.x, value.y)
                })
                .x_axis_label("Hour")
                .y_axis_label("Temperature (K)")
                .include_y(0.0)
                .include_y(10000.0)
                .allow_drag(false);

            plot.show(ui, |plot_ui| {
                // Draw reference lines
                plot_ui.hline(HLine::new(1000.0).color(egui::Color32::RED));
                plot_ui.hline(HLine::new(6500.0).color(egui::Color32::WHITE));
                
                if plot_ui.plot_bounds().max()[1] > 15000.0 {
                    plot_ui.hline(HLine::new(25000.0).color(egui::Color32::BLUE));
                }

                // Draw the curve
                let curve_points: PlotPoints = (0..=240)
                    .map(|i| {
                        let x = i as f32 / 10.0;
                        let y = self.config.get_temperature(x) as f64;
                        [x as f64, y]
                    })
                    .collect();
                plot_ui.line(Line::new(curve_points).width(2.0));

                // Draw control points
                let points: PlotPoints = self.config.points.iter()
                    .map(|p| [p.hour as f64, p.temp as f64])
                    .collect();
                plot_ui.points(Points::new(points).radius(6.0).color(egui::Color32::RED));

                // Handle dragging
                let is_pressed = plot_ui.ctx().input(|i| i.pointer.primary_pressed());
                let is_down = plot_ui.ctx().input(|i| i.pointer.primary_down());
                
                if is_pressed {
                     if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        // Find closest point
                        let mut closest_dist = f64::MAX;
                        let mut closest_idx = None;

                        for (i, p) in self.config.points.iter().enumerate() {
                            let dx = p.hour as f64 - pointer_pos.x;
                            let dy = (p.temp as f64 - pointer_pos.y) / 1000.0; // Scale y to make distance comparable
                            let dist = dx*dx + dy*dy;
                            if dist < 0.5 { // Threshold
                                if dist < closest_dist {
                                    closest_dist = dist;
                                    closest_idx = Some(i);
                                }
                            }
                        }
                        self.dragging_index = closest_idx;
                     }
                }
                
                if is_down {
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        if let Some(idx) = self.dragging_index {
                            // Update point
                            let p = &mut self.config.points[idx];
                            p.hour = pointer_pos.x.clamp(0.0, 24.0) as f32;
                            p.temp = pointer_pos.y.clamp(1000.0, 25000.0) as u32;
                        }
                    }
                } else {
                    if self.dragging_index.is_some() {
                        self.dragging_index = None;
                        // Sort points to ensure spline works correctly
                        self.config.points.sort_by(|a, b| a.hour.partial_cmp(&b.hour).unwrap());
                    }
                }

                // Double click to add point
                if plot_ui.response().double_clicked() {
                     if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        use crate::config::ConfigPoint;
                        let new_point = ConfigPoint {
                            hour: pointer_pos.x.clamp(0.0, 24.0) as f32,
                            temp: pointer_pos.y.clamp(1000.0, 25000.0) as u32,
                        };
                        self.config.points.push(new_point);
                        self.config.points.sort_by(|a, b| a.hour.partial_cmp(&b.hour).unwrap());
                     }
                }

                // Right click to remove point
                if plot_ui.response().clicked_by(egui::PointerButton::Secondary) {
                     if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        // Find closest point
                        let mut closest_dist = f64::MAX;
                        let mut closest_idx = None;

                        for (i, p) in self.config.points.iter().enumerate() {
                            let dx = p.hour as f64 - pointer_pos.x;
                            let dy = (p.temp as f64 - pointer_pos.y) / 1000.0; // Scale y to make distance comparable
                            let dist = dx*dx + dy*dy;
                            if dist < 0.5 { // Threshold
                                if dist < closest_dist {
                                    closest_dist = dist;
                                    closest_idx = Some(i);
                                }
                            }
                        }
                        
                        if let Some(idx) = closest_idx {
                            if self.config.points.len() > 1 {
                                self.config.points.remove(idx);
                            }
                        }
                     }
                }
            });
            
            ui.add_space(10.0);
            ui.label("💡 Drag red points to adjust the curve. Double-click to add new points. Right-click to remove points.");
        });
    }
}
