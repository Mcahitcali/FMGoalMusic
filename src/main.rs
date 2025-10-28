mod config;
mod audio;
mod capture;
mod ocr;
mod utils;

use std::thread;
use std::time::Duration;
use utils::{AppState, Debouncer, Timer};
use rdev::{listen, Event, EventType, Key, Button};
use std::collections::HashMap;

fn main() {
    println!("===========================================");
    println!("  FM Goal Musics - Goal Detection System");
    println!("===========================================\n");
    
    // Load configuration
    let cfg = match config::Config::load() {
        Ok(cfg) => {
            println!("✓ Configuration loaded");
            println!("  Capture region: [{}, {}, {}, {}]", 
                cfg.capture_region[0], cfg.capture_region[1],
                cfg.capture_region[2], cfg.capture_region[3]);
            println!("  OCR threshold: {}", cfg.ocr_threshold);
            println!("  Debounce: {}ms\n", cfg.debounce_ms);
            cfg
        }
        Err(e) => {
            eprintln!("✗ Failed to load config: {}", e);
            std::process::exit(1);
        }
    };
    
    // Run in test mode or production mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        run_tests(&cfg);
    } else {
        run_detection_loop(&cfg);
    }
}

/// Main detection loop - continuously monitors for "GOAL" text
fn run_detection_loop(cfg: &config::Config) {
    println!("Initializing detection system...\n");
    
    // Initialize audio manager
    let audio_path = match cfg.audio_file_full_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("✗ Failed to get audio path: {}", e);
            std::process::exit(1);
        }
    };
    
    let audio_manager = match audio::AudioManager::new(&audio_path) {
        Ok(manager) => {
            println!("✓ Audio manager initialized");
            println!("  Audio file: {}", audio_path.display());
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize audio: {}", e);
            eprintln!("  Make sure goal.mp3 exists at: {}", audio_path.display());
            std::process::exit(1);
        }
    };
    
    // Initialize capture manager
    let region = capture::CaptureRegion::from_array(cfg.capture_region);
    let mut capture_manager = match capture::CaptureManager::new(region) {
        Ok(manager) => {
            println!("✓ Capture manager initialized");
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize capture: {}", e);
            eprintln!("  Note: On macOS, grant Screen Recording permission");
            eprintln!("  System Preferences > Security & Privacy > Privacy > Screen Recording");
            std::process::exit(1);
        }
    };
    
    // Initialize OCR manager
    let mut ocr_manager = match ocr::OcrManager::new(cfg.ocr_threshold) {
        Ok(manager) => {
            println!("✓ OCR manager initialized");
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize OCR: {}", e);
            eprintln!("  Install Tesseract:");
            eprintln!("  macOS: brew install tesseract");
            eprintln!("  Linux: sudo apt-get install tesseract-ocr");
            std::process::exit(1);
        }
    };
    
    // Initialize application state and debouncer
    let state = AppState::new();
    let mut debouncer = Debouncer::new(cfg.debounce_ms);
    
    // Start paused - user must press F8 to begin
    state.set_paused(true);
    
    println!("\n===========================================");
    println!("  Detection system ready!");
    println!("  Press Cmd+1 to START/STOP detection");
    println!("  Press Ctrl+C to quit");
    println!("===========================================\n");
    println!("Status: PAUSED (press Cmd+1 to start)\n");
    
    // Setup Ctrl+C handler
    let state_clone = state.clone();
    ctrlc::set_handler(move || {
        println!("\n\nShutting down...");
        state_clone.stop();
    }).expect("Error setting Ctrl-C handler");
    
    // Setup keyboard listener in separate thread
    let state_for_keyboard = state.clone();
    
    thread::spawn(move || {
        let mut modifier_keys = HashMap::new();
        
        if let Err(e) = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    match key {
                        Key::ControlLeft | Key::MetaLeft => {
                            modifier_keys.insert(key, true);
                        }
                        Key::Num1 => {
                            // Check if Cmd is pressed
                            if modifier_keys.contains_key(&Key::MetaLeft) {
                                let is_paused = state_for_keyboard.toggle_pause();
                                if is_paused {
                                    println!("\n⏸️  PAUSED - Press Cmd+1 to resume\n");
                                } else {
                                    println!("\n▶️  STARTED - Monitoring for goals...\n");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                EventType::KeyRelease(key) => {
                    match key {
                        Key::ControlLeft | Key::MetaLeft => {
                            modifier_keys.remove(&key);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }) {
            eprintln!("Error setting up keyboard listener: {:?}", e);
        }
    });
    
    let mut frame_count = 0u64;
    let mut detection_count = 0u64;
    
    // Main detection loop
    while state.is_running() {
        frame_count += 1;
        
        // Check if paused
        if state.is_paused() {
            thread::sleep(Duration::from_millis(250));
            continue;
        }
        
        let loop_timer = Timer::start();
        
        // Capture screen
        let capture_timer = Timer::start();
        let image = match capture_manager.capture_region() {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Capture error: {}", e);
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        };
        let capture_time = capture_timer.elapsed_ms();
        
        // Perform OCR
        let ocr_timer = Timer::start();
        let goal_detected = match ocr_manager.detect_goal(&image) {
            Ok(detected) => detected,
            Err(e) => {
                eprintln!("OCR error: {}", e);
                false
            }
        };
        let ocr_time = ocr_timer.elapsed_ms();
        
        // If goal detected and not in debounce period, play sound
        if goal_detected && debouncer.should_trigger() {
            detection_count += 1;
            println!("\n🎯 GOAL DETECTED! (#{}) - Playing sound...", detection_count);
            
            if let Err(e) = audio_manager.play_sound() {
                eprintln!("  ✗ Failed to play sound: {}", e);
            }
        }
        
        let total_time = loop_timer.elapsed_ms();
        
        // Print status every 100 frames
        if frame_count % 100 == 0 {
            let fps = 1000.0 / total_time;
            println!("Frame {}: {:.1}ms total ({:.1} FPS) | Capture: {:.1}ms | OCR: {:.1}ms | Detections: {}",
                frame_count, total_time, fps, capture_time, ocr_time, detection_count);
        }
        
        // Small sleep to prevent CPU overuse (aim for ~60 FPS for faster response)
        let target_frame_time = 16.0; // ~60 FPS
        if total_time < target_frame_time {
            thread::sleep(Duration::from_millis((target_frame_time - total_time) as u64));
        }
    }
    
    println!("\n===========================================");
    println!("  Detection stopped");
    println!("  Total frames: {}", frame_count);
    println!("  Total detections: {}", detection_count);
    println!("===========================================");
}

/// Run test mode
fn run_tests(cfg: &config::Config) {
    println!("Running in TEST mode\n");
    
    // Run all tests
    test_config(cfg);
    test_audio(cfg);
    test_capture(cfg);
    test_ocr(cfg);
    
    println!("\n✓ All tests completed!");
}

fn test_config(cfg: &config::Config) {
    println!("=== Config Test ===");
    println!("  Capture region: {:?}", cfg.capture_region);
    println!("  Audio file: {}", cfg.audio_file_path);
    println!("  OCR threshold: {}", cfg.ocr_threshold);
    println!("  Debounce: {}ms", cfg.debounce_ms);
    println!("  Config path: {}", config::Config::config_dir_display());
    println!();
}

fn test_audio(cfg: &config::Config) {
    println!("=== Audio Test ===");
    
    let audio_path = match cfg.audio_file_full_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("✗ Failed to get audio path: {}", e);
            return;
        }
    };
    
    match audio::AudioManager::new(&audio_path) {
        Ok(audio_manager) => {
            println!("✓ Audio manager initialized");
            println!("  Audio file: {}", audio_path.display());
            
            // Uncomment to test playback
            // println!("  Playing test sound...");
            // if let Err(e) = audio_manager.play_sound() {
            //     eprintln!("  ✗ Failed to play: {}", e);
            // }
            // std::thread::sleep(std::time::Duration::from_secs(2));
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize audio: {}", e);
            eprintln!("  Make sure goal.mp3 exists at: {}", audio_path.display());
        }
    }
    println!();
}

fn test_capture(cfg: &config::Config) {
    println!("=== Capture Test ===");
    
    let region = capture::CaptureRegion::from_array(cfg.capture_region);
    let mut capture_manager = match capture::CaptureManager::new(region) {
        Ok(manager) => {
            println!("✓ Capture manager initialized");
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize capture: {}", e);
            eprintln!("  Note: On macOS, grant Screen Recording permission");
            eprintln!("  System Preferences > Security & Privacy > Privacy > Screen Recording");
            return;
        }
    };
    
    match capture_manager.capture_region() {
        Ok(image) => {
            let screenshot_path = "test_screenshot.png";
            match image.save(screenshot_path) {
                Ok(_) => {
                    println!("✓ Screenshot saved: {}", screenshot_path);
                    println!("  Size: {}x{}", image.width(), image.height());
                }
                Err(e) => {
                    eprintln!("✗ Failed to save screenshot: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to capture screen: {}", e);
        }
    }
    println!();
}

fn test_ocr(cfg: &config::Config) {
    println!("=== OCR Test ===");
    
    let mut ocr_manager = match ocr::OcrManager::new(cfg.ocr_threshold) {
        Ok(manager) => {
            println!("✓ OCR manager initialized");
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize OCR: {}", e);
            eprintln!("  Install Tesseract:");
            eprintln!("  macOS: brew install tesseract");
            eprintln!("  Linux: sudo apt-get install tesseract-ocr");
            return;
        }
    };
    
    // Test with test_screenshot.png
    if let Ok(test_image) = image::open("test-screenshot.png") {
        println!("\nTesting test-screenshot.png:");
        println!("  Full image size: {}x{}", test_image.width(), test_image.height());
        
        // Crop to the configured capture region
        let region = cfg.capture_region;
        println!("  Cropping to region: x={}, y={}, w={}, h={}", 
            region[0], region[1], region[2], region[3]);
        
        let test_image = test_image.to_rgba8();
        let cropped = image::imageops::crop_imm(
            &test_image,
            region[0],
            region[1],
            region[2],
            region[3]
        ).to_image();

        cropped.save("cropped.png").unwrap();
        
        println!("  Cropped size: {}x{}", cropped.width(), cropped.height());
        
        println!("\n--- ALL DETECTED TEXT ---");
        match ocr_manager.get_text(&cropped) {
            Ok(text) => {
                if text.is_empty() {
                    println!("  (no text detected)");
                } else {
                    println!("'{}'", text);
                    println!("--- END TEXT ---\n");
                    
                    // Show each line
                    println!("Lines detected:");
                    for (i, line) in text.lines().enumerate() {
                        println!("  Line {}: '{}'", i + 1, line);
                    }
                }
                
                println!("\n--- GOAL DETECTION ---");
                match ocr_manager.detect_goal(&cropped) {
                    Ok(detected) => {
                        if detected {
                            println!("  ✓ GOAL detected!");
                        } else {
                            println!("  ✗ GOAL not detected");
                            println!("  (Text must contain 'GOAL' substring)");
                        }
                    }
                    Err(e) => {
                        eprintln!("  ✗ Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to get text: {}", e);
            }
        }
    } else {
        eprintln!("✗ test-screenshot.png not found");
        eprintln!("  Run test_capture() first to create it");
    }
    println!();
}
