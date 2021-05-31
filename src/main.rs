use std::env;
use std::fs::File;
use std::io::prelude::*;

use tinygraph_x::scene::{FrameBuffer, Scene};

fn main() {
    let scene = Scene::from_file(&get_scene_file());
    let framebuffer = scene.render();

    export_to_ppm(&framebuffer, &get_out_file()).expect("failed to export to ppm");
}

fn export_to_ppm(framebuffer: &FrameBuffer, outfile: &str) -> std::io::Result<()> {
    println!("exporting to {}...", outfile);

    let mut file = File::create(outfile)?;
    let header = format!("P6\n{} {}\n255\n", framebuffer.width, framebuffer.height);

    file.write_all(header.as_bytes())?;

    let buffer = framebuffer
        .buffer
        .iter()
        .rev()
        .map(|pixel| {
            vec![
                ((255.0 * num::clamp(pixel.x, 0.0, 1.0)) as u8),
                ((255.0 * num::clamp(pixel.y, 0.0, 1.0)) as u8),
                ((255.0 * num::clamp(pixel.z, 0.0, 1.0)) as u8),
            ]
        })
        .flatten()
        .collect::<Vec<u8>>();

    file.write_all(buffer.as_slice())?;

    println!("exporting done!");

    Ok(())
}

fn get_scene_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        args[2].clone()
    } else {
        String::from("scene.json")
    }
}

fn get_out_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("out.ppm")
    }
}
