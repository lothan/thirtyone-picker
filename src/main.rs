mod quantize;

#[macro_use] extern crate rocket;

use image::codecs::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, DynamicImage, GrayImage, ImageFormat, ImageReader, Luma, Pixel};
use rocket::form::validate::range;
// use rocket::futures::io::BufReader;
// use rocket::http::{ContentType, Status};
// use rocket::{response::Redirect, Build, Response, Rocket};
use rocket::{response::Redirect};
use rocket::form::{Form};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};
use std::collections::HashMap;
use std::io::{Write};
use std::path::Path;
use std::sync::Arc;
use std::{io::{Cursor, BufReader}, fs, fs::File, option::Option};
// use log::{info, debug};
use anyhow::Result;
use serde::{Deserialize, Serialize};

const CHOICE_SAVE_PATH : &str = "choices.json";
const INPUT_IMG_DIR : &str = "imgs/";
// const OUTPUT_IMG_DIR : &str = "out/";
const OUTPUT_STILLS_DIR : &str = "stills/";
const OUTPUT_GIFS_DIR : &str = "gifs/";

const ALGOS : [(&str, fn(&DynamicImage) -> Option<GrayImage>); 6] = [
    ("lt128", quantize::luma_threshold_128), 
    ("ltm", quantize::luma_threshold_mean), 
    ("rtm", quantize::red_threshold_mean), 
    ("dm2", quantize::dither_m2_threshold), 
    ("dm4", quantize::dither_m4_threshold), 
    ("dm8", quantize::dither_m8_threshold),
];

// TODO: specify preambles and deletes with checkboxes

#[derive(FromForm, Serialize, Deserialize)]
struct SubmitForm {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    name: String,
    algo: String,
}

#[get("/")]
fn index() -> Template {
    let choices_json : String = fs::read_to_string(CHOICE_SAVE_PATH).unwrap_or(String::from("[]"));
    let choices : Vec<Choice> = serde_json::from_str(&choices_json).expect("error deserializing choices");
    let img_paths = get_img_paths(INPUT_IMG_DIR).expect("error getting img paths");

    let algo_names = ALGOS.map(|x| x.0);
    Template::render("index", context! {img_paths: img_paths,
                                        algo_names: algo_names,
                                        choices: choices,
    })
}

fn still_to_rrframe(img: GrayImage) -> Vec<u8> {
    let mut ret = vec![];
    if img.width() != 88 && img.height() != 31 {
        panic!("still_to_rrframe: Incorrect dimensions!");
    }
    let mut cur_byte = 0;
    for y in 0..38 {
        for x in 0..96 {
            let bit = if y < 3 || y >= 34 || x < 4 || x >= 92 ||
                *img.get_pixel(x-4, y-3) == Luma([0]) { 0 } else { 1 };
            cur_byte |= bit << 7 - (x % 8);
            if x % 8 == 7 {
                ret.push(cur_byte);
                cur_byte = 0;
            }
        }
    }

    ret 
}

fn get_gif_duration_ms(gif: Vec<u8>) -> i32 {

    0
}

fn to_rrimg(input: Vec<u8>) -> Option<Vec<u8>> {
    let mut ret = vec![];

    let cursor = Cursor::new(&input);
    match image::guess_format(&input).expect("cannot guess converted_data") {
        ImageFormat::Gif => {
            let gif_dec = GifDecoder::new(cursor).unwrap();

            let frames : Vec<_> = gif_dec.into_frames().into_iter().collect();
            if frames.len() == 1 {
                let frame = frames.get(0).unwrap().as_ref().unwrap();
                let img = DynamicImage::from(frame.clone().into_buffer());
                ret = still_to_rrframe(img.into_luma8());
            } else { 
                for frame in frames {
                    let frame = frame.unwrap();
                    let delay = frame.delay();
                    let grayimg = DynamicImage::from(frame.into_buffer()).into_luma8();
                    let mut rrframe = still_to_rrframe(grayimg);

                    // rapidriter is 10fps and 1000 frames
                    // https://github.com/gregsadetsky/rapidriteros/blob/main/renderers/wasm/src/main.rs#L63
                    // so number of frames per frame is 100*ms

                    let (num, denom) = delay.numer_denom_ms();
                    let delay_ms = num as f64 / denom as f64;

                    let real_delay_ms = delay_ms.max(100.0);
                    let num_frames = (real_delay_ms / 100.0).round() as usize;
                    let num_frames = num_frames.max(1);
                    for _ in 0..num_frames {
                        ret.append(&mut rrframe.clone());
                    }
                }
            }
        },
        ImageFormat::Png => {
            let img = ImageReader::with_format(cursor, ImageFormat::Png).decode().unwrap();
            ret = still_to_rrframe(img.to_luma8());
        },
        _ => { return None }
    }

    if ret.is_empty() {
        None
    } else {
        Some(ret)
    }
}

