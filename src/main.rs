use std::env;
use std::fs::File;
use std::io::prelude::*;

use cgmath::{InnerSpace, Vector3};
use num::{clamp, Zero};

use tinygraph_x::light::Light;
use tinygraph_x::shapes::material::{Color, Material};
use tinygraph_x::shapes::shape::Shape;
use tinygraph_x::shapes::sphere::Sphere;

type Pixel = Vector3<f32>;

struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Pixel>,
}

struct RayHit {
    hit_point: Vector3<f32>,
    hit_normal: Vector3<f32>,
    material: Material,
}

fn scene_intersect(orig: Vector3<f32>, dir: Vector3<f32>, spheres: &[Sphere]) -> Option<RayHit> {
    // get the sphere with the shortest distance to orig
    match spheres
        .iter()
        .filter_map(|sphere| match sphere.ray_intersect(orig, dir) {
            Some(sphere_dist) => Some((sphere, sphere_dist)),
            None => None,
        })
        .min_by(|(_, sphere_dist_1), (_, sphere_dist_2)| {
            sphere_dist_1
                .partial_cmp(sphere_dist_2)
                .expect("tried to compare to NaN")
        }) {
        Some((sphere, sphere_dist)) => {
            let hit_point = orig + dir * sphere_dist;
            let hit_normal = (hit_point - sphere.center).normalize();
            Some(RayHit {
                hit_point,
                hit_normal,
                material: *sphere.get_material(),
            })
        }
        None => None,
    }
}

fn cast_ray(
    ray_orig: Vector3<f32>,
    ray_dir: Vector3<f32>,
    spheres: &[Sphere],
    lights: &[Light],
) -> Pixel {
    match scene_intersect(ray_orig, ray_dir, spheres) {
        Some(ray_hit) => {
            let diffuse_light_intensity = lights
                .iter()
                .map(|light| {
                    let light_dir = (light.position - ray_hit.hit_point).normalize();
                    light.intensity * 0.0f32.max(cgmath::dot(light_dir, ray_hit.hit_normal))
                })
                .sum();

            ray_hit.material.diffuse_color * diffuse_light_intensity
        }
        None => Pixel::new(0.2, 0.7, 0.8), // background color
    }
}

fn render(spheres: &[Sphere], lights: &[Light]) -> FrameBuffer {
    println!("rendering....");

    let fov: f32 = 80.0;

    let mut framebuffer = FrameBuffer {
        width: 1024,
        height: 768,
        buffer: Vec::<Vector3<f32>>::with_capacity(1024 * 768),
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
                .push(cast_ray(Vector3::zero(), ray_dir, spheres, lights));
        }
    }

    println!("rendering done");

    framebuffer
}

fn main() {
    let ivory = Material::new(Color::new(0.4, 0.4, 0.3));
    let red_rubber = Material::new(Color::new(0.3, 0.1, 0.1));

    let spheres = vec![
        Sphere::new(Vector3::new(1.5, -0.5, -18.), 3.0, red_rubber),
        Sphere::new(Vector3::new(7.0, 5.0, -18.0), 4.0, ivory),
    ];

    let lights = vec![Light::new(Vector3::new(-20.0, 20.0, 20.0), 1.5)];

    let framebuffer = render(&spheres, &lights);

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
