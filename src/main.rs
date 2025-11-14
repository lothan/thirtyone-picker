mod quantize;

#[macro_use] extern crate rocket;

use image::codecs::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, DynamicImage, EncodableLayout, ImageEncoder, ImageFormat, ImageReader, RgbaImage};
use quantize::simple_two_tone;
// use rocket::futures::io::BufReader;
use rocket::http::{ContentType, Status};
use rocket::{response, Build, Response, Rocket};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};
use std::arch::x86_64::_mm_cmpunord_sd;
use std::fs::read_to_string;
use std::{io::{Cursor, BufReader}, fs, fs::File, path::PathBuf, any::Any};
// use log::{info, debug};
use anyhow::Result;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/")]
fn hello2() -> String {
    format!("Hello, {} year old named {}!", 4321, "fdjsklf")
}

#[get("/")]
fn index() -> Template {
    // rocket::build().mount("/", routes![hello2])
    let img_paths = get_img_paths("imgs").expect("error getting img paths");

    // format!("Hello, {} year old named {}!", "yoo", 342)
    Template::render("index", context! {img_paths: img_paths})
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

    let img_path = format!("imgs/{}", img_fname);

    let mut out = Vec::new();
    let mut cursor = Cursor::new(&mut out);
    let algo = quantize::lax_two_tone;
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
    rocket::build().mount("/", routes![index, convert_img])
        .mount("/imgs/base", FileServer::from(relative!("imgs")))
        .attach(Template::fairing())
}
