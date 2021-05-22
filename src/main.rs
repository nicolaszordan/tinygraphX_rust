use std::env;
use std::fs::File;
use std::io::prelude::*;

use cgmath::{dot, BaseFloat, InnerSpace, Vector3};
use num::{clamp, Zero};

use tinygraph_x::light::Light;
use tinygraph_x::shapes::material::{Albedo, Color, Material};
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
    depth: usize,
) -> Pixel {
    if depth > 4 {
        return Pixel::new(0.2, 0.7, 0.8); // background color
    }

    match scene_intersect(ray_orig, ray_dir, spheres) {
        Some(ray_hit) => {
            // calc reflect
            let reflect_dir = reflect(ray_dir, ray_hit.hit_normal);
            let reflect_orig = if dot(reflect_dir, ray_hit.hit_normal) < 0.0 {
                ray_hit.hit_point - ray_hit.hit_normal * 1e-3
            } else {
                ray_hit.hit_point + ray_hit.hit_normal * 1e-3
            };
            let reflect_color = cast_ray(reflect_orig, reflect_dir, spheres, lights, depth + 1);

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
            let refract_color = cast_ray(refract_orig, refract_dir, spheres, lights, depth + 1);

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

                    match scene_intersect(shadow_orig, light_dir, spheres) {
                        Some(shadow_hit) => {
                            if vec_norm(shadow_hit.hit_point - shadow_orig) < light_distance {
                                return (0.0, 0.0);
                            }
                        }
                        None => (),
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
                .push(cast_ray(Vector3::zero(), ray_dir, spheres, lights, 0));
        }
    }

    println!("rendering done");

    framebuffer
}

fn main() {
    let ivory = Material::new(
        Albedo::new(0.6, 0.3, 0.1, 0.0),
        Color::new(0.4, 0.4, 0.3),
        50.0,
        1.0,
    );
    let red_rubber = Material::new(
        Albedo::new(0.9, 0.1, 0.0, 0.0),
        Color::new(0.3, 0.1, 0.1),
        10.0,
        1.0,
    );
    let glass = Material::new(
        Albedo::new(0.0, 0.5, 0.1, 0.8),
        Color::new(0.6, 0.7, 0.8),
        125.0,
        1.5,
    );
    let mirror = Material::new(
        Albedo::new(0.0, 10.0, 0.8, 0.0),
        Color::new(1.0, 1.0, 1.0),
        1425.0,
        1.0,
    );

    let spheres = vec![
        Sphere::new(Vector3::new(-3.0, 0.0, -16.0), 2.0, ivory),
        Sphere::new(Vector3::new(-1.0, -1.5, -12.), 2.0, glass),
        Sphere::new(Vector3::new(1.5, -0.5, -18.), 3.0, red_rubber),
        Sphere::new(Vector3::new(7.0, 5.0, -18.0), 4.0, mirror),
    ];

    let lights = vec![
        Light::new(Vector3::new(-20.0, 20.0, 20.0), 1.5),
        Light::new(Vector3::new(30.0, 50.0, -25.0), 1.8),
        Light::new(Vector3::new(30.0, 20.0, 30.0), 1.7),
    ];

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

fn get_out_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("out.ppm")
    }
}
