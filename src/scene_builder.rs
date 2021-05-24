use std::collections::HashMap;
use std::fs::File;

use cgmath::Vector3;
use serde::{Deserialize, Serialize};

use crate::light::Light;
use crate::shapes::material::Material;

use crate::shapes::checkboard_disk::CheckBoardDisk;
use crate::shapes::disk::Disk;
use crate::shapes::plane::Plane;
use crate::shapes::shape::Shape;
use crate::shapes::sphere::Sphere;

pub struct Scene {
    pub materials: HashMap<String, Material>,
    pub lights: Vec<Light>,
    pub shapes: Vec<Box<dyn Shape>>,
}

impl Scene {
    pub fn from_file(file_path: &str) -> Self {
        println!("importing scene from {}", file_path);
        let scene_json = SceneJson::from_file(file_path);
        let shapes = scene_json
            .shapes
            .spheres
            .iter()
            .map(|sphere| {
                Box::new(sphere.clone().into_sphere(&scene_json.materials)) as Box<dyn Shape>
            })
            .chain(scene_json.shapes.planes.iter().map(|plane| {
                Box::new(plane.clone().into_plane(&scene_json.materials)) as Box<dyn Shape>
            }))
            .chain(scene_json.shapes.disks.iter().map(|disk| {
                Box::new(disk.clone().into_disk(&scene_json.materials)) as Box<dyn Shape>
            }))
            .chain(scene_json.shapes.checkboard_disks.iter().map(|disk| {
                Box::new(disk.clone().into_checkboard_disk(&scene_json.materials)) as Box<dyn Shape>
            }))
            .collect();
        Self {
            materials: scene_json.materials,
            lights: scene_json.lights,
            shapes,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SceneJson {
    pub materials: HashMap<String, MaterialJson>,
    pub lights: Vec<LightJson>,
    pub shapes: ShapesJson,
}

#[derive(Serialize, Deserialize)]
struct ShapesJson {
    pub spheres: Vec<SphereJson>,
    pub planes: Vec<PlaneJson>,
    pub disks: Vec<DiskJson>,
    pub checkboard_disks: Vec<CheckBoardDiskJson>,
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

impl SceneJson {
    fn from_file(file_path: &str) -> SceneJson {
        let file =
            File::open(file_path).unwrap_or_else(|_| panic!("failed to open file: {}", file_path));
        serde_json::from_reader(file)
            .unwrap_or_else(|_| panic!("failed to parse file: {}", file_path))
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
