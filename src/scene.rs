use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs::File;

use cgmath::{InnerSpace, Vector3};
use image::io::Reader as ImageReader;
use image::{Rgb, RgbImage};
use num::Zero;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use indicatif::{ParallelProgressIterator};

use crate::light::Light;
use crate::shapes::material::Material;

use crate::shapes::checkboard_disk::CheckBoardDisk;
use crate::shapes::disk::Disk;
use crate::shapes::mesh::Mesh;
use crate::shapes::plane::Plane;
use crate::shapes::polygon::Polygon;
use crate::shapes::shape::{Ray, RayHit, Shape};
use crate::shapes::sphere::Sphere;

pub struct Scene {
    pub materials: HashMap<String, Material>,
    pub lights: Vec<Light>,
    pub shapes: Vec<Box<dyn Shape + Sync>>,
    pub background: RgbImage,

    pub frame_width: usize,
    pub frame_height: usize,
    pub fov: f32,
    pub max_reflect_depth: usize,
}

pub type Pixel = Vector3<f32>;

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Pixel>,
}

impl Scene {
    pub fn from_file(file_path: &str) -> Self {
        println!("importing scene: [file={}]", file_path);
        let scene_json = SceneJson::from_file(file_path);
        let shapes = scene_json
            .shapes
            .spheres
            .iter()
            .map(|sphere| {
                Box::new(sphere.clone().into_sphere(&scene_json.materials)) as Box<dyn Shape + Sync>
            })
            .chain(scene_json.shapes.planes.iter().map(|plane| {
                Box::new(plane.clone().into_plane(&scene_json.materials)) as Box<dyn Shape + Sync>
            }))
            .chain(scene_json.shapes.disks.iter().map(|disk| {
                Box::new(disk.clone().into_disk(&scene_json.materials)) as Box<dyn Shape + Sync>
            }))
            .chain(scene_json.shapes.checkboard_disks.iter().map(|disk| {
                Box::new(disk.clone().into_checkboard_disk(&scene_json.materials))
                    as Box<dyn Shape + Sync>
            }))
            .chain(scene_json.shapes.polygons.iter().map(|polygon| {
                Box::new(polygon.clone().into_polygon(&scene_json.materials))
                    as Box<dyn Shape + Sync>
            }))
            .chain(scene_json.shapes.objs.iter().map(|obj| {
                Box::new(obj.clone().into_mesh(&scene_json.materials)) as Box<dyn Shape + Sync>
            }))
            .collect();
        println!("importing scene done!");

        println!("importing background: [file={}]", file_path);
        let background = Scene::create_background(&scene_json.background);
        println!("importing background done!");

        Self {
            materials: scene_json.materials,
            lights: scene_json.lights,
            shapes,
            background,
            frame_width: scene_json.frame_width,
            frame_height: scene_json.frame_height,
            fov: -scene_json.fov_in_degrees * (PI / 180.0),
            max_reflect_depth: scene_json.max_reflect_depth,
        }
    }

