use arrayvec::ArrayVec;
use num::clamp;
use std::env;
use std::fs::File;
use std::io::prelude::*;

type Vec3<T> = [T; 3];

struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Vec3<f32>>,
}

fn render() -> FrameBuffer {
    println!("rendering....");

    let mut framebuffer = FrameBuffer {
        width: 1024,
        height: 768,
        buffer: Vec::<Vec3<f32>>::new(),
    };

    for j in 0..framebuffer.height {
        for i in 0..framebuffer.width {
            let pixel = [
                j as f32 / framebuffer.height as f32,
                i as f32 / framebuffer.width as f32,
                0.0,
            ];
            framebuffer.buffer.push(pixel);
        }
    }

    println!("rendering done");

    framebuffer
}

fn export_to_ppm(framebuffer: &FrameBuffer, outfile: &str) -> std::io::Result<()> {
    println!("exporting to {}...", outfile);

    let mut file = File::create(outfile)?;
    let header = format!("P6\n{} {}\n255\n", framebuffer.width, framebuffer.height);

    file.write_all(header.as_bytes())?;

    let buffer = framebuffer
        .buffer
        .iter()
        .map(|pixel| {
            pixel
                .iter()
                .map(|c| ((255.0 * clamp(*c, 0.0, 1.0)) as u8))
                .collect::<ArrayVec<u8, 3>>()
        })
        .flatten()
        .collect::<Vec<u8>>();

    file.write_all(buffer.as_slice())?;

    println!("exporting done");

    Ok(())
}

fn get_out_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args[1].clone()
    } else {
        "out.ppm".to_owned()
    }
}

fn main() {
    let framebuffer = render();
    export_to_ppm(&framebuffer, &get_out_file()).expect("failed to export to ppm");
}
