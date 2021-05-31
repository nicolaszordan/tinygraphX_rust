use crate::shapes::material::Material;
use crate::shapes::polygon::Polygon;
use crate::shapes::shape::{Ray, RayHit, Shape};
use crate::wavefront::Obj;

use cgmath::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    polygons: Vec<Polygon>,
    bounding_min: Vector3<f32>,
    bounding_max: Vector3<f32>,
    material: Material,

    //
    bb_miss: u32,
    mesh_miss: u32,
    mesh_hit: u32,
}

impl Mesh {
    pub fn from_wavefront_file(
        file_name: &str,
        material: &Material,
    ) -> Result<Mesh, std::io::Error> {
        let obj = Obj::from_file(file_name)?;
        let polygons: Vec<Polygon> = obj
            .faces
            .iter()
            .map(|face| {
                Polygon::new(
                    obj.vertexes[face[0] - 1],
                    obj.vertexes[face[1] - 1],
                    obj.vertexes[face[2] - 1],
                    *material,
                )
            })
            .collect();

        let (bounding_min, bounding_max) = Mesh::min_max_vertexes(&obj.vertexes);

        Ok(Mesh {
            polygons,
            bounding_min,
            bounding_max,
            material: *material,
            bb_miss: 0,
            mesh_miss: 0,
            mesh_hit: 0,
        })
    }

    fn min_max_vertexes(vertexes: &[Vector3<f32>]) -> (Vector3<f32>, Vector3<f32>) {
        let ((min_x, max_x), (min_y, max_y), (min_z, max_z)) = vertexes
            .iter()
            .map(|vertex| (vertex.x, vertex.y, vertex.z))
            .fold(
                (
                    (vertexes[0].x, vertexes[0].x),
                    (vertexes[0].y, vertexes[0].y),
                    (vertexes[0].z, vertexes[0].z),
                ),
                |((min_x, max_x), (min_y, max_y), (min_z, max_z)), (x, y, z)| {
                    (
                        (min_x.min(x), max_x.max(x)),
                        (min_y.min(y), max_y.max(y)),
                        (min_z.min(z), max_z.max(z)),
                    )
                },
            );

        (
            Vector3::new(min_x, min_y, min_z),
            Vector3::new(max_x, max_y, max_z),
        )
    }

    fn is_ray_intersecting_with_bounding_box(&self, ray: &Ray) -> bool {
        let tx_min = (self.bounding_min.x - ray.origin.x) * ray.inv_direction.x;
        let tx_max = (self.bounding_max.x - ray.origin.x) * ray.inv_direction.x;
        let ty_min = (self.bounding_min.y - ray.origin.y) * ray.inv_direction.y;
        let ty_max = (self.bounding_max.y - ray.origin.y) * ray.inv_direction.y;
        let tz_min = (self.bounding_min.z - ray.origin.z) * ray.inv_direction.z;
        let tz_max = (self.bounding_max.z - ray.origin.z) * ray.inv_direction.z;

        let tmin = tx_min
            .min(tx_max)
            .max(ty_min.min(ty_max))
            .max(tz_min.min(tz_max));
        let tmax = tx_min
            .max(tx_max)
            .min(ty_min.max(ty_max))
            .min(tz_min.max(tz_max));

        if tmax < 0.0 || tmin > tmax {
            false
        } else {
            true
        }
    }
}

impl Shape for Mesh {
    fn ray_intersect(&self, ray: &Ray) -> Option<RayHit> {
        // check intersect with bounding box
        if !self.is_ray_intersecting_with_bounding_box(ray) {
            return None;
        }

        // check intersect with polygons
        self.polygons
            .iter()
            .filter_map(|polygon| polygon.ray_intersect(ray))
            .min_by(|ray_hit_1, ray_hit_2| {
                ray_hit_1
                    .hit_dist
                    .partial_cmp(&ray_hit_2.hit_dist)
                    .expect("tried to compare to NaN")
            })
    }
}
