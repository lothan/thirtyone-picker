mod quantize;

#[macro_use] extern crate rocket;

use image::{EncodableLayout, ImageReader, RgbaImage};
use rocket::http::{ContentType, Status};
use rocket::{response, Build, Response, Rocket};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template, context};
use std::{io::{Cursor}, fs, path::PathBuf};
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
        if f.path().is_file() && fname.ends_with(".gif") {
            paths.push(f.path().file_name().unwrap().to_str().unwrap().into());
        }         
    }

    Ok(paths)
}
             
#[get("/conv/<algo>/<img_path>")]
fn convert_img(algo: &str, img_path: &str) -> Option<Vec<u8>> {
    let img = ImageReader::open(format!("imgs/{}", img_path)).unwrap().decode().unwrap();

    // let frames = img.into_frames();
    let mut out = Vec::new();
    let mut cursor = Cursor::new(&mut out);
    
    // let _ = img.write_to(&mut cursor, image::ImageFormat::Gif);
    // return Some(out)

    match quantize::simple_two_tone(&img) {
        None => None, //Response::build().status(Status::NoContent).finalize(),
        Some(quantized) => {
            quantized.write_to(&mut cursor, image::ImageFormat::Png).expect("Cannot write");
            Some(out)
            // Some(quantized.as_bytes().into())
        //     Response::build()
        //         .header(ContentType::GIF)
        //         .sized_body(imgbytes.len(), Cursor::new(imgbytes))
        //         .finalize()
        }
    }
}


#[launch]
fn rocket() -> _ {
    // env_logger::init();
               
    rocket::build().mount("/", routes![index, convert_img])
        .mount("/imgs/base", FileServer::from(relative!("imgs")))
        .attach(Template::fairing())
}