    fn create_background(background_file: &str) -> RgbImage {
        ImageReader::open(background_file)
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open background file: {}: {}",
                    background_file, err
                )
            })
            .decode()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to decode background file: {}: {}",
                    background_file, err
                )
            })
            .to_rgb8()
    }

    pub fn render(&self) -> FrameBuffer {
        println!("rendering...");
        let framebuffer = FrameBuffer {
            width: self.frame_width,
            height: self.frame_height,
            buffer: (0..self.frame_height)
                .into_par_iter()
                .progress()
                .map(|y| self.render_line(y))
                .flatten()
                .collect(),
        };
        println!("rendering done!");

        framebuffer
    }

    fn render_line(&self, y: usize) -> Vec<Pixel> {
        (0..self.frame_width)
            .map(|x| {
                let ray_dir_x = (2.0 * (x as f32 + 0.5) / self.frame_width as f32 - 1.0)
                    * (self.fov / 2.0).tan()
                    * self.frame_width as f32
                    / self.frame_height as f32;

                let ray_dir_y = -(2.0 * (y as f32 + 0.5) / self.frame_height as f32 - 1.0)
                    * (self.fov / 2.0).tan();

                let ray_dir = Vector3::new(ray_dir_x, ray_dir_y, -1.0).normalize();
                let ray = Ray::new(Vector3::zero(), ray_dir);

                self.cast_ray(&ray, 0)
            })
            .collect()
    }

    fn cast_ray(&self, ray: &Ray, depth: usize) -> Pixel {
        if depth > self.max_reflect_depth {
            return self.get_background_pixel(ray.direction);
        }

        match self.scene_intersect(ray) {
            Some(ray_hit) => {
                // calc reflect
                let reflect_dir = reflect(ray.direction, ray_hit.hit_normal);
                let reflect_orig = if reflect_dir.dot(ray_hit.hit_normal) < 0.0 {
                    ray_hit.hit_point - ray_hit.hit_normal * 1e-3
                } else {
                    ray_hit.hit_point + ray_hit.hit_normal * 1e-3
                };
                let reflect_ray = Ray::new(reflect_orig, reflect_dir);
                let reflect_color = self.cast_ray(&reflect_ray, depth + 1);

                // calc refract
                let refract_dir = refract(
                    ray.direction,
                    ray_hit.hit_normal,
                    ray_hit.material.refractive_index,
                );
                let refract_orig = if refract_dir.dot(ray_hit.hit_normal) < 0.0 {
                    ray_hit.hit_point - ray_hit.hit_normal * 1e-3
                } else {
                    ray_hit.hit_point + ray_hit.hit_normal * 1e-3
                };
                let refract_ray = Ray::new(refract_orig, refract_dir);
                let refract_color = self.cast_ray(&refract_ray, depth + 1);

                let (diffuse_light_intensity, specular_light_intensity) = self
                    .lights
                    .iter()
                    .map(|light| {
                        let light_dir = (light.position - ray_hit.hit_point).normalize();
                        let light_distance = (light.position - ray_hit.hit_point).magnitude();

                        let shadow_orig = if light_dir.dot(ray_hit.hit_normal) < 0.0 {
                            ray_hit.hit_point - ray_hit.hit_normal * 1e-3
                        } else {
                            ray_hit.hit_point + ray_hit.hit_normal * 1e-3
                        };

                        let shadow_ray = Ray::new(shadow_orig, light_dir);

                        if let Some(shadow_hit) = self.scene_intersect(&shadow_ray) {
                            if (shadow_hit.hit_point - shadow_orig).magnitude() < light_distance {
                                return (0.0, 0.0);
                            }
                        }

                        (
                            light.intensity * 0.0f32.max(light_dir.dot(ray_hit.hit_normal)),
                            light.intensity
                                * 0.0f32
                                    .max(reflect(light_dir, ray_hit.hit_normal).dot(ray.direction))
                                    .powf(ray_hit.material.specular_exponent),
                        )
                    })
                    .fold((0.0, 0.0), |acc, x| (acc.0 + x.0, acc.1 + x.1)); // sum of both intensities

                ray_hit.material.diffuse_color
                    * diffuse_light_intensity
                    * ray_hit.material.albedo[0]
                    + Vector3::new(1.0, 1.0, 1.0)
                        * specular_light_intensity
                        * ray_hit.material.albedo[1]
                    + reflect_color * ray_hit.material.albedo[2]
                    + refract_color * ray_hit.material.albedo[3]
            }
            None => self.get_background_pixel(ray.direction),
        }
    }

    fn scene_intersect(&self, ray: &Ray) -> Option<RayHit> {
        // get the shape with the shortest distance to orig
        self.shapes
            .iter()
            .filter_map(|shape| shape.ray_intersect(ray))
            .min_by(|ray_hit_1, ray_hit_2| {
                ray_hit_1
                    .hit_dist
                    .partial_cmp(&ray_hit_2.hit_dist)
                    .expect("tried to compare to NaN")
            })
    }

    fn get_background_pixel(&self, ray_dir: Vector3<f32>) -> Pixel {
        let envmap_width = self.background.width();
        let envmap_height = self.background.height();

        let x_raw = (((ray_dir.z.atan2(ray_dir.x) / (2.0 * PI) + 0.5) * envmap_width as f32)
            + envmap_width as f32)
            % envmap_width as f32;

        let y_raw = ray_dir.y.acos() / PI * envmap_height as f32;

        let x = num::clamp(x_raw as u32, 0, envmap_width - 1);
        let y = num::clamp(y_raw as u32, 0, envmap_height - 1);

        let bg_pixel: &Rgb<u8> = self.background.get_pixel(x, y);

        Pixel::new(
            bg_pixel.0[0] as f32,
            bg_pixel.0[1] as f32,
            bg_pixel.0[2] as f32,
        ) / 255.0
    }
}

