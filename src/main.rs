use std::env;

use image::{ImageResult, RgbImage};

use tinygraph_x::scene::{FrameBuffer, Scene};

fn main() {
    let scene = Scene::from_file(&get_scene_file());
    let framebuffer = scene.render();

    export(&framebuffer, &get_out_file()).expect("failed to export to ppm");
}

fn export(framebuffer: &FrameBuffer, outfile: &str) -> ImageResult<()> {
    let image_buffer = RgbImage::from_vec(
        framebuffer.width as u32,
        framebuffer.height as u32,
        framebuffer
            .buffer
            .iter()
            .rev()
            .flat_map(|pixel| {
                [
                    ((255.0 * num::clamp(pixel.x, 0.0, 1.0)) as u8),
                    ((255.0 * num::clamp(pixel.y, 0.0, 1.0)) as u8),
                    ((255.0 * num::clamp(pixel.z, 0.0, 1.0)) as u8),
                ]
            })
            .collect(),
    )
    .unwrap();

    println!("exporting to {}...", outfile);
    image_buffer.save(outfile)?;
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
        String::from("out.png")
    }
}