#[post("/submit_choices", data="<form>")]
fn submit_choices(form: Form<HashMap<String, String>>) -> Redirect {
    let submissions = form.into_inner();
    let mut f = File::create(CHOICE_SAVE_PATH).unwrap();

    let choices : Vec<(&str, &str)> = submissions.iter().map(|(_, val)| val.split_once(":").unwrap()).collect();

    f.write(serde_json::to_string_pretty(&choices).unwrap().as_bytes());

    fs::remove_dir_all(OUTPUT_STILLS_DIR);
    fs::create_dir(OUTPUT_STILLS_DIR);
    fs::remove_dir_all(OUTPUT_GIFS_DIR);
    fs::create_dir(OUTPUT_GIFS_DIR);
    for (img_fname, algo_name) in choices {
        let converted = convert_img(algo_name, img_fname).expect("Cannot convert image");
        if let Some(rrimg_data) = to_rrimg(converted) { 
            let out_img_path = if rrimg_data.len() == 456 {
                OUTPUT_STILLS_DIR.to_string() + img_fname + "." + algo_name + ".rrimg"
            } else {
                OUTPUT_GIFS_DIR.to_string() + img_fname + "." + algo_name + ".rrimg"
            };
            
            let mut outfile = File::create(out_img_path).expect("Cannot create outfile");
            outfile.write(&rrimg_data);
        };
    };
    
    Redirect::to("/")
}


fn get_img_paths(dir: &str) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    
    for f in fs::read_dir(dir)? {
        let f = f?;
        let fname = f.file_name().into_string().expect("bad dir string");
        if f.path().is_file() {
            paths.push(fname);
        }         
    }

    Ok(paths)
}


#[get("/conv/<algo_name>/<img_fname>")]
fn convert_img(algo_name: &str, img_fname: &str) -> Option<Vec<u8>> {
    // let img_path = format!("{}", img_fname);
    let img_path = INPUT_IMG_DIR.to_string() + img_fname;

    let mut out = Vec::new();
    let mut cursor = Cursor::new(&mut out);

    let algo_match : Vec<_> = ALGOS.into_iter().filter(|x| x.0 == algo_name).collect();
    if algo_match.len() != 1 {
        return None
    }
    let algo = algo_match[0].1;

    // let algo = quantize::lax_two_tone;
    match image::guess_format(&std::fs::read(&img_path).expect("cannot read")).expect("cannot guess string") {
        ImageFormat::Gif => {

            let gif_reader = BufReader::new(File::open(&img_path).expect("Cannot open file"));
            let gif_dec = GifDecoder::new(gif_reader).expect("Cannot create gif decoder");

            let mut gif_enc = GifEncoder::new(cursor);
            gif_enc.set_repeat(image::codecs::gif::Repeat::Infinite).expect("cannot set infinite");

            for frame in gif_dec.into_frames() {
                let frame = frame.expect("cannot into_frames");
                let (left, top, delay) = (frame.left(), frame.top(), frame.delay());

                if let Some(out_img) = algo(&image::DynamicImage::from(frame.into_buffer())) {
                    let rgba_img = DynamicImage::ImageLuma8(out_img).to_rgba8();
                    let out_frame = image::Frame::from_parts(rgba_img, left, top, delay);
                    gif_enc.encode_frame(out_frame).expect("cannot encode frame")
                } else {
                    return None
                }
            }

            drop(gif_enc);  // Drop encoder to release the borrow of `out`
            return Some(out)
            // match quantize::simple_two_tone(&img) {
            //     None => None, //Response::build().status(Status::NoContent).finalize(),
            //     Some(quantized) => {
            //         quantized.write_to(&mut cursor, image::ImageFormat::Png).expect("Cannot write");
            //         Some(out)
            //     }
            // }

        },
        ImageFormat::Png => {
            let img = ImageReader::open(img_path).unwrap().decode().unwrap();
            if let Some(out_img) = algo(&img) {
                out_img.write_to(&mut cursor, image::ImageFormat::Png).expect("cannot png to cursor");
            } else {
                return None
            }

            Some(out)
        },
        _ => None
    }
    // let img_dec : GifDecoder<R> = ImageReader::open(format!("imgs/{}", img_path)).unwrap().into_decoder().unwrap();

}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, convert_img, submit_choices])
        .mount("/imgs/base", FileServer::from(Path::new(INPUT_IMG_DIR).canonicalize().unwrap()))
        .attach(Template::fairing())
}
