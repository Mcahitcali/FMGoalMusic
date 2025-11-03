use eframe::egui;
use image::{ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};

/// Region selector state
pub struct RegionSelector {
    screenshot: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    start_pos: Option<egui::Pos2>,
    current_pos: Option<egui::Pos2>,
    selected_region: Option<[u32; 4]>,
    texture: Option<egui::TextureHandle>,
}

impl RegionSelector {
    pub fn new() -> Self {
        Self {
            screenshot: None,
            start_pos: None,
            current_pos: None,
            selected_region: None,
            texture: None,
        }
    }
    
    pub fn capture_screenshot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Use xcap to capture the entire screen
        use xcap::Monitor;

        // Get all monitors
        let monitors = Monitor::all()
            .map_err(|e| format!("Failed to enumerate monitors: {}", e))?;

        if monitors.is_empty() {
            return Err("No monitors found".into());
        }

        // Get primary monitor (first in the list)
        let monitor = monitors.into_iter().next()
            .ok_or("Failed to get primary monitor")?;

        // Capture full screen with permission error handling
        // Note: xcap v0.7 returns ImageBuffer<Rgba<u8>, Vec<u8>> directly
        let image = monitor.capture_image().map_err(|e| -> Box<dyn std::error::Error> {
            let error_msg = format!("{}", e);

            #[cfg(target_os = "macos")]
            if error_msg.contains("permission") || error_msg.contains("denied") || error_msg.contains("authorization") {
                return format!(
                    "Screen Recording permission required.\n\
                    \n\
                    To grant permission on macOS:\n\
                    1. Open System Preferences/Settings > Privacy & Security\n\
                    2. Click 'Screen Recording'\n\
                    3. Enable permission for this application\n\
                    4. Restart the application and try again\n\
                    \n\
                    Original error: {}", e
                ).into();
            }

            format!("Failed to capture screen: {}", e).into()
        })?;

        self.screenshot = Some(image);
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn get_selected_region(&self) -> Option<[u32; 4]> {
        self.selected_region
    }
}


// Thread-local storage for region selection result
thread_local! {
    static REGION_RESULT: Arc<Mutex<Option<[u32; 4]>>> = Arc::new(Mutex::new(None));
}

/// Open region selector window and return selected region
pub fn select_region() -> Option<[u32; 4]> {
    REGION_RESULT.with(|result| {
        // Clear previous result
        *result.lock().unwrap() = None;
        
        let result_clone = Arc::clone(result);
        
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_decorations(false)
                .with_transparent(false),
            ..Default::default()
        };
        
        let _ = eframe::run_native(
            "Select Region",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(RegionSelectorApp {
                    selector: RegionSelector::new(),
                    result: Arc::clone(&result_clone),
                }))
            }),
        );
        
        // Get the result
        result.lock().unwrap().take()
    })
}

/// Wrapper app that handles region selection and sends result
struct RegionSelectorApp {
    selector: RegionSelector,
    result: Arc<Mutex<Option<[u32; 4]>>>,
}

impl eframe::App for RegionSelectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // If no screenshot, capture one
            if self.selector.screenshot.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Capturing screenshot...");
                });
                
                if let Err(e) = self.selector.capture_screenshot() {
                    ui.label(format!("Error: {}", e));
                }
                return;
            }
            
            // Load screenshot as texture
            if self.selector.texture.is_none() {
                if let Some(ref screenshot) = self.selector.screenshot {
                    let size = [screenshot.width() as usize, screenshot.height() as usize];
                    let pixels = screenshot.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        pixels.as_slice(),
                    );
                    self.selector.texture = Some(ctx.load_texture(
                        "screenshot",
                        color_image,
                        egui::TextureOptions::default(),
                    ));
                }
            }
            
            // Display screenshot
            if let Some(ref texture) = self.selector.texture {
                let available_size = ui.available_size();
                let img_size = texture.size_vec2();
                
                // Scale to fit screen
                let scale = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                let display_size = img_size * scale;
                
                let response = ui.add(
                    egui::Image::new(texture)
                        .fit_to_exact_size(display_size)
                        .sense(egui::Sense::click_and_drag())
                );
                
                // Handle mouse input for region selection
                if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let local = pos - response.rect.min;
                        let local_pos = egui::Pos2::new(local.x, local.y);
                        self.selector.start_pos = Some(local_pos);
                        self.selector.current_pos = Some(local_pos);
                    }
                }
                
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let local = pos - response.rect.min;
                        let local_pos = egui::Pos2::new(local.x, local.y);
                        self.selector.current_pos = Some(local_pos);
                    }
                }
                
                if response.drag_stopped() {
                    if let (Some(start), Some(end)) = (self.selector.start_pos, self.selector.current_pos) {
                        // Clamp points to the rendered image
                        let start = egui::Pos2::new(
                            start.x.clamp(0.0, display_size.x),
                            start.y.clamp(0.0, display_size.y),
                        );
                        let end = egui::Pos2::new(
                            end.x.clamp(0.0, display_size.x),
                            end.y.clamp(0.0, display_size.y),
                        );

                        let x1 = start.x.min(end.x);
                        let y1 = start.y.min(end.y);
                        let x2 = start.x.max(end.x);
                        let y2 = start.y.max(end.y);

                        // Convert from display coordinates to actual screen coordinates
                        let screen_width = texture.size()[0] as u32;
                        let screen_height = texture.size()[1] as u32;

                        let x = ((x1 / scale).round() as i64).clamp(0, (screen_width.saturating_sub(1)) as i64);
                        let y = ((y1 / scale).round() as i64).clamp(0, (screen_height.saturating_sub(1)) as i64);
                        let mut w = (((x2 - x1) / scale).round() as i64).max(1);
                        let mut h = (((y2 - y1) / scale).round() as i64).max(1);

                        let max_w = (screen_width as i64).saturating_sub(x).max(1);
                        let max_h = (screen_height as i64).saturating_sub(y).max(1);
                        w = w.min(max_w);
                        h = h.min(max_h);

                        let region = [x as u32, y as u32, w as u32, h as u32];

                        self.selector.selected_region = Some(region);
                        
                        // Store result
                        if let Ok(mut result) = self.result.lock() {
                            *result = Some(region);
                        }
                        
                        log::info!("✓ Region selected: {:?}", region);
                        
                        // Close window by requesting close
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                
                // Draw selection rectangle
                if let (Some(start), Some(current)) = (self.selector.start_pos, self.selector.current_pos) {
                    let rect = egui::Rect::from_two_pos(start, current);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                        egui::StrokeKind::Inside,
                    );
                    
                    // Show dimensions
                    let w = (rect.width() / scale) as u32;
                    let h = (rect.height() / scale) as u32;
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{}x{}", w, h),
                        egui::FontId::proportional(20.0),
                        egui::Color32::WHITE,
                    );
                }
            }
            
            // Instructions
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Click and drag to select the region where 'GOAL FOR' appears")
                        .size(20.0)
                        .color(egui::Color32::WHITE)
                        .background_color(egui::Color32::from_black_alpha(200))
                );
                ui.label(
                    egui::RichText::new("Press ESC to cancel")
                        .size(16.0)
                        .color(egui::Color32::WHITE)
                        .background_color(egui::Color32::from_black_alpha(200))
                );
            });
            
            // ESC to cancel
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        
        ctx.request_repaint();
    }
}
