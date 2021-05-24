use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{prelude::*, stdout};

use cgmath::{dot, BaseFloat, InnerSpace, Vector3};
use image::{Rgb, RgbImage};
use num::{clamp, Zero};

use tinygraph_x::light::Light;
use tinygraph_x::scene::Scene;
use tinygraph_x::shapes::shape::{RayHit, Shape};

type Pixel = Vector3<f32>;

struct FrameBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Pixel>,
}

fn reflect(incoming: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    incoming - normal * 2.0 * dot(incoming, normal)
}

fn refract(incoming: Vector3<f32>, normal: Vector3<f32>, refractive_index: f32) -> Vector3<f32> {
    let mut cos_incoming = -clamp(dot(incoming, normal), -1.0, 1.0);
    let mut etai = 1.0;
    let mut etat = refractive_index;
    let mut n = normal;
    if cos_incoming < 0.0 {
        cos_incoming = -cos_incoming;
        std::mem::swap(&mut etai, &mut etat);
        n = -normal;
    }
    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cos_incoming * cos_incoming);

    if k < 0.0 {
        Vector3::zero()
    } else {
        incoming * eta + n * (eta * cos_incoming - k.sqrt())
    }
}

fn vec_norm<T: BaseFloat>(vec: Vector3<T>) -> T {
    (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt()
}

fn scene_intersect(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    shapes: &[Box<dyn Shape>],
) -> Option<RayHit> {
    // get the sphere with the shortest distance to orig
    shapes
        .iter()
        .filter_map(|shape| shape.ray_intersect(orig, dir))
        .min_by(|ray_hit_1, ray_hit_2| {
            ray_hit_1
                .hit_dist
                .partial_cmp(&ray_hit_2.hit_dist)
                .expect("tried to compare to NaN")
        })
}

fn cast_ray(
    ray_orig: Vector3<f32>,
    ray_dir: Vector3<f32>,
    shapes: &[Box<dyn Shape>],
    lights: &[Light],
    background: &RgbImage,
    depth: usize,
) -> Pixel {
    if depth > 10 {
        return get_background_pixel(ray_dir, background);
    }

    match scene_intersect(ray_orig, ray_dir, shapes) {
        Some(ray_hit) => {
            // calc reflect
            let reflect_dir = reflect(ray_dir, ray_hit.hit_normal);
            let reflect_orig = if dot(reflect_dir, ray_hit.hit_normal) < 0.0 {
                ray_hit.hit_point - ray_hit.hit_normal * 1e-3
            } else {
                ray_hit.hit_point + ray_hit.hit_normal * 1e-3
            };
            let reflect_color = cast_ray(
                reflect_orig,
                reflect_dir,
                shapes,
                lights,
                background,
                depth + 1,
            );

            // calc refract
            let refract_dir = refract(
                ray_dir,
                ray_hit.hit_normal,
                ray_hit.material.refractive_index,
            );
            let refract_orig = if dot(refract_dir, ray_hit.hit_normal) < 0.0 {
                ray_hit.hit_point - ray_hit.hit_normal * 1e-3
            } else {
                ray_hit.hit_point + ray_hit.hit_normal * 1e-3
            };
            let refract_color = cast_ray(
                refract_orig,
                refract_dir,
                shapes,
                lights,
                background,
                depth + 1,
            );

            let (diffuse_light_intensity, specular_light_intensity) = lights
                .iter()
                .map(|light| {
                    let light_dir = (light.position - ray_hit.hit_point).normalize();
                    let light_distance = vec_norm(light.position - ray_hit.hit_point);

                    let shadow_orig = if dot(light_dir, ray_hit.hit_normal) < 0.0 {
                        ray_hit.hit_point - ray_hit.hit_normal * 1e-3
                    } else {
                        ray_hit.hit_point + ray_hit.hit_normal * 1e-3
                    };

                    if let Some(shadow_hit) = scene_intersect(shadow_orig, light_dir, shapes) {
                        if vec_norm(shadow_hit.hit_point - shadow_orig) < light_distance {
                            return (0.0, 0.0);
                        }
                    }

                    (
                        light.intensity * 0.0f32.max(dot(light_dir, ray_hit.hit_normal)),
                        light.intensity
                            * 0.0f32
                                .max(dot(reflect(light_dir, ray_hit.hit_normal), ray_dir))
                                .powf(ray_hit.material.specular_exponent),
                    )
                })
                .fold((0.0, 0.0), |acc, x| (acc.0 + x.0, acc.1 + x.1)); // sum of both intensities

            ray_hit.material.diffuse_color * diffuse_light_intensity * ray_hit.material.albedo[0]
                + Vector3::new(1.0, 1.0, 1.0)
                    * specular_light_intensity
                    * ray_hit.material.albedo[1]
                + reflect_color * ray_hit.material.albedo[2]
                + refract_color * ray_hit.material.albedo[3]
        }
        None => get_background_pixel(ray_dir, background),
    }
}

fn get_background_pixel(ray_dir: Vector3<f32>, background: &RgbImage) -> Pixel {
    let envmap_width = background.width();
    let envmap_height = background.height();

    let x_raw = (((ray_dir.z.atan2(ray_dir.x) / (2.0 * PI) + 0.5) * envmap_width as f32)
        + envmap_width as f32)
        % envmap_width as f32;

    let y_raw = ray_dir.y.acos() / PI * envmap_height as f32;

    let x = clamp(x_raw as u32, 0, envmap_width - 1);
    let y = clamp(y_raw as u32, 0, envmap_height - 1);

    let bg_pixel: &Rgb<u8> = background.get_pixel(x, y);
    Pixel::new(
        bg_pixel.0[0] as f32,
        bg_pixel.0[1] as f32,
        bg_pixel.0[2] as f32,
    ) / 255.0
}

fn render(shapes: &[Box<dyn Shape>], lights: &[Light], background: &RgbImage) -> FrameBuffer {
    let fov: f32 = 80.0;

    let mut framebuffer = FrameBuffer {
        width: 1024 * 4,
        height: 768 * 4,
        buffer: Vec::<Vector3<f32>>::with_capacity(1024 * 768),
    };

    let mut stdout = stdout();
    for j in 0..framebuffer.height {
        print!("\rrendering {}%....", (j * 100) / framebuffer.height);
        stdout.flush().unwrap();
        for i in 0..framebuffer.width {
            let ray_dir_x = (2.0 * (i as f32 + 0.5) / framebuffer.width as f32 - 1.0)
                * (fov / 2.0).tan()
                * framebuffer.width as f32
                / framebuffer.height as f32;

            let ray_dir_y =
                -(2.0 * (j as f32 + 0.5) / framebuffer.height as f32 - 1.0) * (fov / 2.0).tan();

            let ray_dir = Vector3::new(ray_dir_x, ray_dir_y, -1.0).normalize();

            framebuffer.buffer.push(cast_ray(
                Vector3::zero(),
                ray_dir,
                shapes,
                lights,
                background,
                0,
            ));
        }
    }

    println!("\rrendering done   ");

    framebuffer
}

fn main() {
    let scene = Scene::from_file(&get_scene_file());

    let framebuffer = render(&scene.shapes, &scene.lights, &scene.background);

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
