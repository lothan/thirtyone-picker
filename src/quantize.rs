use std::path::PathBuf;
use std::collections::HashSet;

use image::{DynamicImage, GrayImage, Pixel, Luma};

// SVM
// (2) K means clustering
// do the luma based on the median 
// DBScan?? 

// Always returns either (0,0,0,0) pixels or (255, 255, 255, 255)

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

pub fn luma_threshold_mean(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_luma8();

    let pxs : Vec<u8> =  img.pixels().map(|x| x.0[0]).collect();
    let sum : u32 = pxs.iter().map(|&x| x as u32).sum(); 
    let mean = (sum / pxs.len() as u32) as u8;

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        if img.get_pixel(x, y).to_luma().0[0] > mean {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}

pub fn red_threshold_mean(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_rgb8();

    let pxs : Vec<u8> =  img.pixels().map(|x| x.0[0]).collect();
    let sum : u32 = pxs.iter().map(|&x| x as u32).sum(); 
    let mean = (sum / pxs.len() as u32) as u8;

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        if img.get_pixel(x, y).to_rgb().0[0] > mean {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}


const M2_THRESHOLD_MAP : [[u8; 2]; 2] = [[0, 192], [128, 64]];

pub fn dither_m2_threshold(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_luma8();

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let threshold = M2_THRESHOLD_MAP[(x % 2) as usize][(y % 2) as usize]; 
        if img.get_pixel(x, y).to_luma().0[0] > threshold {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}

const M4_THRESHOLD_MAP : [[u8; 4]; 4] = [[0, 192, 48, 240], [128, 64, 176, 112], [32, 224, 16, 208], [160, 96, 114, 80]];

pub fn dither_m4_threshold(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_luma8();

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let threshold = M4_THRESHOLD_MAP[(x % 4) as usize][(y % 4) as usize]; 
        if img.get_pixel(x, y).to_luma().0[0] > threshold {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}
    
const M8_THRESHOLD_MAP : [[u8; 8]; 8] = [[0, 128, 32, 160, 8, 136, 40, 168], [192, 64, 224, 96, 200, 72, 232, 104], [48, 176, 16, 144, 56, 184, 24, 152], [240, 112, 208, 80, 248, 120, 216, 88], [12, 140, 44, 172, 4, 132, 36, 164], [204, 76, 236, 108, 196, 68, 228, 100], [60, 188, 28, 156, 52, 180, 20, 148], [252, 124, 220, 92, 244, 116, 212, 84]] ;

pub fn dither_m8_threshold(img: &DynamicImage) -> Option<GrayImage> {
    let img = img.clone().into_luma8();

    Some(GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let threshold = M8_THRESHOLD_MAP[(x % 8) as usize][(y % 8) as usize]; 
        if img.get_pixel(x, y).to_luma().0[0] > threshold {
            Luma([255])
        } else {
            Luma([0])
        }
    }))
}
