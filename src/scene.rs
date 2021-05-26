use std::collections::HashMap;
use std::fs::File;

use cgmath::Vector3;
use image::io::Reader as ImageReader;
use image::RgbImage;
use serde::{Deserialize, Serialize};

use crate::light::Light;
use crate::shapes::material::Material;
use crate::wavefront::Obj;

use crate::shapes::checkboard_disk::CheckBoardDisk;
use crate::shapes::disk::Disk;
use crate::shapes::plane::Plane;
use crate::shapes::polygon::Polygon;
use crate::shapes::shape::Shape;
use crate::shapes::sphere::Sphere;

pub struct Scene {
    pub materials: HashMap<String, Material>,
    pub lights: Vec<Light>,
    pub shapes: Vec<Box<dyn Shape + Sync>>,
    pub background: RgbImage,
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
                Box::new(disk.clone().into_checkboard_disk(&scene_json.materials)) as Box<dyn Shape + Sync>
            }))
            .chain(scene_json.shapes.polygons.iter().map(|polygon| {
                Box::new(polygon.clone().into_polygon(&scene_json.materials)) as Box<dyn Shape + Sync>
            }))
            .collect();
        println!("importing background: [file={}]", file_path);
        let background = Scene::create_background(&scene_json.background);
        Self {
            materials: scene_json.materials,
            lights: scene_json.lights,
            shapes,
            background,
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
}

#[derive(Serialize, Deserialize)]
struct SceneJson {
    pub materials: HashMap<String, MaterialJson>,
    pub lights: Vec<LightJson>,
    pub shapes: ShapesJson,
    pub background: String,
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
        let mut scene_json: SceneJson = serde_json::from_reader(file)
            .unwrap_or_else(|err| panic!("failed to parse file: {}: {}", file_path, err));
        scene_json.import_objs();
        scene_json
    }

    fn import_objs(&mut self) {
        self.shapes.polygons.extend(
            self.shapes
                .objs
                .iter()
                .map(|obj_json| {
                    let obj = Obj::from_file(&obj_json.wavefront).expect("failed import obj file");
                    obj.faces
                        .iter()
                        .map(|face| PolygonJson {
                            vertex_0: obj.vertexes[face[0] - 1],
                            vertex_1: obj.vertexes[face[1] - 1],
                            vertex_2: obj.vertexes[face[2] - 1],
                            material: obj_json.material.clone(),
                        })
                        .collect::<Vec<PolygonJson>>()
                })
                .flatten(),
        );
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
