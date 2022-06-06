//! Image processing module for Glassvis application.

use std::f32;
use std::path::Path;

use image::{DynamicImage, FilterType, GenericImage, GenericImageView, GrayImage, ImageBuffer, RgbImage, RgbaImage, Luma, Pixel, Rgba};
use imageproc::contrast::{adaptive_threshold, equalize_histogram, otsu_level, threshold};
use imageproc::corners::{Corner, corners_fast9, corners_fast12};
use imageproc::drawing::{draw_hollow_rect};
use imageproc::definitions::{Clamp, HasWhite};
use imageproc::edges::canny;
use imageproc::gradients::{horizontal_sobel, vertical_sobel, sobel_gradients, horizontal_prewitt, vertical_prewitt, prewitt_gradients};
use imageproc::map::{red_channel, green_channel, blue_channel, map_pixels, map_subpixels};
use imageproc::rect::Rect;
use imageproc::utils::{pixel_diffs};

/// Represents a 2D point.
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Creates a dynamic image buffer from a specified image file.
pub fn open(img_file: &str) -> DynamicImage {
    let src = image::open(&img_file)
        .expect("No image found at specified path!");
    
    return src;
}

/// Creates a grayscale image buffer from a specified image file.
pub fn open_luma(img_file: &str) -> GrayImage {
    let src = image::open(&img_file)
        .expect("No image found at specified path!")
        .to_luma();

    return src;
}

/// Creates a grayscale image by extracting the specified channel of an RGB image.
pub fn extract_channel(src: RgbImage, channel: &str) -> GrayImage {
    let dst = match channel {
        "green" => green_channel(&src),
        "red"   => red_channel(&src),
        "blue"  => blue_channel(&src),
        _        => green_channel(&src),
    };
    
    return dst;
}

/// Resizes an image to a given width and height keeping the aspect ratio.
pub fn resize(src: DynamicImage, w: u32, h: u32) -> DynamicImage {
    return src.resize(w, h, FilterType::CatmullRom).unsharpen(0.25, 0);
}

/// Saves an image to an image file.
pub fn save(src: &DynamicImage, img_file: &str) {
    let file_path = Path::new(img_file);
    src.save(file_path).unwrap();
}

// Saves a grayscale image to an image file.
pub fn save_luma(src: &GrayImage, img_file: &str) {
    let file_path = Path::new(img_file);
    src.save(file_path).unwrap();
}

/// Applies Canny edge detection filter to input image.
pub fn apply_canny(src: &GrayImage, low: f32, high: f32) -> GrayImage {
    let dst = canny(src, low, high);
    return dst;
}

/// Applies sobel filter to input image.
pub fn sobel(src: &GrayImage) -> GrayImage {
    let filtered = sobel_gradients(&src);
    let dst = map_subpixels(&filtered, u8::clamp);
    return dst;
}

/// Applies horizontal sobel filter to input image.
pub fn sobel_horizon(src: &GrayImage) -> GrayImage {
    let filtered = horizontal_sobel(&src);
    let dst = map_subpixels(&filtered, |x| x as u8);
    return dst;
}

/// Applies vertical sobel filter to input image.
pub fn sobel_vertic(src: &GrayImage) -> GrayImage {
    let filtered = vertical_sobel(&src);
    let dst = map_subpixels(&filtered, |x| x as u8);
    return dst;
}

/// Applies prewitt filter to input image.
pub fn prewitt(src: &GrayImage) -> GrayImage {
    let filtered = prewitt_gradients(&src);
    let dst = map_subpixels(&filtered, u8::clamp);
    return dst;
}

/// Applies horizontal prewitt filter to input image.
pub fn prewitt_horizon(src: &GrayImage) -> GrayImage {
    let filtered = horizontal_prewitt(&src);
    let dst = map_subpixels(&filtered, |x| x as u8);
    return dst;
}

/// Applies vertical prewitt filter to input image.
pub fn prewitt_vertic(src: &GrayImage) -> GrayImage {
    let filtered = vertical_prewitt(&src);
    let dst = map_subpixels(&filtered, |x| x as u8);
    return dst;
}

/// Detects corners, also known as interest points, using FAST-9 features.
pub fn detect_corners_f9(src: &GrayImage, level: u8) -> Vec<Corner> {
    let corners = corners_fast9(&src, level);
    return corners;
}