fn reflect(incoming: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    incoming - normal * 2.0 * incoming.dot(normal)
}

fn refract(incoming: Vector3<f32>, normal: Vector3<f32>, refractive_index: f32) -> Vector3<f32> {
    let mut cos_incoming = -num::clamp(incoming.dot(normal), -1.0, 1.0);
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

#[derive(Serialize, Deserialize)]
struct SceneJson {
    pub materials: HashMap<String, MaterialJson>,
    pub lights: Vec<LightJson>,
    pub shapes: ShapesJson,
    pub background: String,
    pub frame_width: usize,
    pub frame_height: usize,
    pub fov_in_degrees: f32,
    pub max_reflect_depth: usize,
}

#[derive(Serialize, Deserialize)]
struct ShapesJson {
    pub spheres: Vec<SphereJson>,
    pub planes: Vec<PlaneJson>,
    pub disks: Vec<DiskJson>,
    pub checkboard_disks: Vec<CheckBoardDiskJson>,
    pub polygons: Vec<PolygonJson>,
    pub objs: Vec<ObjJson>,
}

type MaterialJson = Material;

type LightJson = Light;

#[derive(Serialize, Deserialize, Clone)]
struct SphereJson {
    center: Vector3<f32>,
    radius: f32,
    material: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PlaneJson {
    point: Vector3<f32>,
    normal: Vector3<f32>,
    material: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct DiskJson {
    center: Vector3<f32>,
    normal: Vector3<f32>,
    radius: f32,
    material: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct CheckBoardDiskJson {
    center: Vector3<f32>,
    normal: Vector3<f32>,
    radius: f32,
    dist_between_mats: f32,
    material1: String,
    material2: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PolygonJson {
    vertex_0: Vector3<f32>,
    vertex_1: Vector3<f32>,
    vertex_2: Vector3<f32>,
    material: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ObjJson {
    wavefront: String,
    material: String,
}

impl SceneJson {
    fn from_file(file_path: &str) -> SceneJson {
        let file = File::open(file_path)
            .unwrap_or_else(|err| panic!("failed to open file: {}: \"{}\"", file_path, err));
        serde_json::from_reader(file)
            .unwrap_or_else(|err| panic!("failed to parse file: {}: {}", file_path, err))
    }
}

impl SphereJson {
    fn into_sphere(self, materials: &HashMap<String, MaterialJson>) -> Sphere {
        Sphere::new(self.center, self.radius, materials[&self.material])
    }
}

impl PlaneJson {
    fn into_plane(self, materials: &HashMap<String, MaterialJson>) -> Plane {
        Plane::new(self.point, self.normal, materials[&self.material])
    }
}

impl DiskJson {
    fn into_disk(self, materials: &HashMap<String, MaterialJson>) -> Disk {
        Disk::new(
            self.center,
            self.normal,
            self.radius,
            materials[&self.material],
        )
    }
}

impl CheckBoardDiskJson {
    fn into_checkboard_disk(self, materials: &HashMap<String, MaterialJson>) -> CheckBoardDisk {
        CheckBoardDisk::new(
            self.center,
            self.normal,
            self.radius,
            self.dist_between_mats,
            materials[&self.material1],
            materials[&self.material2],
        )
    }
}

impl PolygonJson {
    fn into_polygon(self, materials: &HashMap<String, MaterialJson>) -> Polygon {
        Polygon::new(
            self.vertex_0,
            self.vertex_1,
            self.vertex_2,
            materials[&self.material],
        )
    }
}

impl ObjJson {
    fn into_mesh(self, materials: &HashMap<String, MaterialJson>) -> Mesh {
        Mesh::from_wavefront_file(&self.wavefront, &materials[&self.material])
            .expect("failed to import mesh")
    }
}
