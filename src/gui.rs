use std::{fmt::format, future, sync::Arc};

use eframe::egui;
use tokio::sync::{Mutex, RwLock};

use crate::Server;

pub fn server_window(server: Arc<RwLock<Server>>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Server Window",
        options,
        Box::new(|_cc| Ok(Box::new(ServerWindow::new(server)))),
    )
}

pub struct ServerWindow{
    robux: i32,
    server: Arc<RwLock<Server>>,
}

impl ServerWindow {
    pub fn new(server: Arc<RwLock<Server>>) -> Self { 
        Self {
            server,
            robux: 0,
        }
    }
}

impl eframe::App for ServerWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(server) = self.server.try_read() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("hello, world!");
                
                if ui.button("PRESS FOR ROBUX!!!").clicked(){
                    self.robux += 1;
                }

                ui.label(format!("{:?}", server.socket.local_addr().unwrap()));

                
                ui.label(format!("{}", self.robux));
            });

            egui::SidePanel::right("ongoing packets")
                .min_width(500.0)
            .show(ctx, |ui| {
                ui.label("ongoing packets: ");
                if let Ok(packets) = server.packets.try_lock() {
                    for client in packets.iter() {
                        ui.label(format!("C:{:?} -> {:?}", client.0, client.1));
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
        }
    }
}
