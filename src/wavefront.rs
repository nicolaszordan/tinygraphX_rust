use std::fs;

use cgmath::Vector3;

// a simple wavefront obj parser
pub struct Obj {
    pub vertexes: Vec<Vector3<f32>>,
    pub faces: Vec<Vector3<usize>>,
}

impl Obj {
    pub fn from_file(file_name: &str) -> Result<Obj, std::io::Error> {
        let contents = fs::read_to_string(file_name)?;
        Ok(Obj::from_string(&contents))
    }

    pub fn from_string(buffer: &str) -> Obj {
        let (vertexes, faces) = buffer
            .lines()
            .filter_map(|line| {
                let words = line.split_ascii_whitespace().collect::<Vec<&str>>();
                if words.len() == 4 {
                    match words[0] {
                        "v" => {
                            let x = words[1].parse().unwrap();
                            let y = words[2].parse().unwrap();
                            let z = words[3].parse().unwrap();
                            Some((Some(Vector3::new(x, y, z)), None))
                        }
                        "f" => {
                            let x = words[1].parse().unwrap();
                            let y = words[2].parse().unwrap();
                            let z = words[3].parse().unwrap();
                            Some((None, Some(Vector3::new(x, y, z))))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .fold(
                (Vec::new(), Vec::new()),
                |(mut acc_v, mut acc_f), (elem_v, elem_f)| {
                    if let Some(elem_v) = elem_v {
                        acc_v.push(elem_v);
                        (acc_v, acc_f)
                    } else {
                        acc_f.push(elem_f.unwrap());
                        (acc_v, acc_f)
                    }
                },
            );

        Obj { vertexes, faces }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_obj_from_string() {
        let obj = Obj::from_string(
            r#"
        v 4.0 -4.0 -9.0 
        v 5.0 -4.0 -9.0 
        v 5.0 -4.0 -8.0 
        f 1 3 2 
        f 2 4 1 
        f 4 5 1 
        "#,
        );

        assert_eq!(
            obj.vertexes,
            vec![
                Vector3::new(4.0, -4.0, -9.0),
                Vector3::new(5.0, -4.0, -9.0),
                Vector3::new(5.0, -4.0, -8.0)
            ]
        );
        assert_eq!(
            obj.faces,
            vec![
                Vector3::new(1, 3, 2),
                Vector3::new(2, 4, 1),
                Vector3::new(4, 5, 1)
            ]
        );
    }
}
