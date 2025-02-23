use std::{sync::Arc, time::Duration};

use eframe::egui::{self, Id, LayerId, Response, Slider, Theme, Vec2b};
use egui_plot::{Line, Plot, PlotBounds, PlotPoints};
use tokio::sync::RwLock;

use crate::Server;

pub fn server_window(server: Arc<RwLock<Server>>) -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.vsync = false;

    eframe::run_native(
        "Server Window",
        options,
        Box::new(|_cc| Ok(Box::new(ServerWindow::new(server)))),
    )
}

pub struct ServerWindow{
    immediate_data_usage: usize,
    data_points: Vec<[f64; 2]>,
    server: Arc<RwLock<Server>>,
    time: f64,
    height: f64,
}

impl ServerWindow {
    pub fn new(server: Arc<RwLock<Server>>) -> Self { 
        Self {
            time: 0.0,
            server,
            immediate_data_usage: 0,
            data_points: vec![[0.0, 0.0]],
            height: 512.0,
        }
    }
}

impl eframe::App for ServerWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut server) = self.server.try_write() {
            ctx.set_theme(Theme::Dark);
            ctx.request_repaint();

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("server gui").highlight();
                ui.label(format!("server addr: {:?}", server.socket.local_addr().unwrap()));
                ui.separator();
                
                ui.label(format!("{:?}", self.data_points.last().unwrap()[1]));

                if ui.button("remove all clients").clicked() {
                    if let Ok(mut packets) = server.client_packets.try_lock() {
                        packets.clear();
                    }
                    server.clients.clear();
                }

                ui.add(Slider::new(&mut self.height, 0.0..=1024.0));

                let points = PlotPoints::new(self.data_points.clone());
                while self.data_points.len() > 512 {
                    self.data_points.remove(0); 
                }
                let line = Line::new(points);

                ui.label("incoming packet flow");
                Plot::new("plot")
                    .width(400.0)
                    .height(200.0)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .allow_double_click_reset(false)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .show(ui, |plot_ui| {
                        plot_ui
                            .set_plot_bounds(
                                PlotBounds::from_min_max([0.0, self.height*0.02], [1.0, self.height])
                            );
                        plot_ui
                            .line(line);
                    });
                
                ui.label("outgoing packet flow (TODO)");
            });
            
            egui::SidePanel::right("client packets")
            .show(ctx, |ui| {
                ui.label("packets from clients: ");
                if let Ok(packets) = server.client_packets.try_lock() {
                    for client in packets.iter() {
                        ui.group(|ui| {
                            ui.label(format!("C: {:?}", client.0));
                            ui.separator();
                            ui.group(|ui| {
                                for packet in client.1.packets.keys() {
                                    ui.label(format!("P: {:?}", packet));
                                }
                            });
                        });
                        for packet in client.1.packets.values() {
                            self.immediate_data_usage += packet.data.len();
                        }

                    }
                    if self.time > 0.005 && self.time < 0.995 {
                        self.data_points.push([
                            self.time,
                            self.immediate_data_usage as f64
                        ]);
                    } else {
                        self.data_points.push([
                            self.time,
                            0.0
                        ]);
                    }

                    self.immediate_data_usage = 0;
                }
            });
            
            egui::SidePanel::right("client packets")
            .show(ctx, |ui| {
                ui.label("server packet storage: ");
                if let Ok(packets) = server.server_packets.try_lock() {
                    for packet in packets.packets.iter() {
                        ui.group(|ui| {
                            ui.label(format!("P: {:?}", packet.0));
                        });
                    }
                }
            });
            
            egui::TopBottomPanel::bottom("queue")
                .exact_height(32.0)
            .show(ctx, |ui| {
                ui.columns(2, |ui| {
                    ui[0].label(format!("queued task count: {:?}", server.queue.len()));
                    for command in &server.queue {
                        ui[1].label(format!("{:?}", command));
                    }
                });
            });

            egui::TopBottomPanel::bottom("pool")
                .max_height(100.0)
            .show(ctx, |ui| {
                ui.label("pool: ");
                ui.label(format!("active: {:?} queue: {:?} panic: {:?}", 
                    server.pool.active_count(), server.pool.queued_count(), server.pool.panic_count())
                );
            });

            self.time += 1.0 / 512.0;
            self.time %= 1.0;

        } else {
            ctx.request_discard("no server");
            
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("server is busy").highlight()
            });
        }

        std::thread::sleep_ms(16);
    }
}
