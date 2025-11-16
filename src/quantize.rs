use std::path::PathBuf;
use std::collections::HashSet;

use image::{DynamicImage, GrayAlphaImage, GrayImage, ImageReader, Pixel, RgbImage, RgbaImage, Luma};


// Always returns either (0,0,0,0) pixels or (255, 255, 255, 255)

pub fn strict_two_tone(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_rgb8();
    let mut colors = HashSet::new();
    // let ret = RgbImage::new(img.width(), img.height());
    for px in img.pixels() {
        colors.insert(px);
        if colors.len() > 2 {
            return None
        }
    }

    let colors : Vec<_> = colors.drain().collect();
    let redpx = if colors[0].to_luma()[0] > colors[1].to_luma()[0] {
        colors[0]
    } else { colors[1] };

 
    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        if img.get_pixel(x, y) == redpx {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}

pub fn luma_threshold_128(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_rgb8();

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        if img.get_pixel(x, y).to_luma().0[0] >= 128 {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}

pub fn luma_threshold_128_inverted(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_rgb8();

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        if img.get_pixel(x, y).to_luma().0[0] < 128 {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}
