use arrayvec::ArrayVec;
use num::clamp;
use std::fs::File;
use std::io::prelude::*;

type Vec3<T> = [T; 3];

struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Vec3<f32>>,
}

fn render() -> FrameBuffer {
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

    framebuffer
}

fn export_to_ppm(framebuffer: &FrameBuffer) -> std::io::Result<()> {
    let mut file = File::create("out.ppm")?;
    let header = format!("P6\n{} {}\n255\n", framebuffer.width, framebuffer.height);

    file.write_all(header.as_bytes())?;

    for pixel in &framebuffer.buffer {
        file.write_all(
            pixel
                .iter()
                .map(|c| ((255.0 * clamp(*c, 0.0, 1.0)) as u8))
                .collect::<ArrayVec<u8, 3>>()
                .as_slice(),
        )?;
    }

    Ok(())
}

fn main() {
    println!("rendering....");
    let framebuffer = render();
    export_to_ppm(&framebuffer).expect("failed to export to ppm");
    println!("done");
}