/// Detects corners, also known as interest points, using FAST-12 features.
pub fn detect_corners_f12(src: &GrayImage, level: u8) -> Vec<Corner> {
    let corners = corners_fast12(&src, level);
    return corners;
}

/// Marks corners using the specified color.
pub fn mark_corners(src: &GrayImage, corners: Vec<Corner>) -> GrayImage {
    let (width, height) = src.dimensions();
    let mut dst = ImageBuffer::new(width, height);
    let white = Luma::white();
    
    for corner in corners.iter() {
        dst.put_pixel(corner.x, corner.y, white);
    }
    return dst;
}

/// Applies histogram equalization filter to input image.
pub fn equalize_hist(src: &GrayImage) -> GrayImage {
    let dst = equalize_histogram(src);
    return dst;
}

/// Applies an adaptive threshold to supplied image.
pub fn adaptive_thresh(src: &GrayImage, rad: u32) -> GrayImage {
    let dst = adaptive_threshold(src, rad);
    return dst;
}

/// Applies an otsu threshold to supplied image.
pub fn otsu_thresh(src: &GrayImage) -> GrayImage {
    let level = otsu_level(src);
    let dst = threshold(src, level);
    return dst;
}

/// Finds pixel diffs between specified images and marks them.
pub fn mark_diffs<'a>(src: &DynamicImage, dst: &'a mut DynamicImage, significance: u8)
-> (&'a DynamicImage, Vec<Point>, u32, u32, u32) {
    
    let diffs = pixel_diffs(src, dst, |p, q| p != q);
    let level = 255 / significance;
    
    let mut points = Vec::new();
    
    let mut counter: u32 = 0;
    let (width, height) = dst.dimensions();

    // filter diffs according to the defect significance
    for diff in diffs.iter() {
        let src_pix = src.get_pixel(diff.x, diff.y).to_luma()[0];
        let dst_pix = dst.get_pixel(diff.x, diff.y).to_luma()[0];

        if src_pix > dst_pix && src_pix - dst_pix > level {
            dst.put_pixel(diff.x, diff.y, Rgba([255, 0, 0, 0]));
            let pt = Point{ x: diff.x as i32, y: diff.y as i32 };
            points.push(pt);
            counter += 1;
        } else if src_pix < dst_pix && dst_pix - src_pix > level {
            dst.put_pixel(diff.x, diff.y, Rgba([255, 0, 0, 0]));
            let pt = Point{ x: diff.x as i32, y: diff.y as i32 };
            points.push(pt);
            counter += 1;
        }
    }

    return (dst, points, width, height, counter);
}

/// Finds edge points in given points vector and returns a bounding box.
pub fn get_box(points: Vec<Point>) -> Rect {
    // set a temporary value for all edge points
    let tmp = points.get(0).unwrap();
    let mut left:   i32 = tmp.x;
    let mut top:    i32 = tmp.y;
    let mut right:  i32 = tmp.x;
    let mut bottom: i32 = tmp.y;
    
    // extract edge points from points vector
    for pt in points.iter() {
        if pt.x < left {
            left = pt.x
        }
        if pt.y < top {
            top = pt.y
        }
        if pt.x > right {
            right = pt.x
        }
        if pt.y > bottom {
            bottom = pt.y
        }
    }
    
    // increase/decrease edge points by 1
    left   -= 1;
    top    -= 1;
    right  += 1;
    bottom += 1;
    
    let width = right - left;
    let height = bottom - top;
    
    // create a new rect representing the bounding box
    let rect = Rect::at(left, top).of_size(width as u32, height as u32);
    
    return rect;
}

/// Draws the borders of a given rectangle.
pub fn draw_rect(src: &DynamicImage, rect: Rect) -> DynamicImage {
    let color = Rgba([0, 255, 0, 0]);
    let bound: RgbaImage = draw_hollow_rect(src, rect, color);
    let dst = rgba_to_dynamic(bound);
    return dst;
}

/// Converts RgbaImage buffer to DynamicImage buffer.
pub fn rgba_to_dynamic(src: RgbaImage) -> DynamicImage {
    let dst = DynamicImage::ImageRgba8(src);
    return dst;
}
