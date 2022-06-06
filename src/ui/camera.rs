//! Camera module for Glassvis application.

use rscam::{Camera, Config, Frame};
use std::fs;
use std::io::Write;
use std::time::SystemTime;

/// Initializes camera device.
pub fn init(){
    let mut cam = Camera::new("/dev/video0").unwrap();

    cam.start(&Config {
        // set interval to 30 fps
        interval: (1, 30),
        resolution: (1280, 720),
        format: b"MJPG",
        ..Default::default()
    })
    .unwrap();
}

/// Captures image and saves it using a timestamp.
pub fn capture_img(cam: &Camera) {
    let time_stamp = SystemTime::now();
    let mut frame: Frame;

    loop {
        frame = cam.capture().unwrap();
        if frame.len() > 0 {
            break;
        }
    }

    let mut file = fs::File::create(&format!("frame-{:?}.jpg", time_stamp)).unwrap();

    file.write_all(&frame).unwrap();
}
