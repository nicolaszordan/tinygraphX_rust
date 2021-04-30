use std::env;
use std::fs::File;
use std::io::prelude::*;

use cgmath::{InnerSpace, Vector3};
use num::{clamp, Zero};
use tinygraph_x::shapes::shape::Shape;
use tinygraph_x::shapes::sphere::Sphere;

type Pixel = Vector3<f32>;

struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Pixel>,
}

fn cast_ray(orig: Vector3<f32>, dir: Vector3<f32>, sphere: &Sphere) -> Pixel {
    let sphere_dist = std::f32::MAX;
    if sphere.ray_intersect(orig, dir, sphere_dist) {
        Pixel::new(0.4, 0.4, 0.3) // sphere color
    } else {
        Pixel::new(0.2, 0.7, 0.8) // background color
    }
}

fn render(sphere: Sphere) -> FrameBuffer {
    println!("rendering....");

    let fov: f32 = 80.0;

    let mut framebuffer = FrameBuffer {
        width: 1024,
        height: 768,
        buffer: Vec::<Vector3<f32>>::new(),
    };

    for j in 0..framebuffer.height {
        for i in 0..framebuffer.width {
            let ray_dir_x = (2.0 * (i as f32 + 0.5) / framebuffer.width as f32 - 1.0)
                * (fov / 2.0).tan()
                * framebuffer.width as f32
                / framebuffer.height as f32;

            let ray_dir_y =
                -(2.0 * (j as f32 + 0.5) / framebuffer.height as f32 - 1.0) * (fov / 2.0).tan();

            let ray_dir = Vector3::new(ray_dir_x, ray_dir_y, -1.0).normalize();

            framebuffer
                .buffer
                .push(cast_ray(Vector3::<f32>::zero(), ray_dir, &sphere));
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
            vec![
                ((255.0 * clamp(pixel.x, 0.0, 1.0)) as u8),
                ((255.0 * clamp(pixel.y, 0.0, 1.0)) as u8),
                ((255.0 * clamp(pixel.z, 0.0, 1.0)) as u8),
            ]
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
        String::from("out.ppm")
    }
}

fn main() {
    let sphere = Sphere::new(Vector3::new(0.0, 0.0, -10.0), 3.0);
    let framebuffer = render(sphere);
    export_to_ppm(&framebuffer, &get_out_file()).expect("failed to export to ppm");
}
