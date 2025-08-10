// Show VIC stream in egui

use eframe::egui::{ColorImage, Context, TextureOptions};
use eframe::{egui, App, NativeOptions};
use image::{ImageBuffer, Rgb};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ultimate64::vicstream::{self, capture_frame, get_socket};
use url::Url;

struct VideoApp {
    latest_frame: Arc<Mutex<Option<ImageBuffer<Rgb<u8>, Vec<u8>>>>>,
    texture_handle: Option<egui::TextureHandle>,
}

impl App for VideoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(frame) = self.latest_frame.lock().unwrap().as_ref() {
            let color_image = ColorImage::from_rgb(
                [frame.width() as usize, frame.height() as usize],
                &frame.clone(),
            );

            if let Some(tex) = &mut self.texture_handle {
                tex.set(color_image, TextureOptions::NEAREST);
            } else {
                self.texture_handle =
                    Some(ctx.load_texture("video", color_image, TextureOptions::NEAREST));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture_handle {
                ui.image(tex);
            } else {
                ui.label("Waiting for video frames...");
            }
        });

        // Keep redrawing for live video
        ctx.request_repaint();
    }
}

fn main() -> anyhow::Result<()> {
    let latest_frame = Arc::new(Mutex::new(None));
    let frame_clone = latest_frame.clone();

    // Background thread: receive frames from UDP and store latest
    thread::spawn(move || {
        let url = Url::parse("udp://239.0.1.64:11000").unwrap();
        let udp_socket = get_socket(&url).unwrap();

        loop {
            match capture_frame(udp_socket.try_clone().unwrap()) {
                Ok(data) => {
                    let img = vicstream::make_image(&data);
                    *frame_clone.lock().unwrap() = Some(img);
                }
                Err(e) => {
                    eprintln!("Error capturing frame: {:?}", e);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    });

    let app = VideoApp {
        latest_frame,
        texture_handle: None,
    };

    let native_options = NativeOptions::default();
    eframe::run_native(
        "VIC Video Stream",
        native_options,
        Box::new(|_| Ok(Box::new(app))),
    )
    .unwrap();

    Ok(())
}
