
#[macro_use]
extern crate clap;
extern crate image;
extern crate colored;

use std::error::Error;
use std::convert::From;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::cmp;
use clap::{ArgMatches, AppSettings};
use image::{DynamicImage, GenericImage};
use colored::*;

#[derive(Debug)]
enum HsError {
    Io(io::Error),
    Image(image::ImageError)
}

impl fmt::Display for HsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HsError::Io(ref err) => err.fmt(f),
            HsError::Image(ref err) => err.fmt(f),
        }
    }
}

impl Error for HsError {
    fn description(&self) -> &str {
        match *self {
            HsError::Io(ref err) => err.description(),
            HsError::Image(ref err) => err.description()
        }
    }
}

impl From<io::Error> for HsError {
    fn from(error: io::Error) -> Self {
        HsError::Io(error)
    }
}

impl From<image::ImageError> for HsError {
    fn from(error: image::ImageError) -> Self {
        HsError::Image(error)
    }
}

type HsResult = std::result::Result<(), HsError>;

fn stencilize(img: DynamicImage, buffer: &mut Vec<u8>, num_layers: u32) {
    for pixel in img.pixels() {
        let color = pixel.2;
        let rgb = &color.data[0..3];
        for byte in rgb {
            let range = 256 / num_layers;
            let remainder = 256 % num_layers;
            for layer in 0..num_layers {
                let lo = layer * range;
                let mut hi = lo + (range -1);

                if lo == range * (num_layers - 1) {
                    hi += remainder;
                }

                if *byte >= lo as u8 && *byte <= hi as u8 {
                    buffer.push(*byte);
                } else {
                    buffer.push(0);
                }
             }
        }
    }
}

fn destencilize(in_buffer: Vec<u8>, imgbuf: &mut Vec<u8>, num_layers: u32) {
    for index in 0..imgbuf.len() {
        imgbuf[index] = 0;
        for layer in 0..num_layers {
            let i = num_layers as usize;
            let j = layer as usize;
            let k = ((index * i) + j) % (in_buffer.len()-1);
            imgbuf[index] = imgbuf[index].saturating_add(in_buffer[k]);
        }
    }
}

fn write_buffer(buffer: &Vec<u8>, output: &str) -> HsResult {
    let mut file = File::create(output)?;
    file.write_all(&buffer[..])?;
    Ok(())
}

fn read_buffer(buffer: &mut Vec<u8>, input: &str) -> HsResult {
    let mut file = File::open(input)?;
    file.read_to_end(buffer)?;
    Ok(())
}

fn write_image_buffer(imgbuf: &Vec<u8>, width: u32, height: u32, output: &str) -> HsResult {
    image::save_buffer(&Path::new(output), &imgbuf[..], width, height, image::RGB(8))?;
    Ok(())
}

fn encode(num_layers: u32, input: &str, output: &str) -> HsResult {
    let img = image::open(&Path::new(input))?;
    let mut buffer: Vec<u8> = Vec::new();
    stencilize(img, &mut buffer, num_layers);
    write_buffer(&buffer, output)?;
    Ok(())
}

fn decode(num_layers: u32, width: u32, height: u32, input: &str, output: &str) -> HsResult {
    let mut in_buffer: Vec<u8> = Vec::new();
    read_buffer(&mut in_buffer, input)?;

    let img_size = width * height * 3;

    let mut out_buffer: Vec<u8> = vec![0;img_size as usize];
    destencilize(in_buffer, &mut out_buffer, num_layers);
    write_image_buffer(&out_buffer, width, height, output)?;

    Ok(())
}

fn int_check(matches: &ArgMatches, name: &str) -> u32 {
    value_t!(matches.value_of(name), u32).unwrap_or_else(|e| {
        e.exit();
    })
}

fn run(matches: ArgMatches) -> HsResult {
    match matches.subcommand() {
        ("encode", Some(m)) => {
            let num_layers = cmp::max(1, int_check(m, "layers"));
            let input = m.value_of("INPUT").unwrap();
            let output = m.value_of("OUTPUT").unwrap();

            encode(num_layers, input, output)
        },
        ("decode", Some(m)) => {
            let num_layers = cmp::max(1, int_check(m, "layers"));
            let width = int_check(m, "width");
            let height = int_check(m, "height");
            let input = m.value_of("INPUT").unwrap();
            let output = m.value_of("OUTPUT").unwrap();

            decode(num_layers, width, height, input, output)
        },
        _ => Ok(())
    }
}

fn main() {
    let app = clap_app!(app =>
        (name: "Hyperstencil")
        (version: "0.1")
        (author: "Marc Sciglimpaglia <frozenbears@gmail.com>")
        (about: "A bespoke multichannel raw image utility")
        (@subcommand encode =>
            (about: "Encodes an RGB image into hyperstencil format")
            (@arg layers: -l --layers +required +takes_value "The number of layers to encode")
            (@arg INPUT: +required +takes_value "The input file")
            (@arg OUTPUT: +required +takes_value "The output file"))
        (@subcommand decode =>
            (about: "Decodes a hyperstencil file into an RGB image")
            (@arg layers: -l --layers +required +takes_value "The number of layers in the input file")
            (@arg width: -w --width +required +takes_value "The width of the image")
            (@arg height: -h --height +required +takes_value "The height of the image")
            (@arg INPUT: +required +takes_value "The input file")
            (@arg OUTPUT: +required +takes_value "The output file"))
     ).setting(AppSettings::SubcommandRequired);

    if let Err(e) = run(app.get_matches()) {
        eprintln!("{} {}", "error:".red().bold(), e);
        process::exit(1);
    }
}
