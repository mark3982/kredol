use cgmath;

pub struct SimpleSceneObject {
    vertices:       Vec<cgmath::Vector3<f32>>,
    scale:          cgmath::Vector3<f32>,
    triangles:      Vec<[u16;3]>,
    quads:          Vec<[u16;4]>,
    name:           String,
    typ:            String,
    groups:         Vec<String>,
    location:       cgmath::Vector3<f32>,
    rotation:       cgmath::Quaternion<f32>,
}

pub struct SimpleSceneFile {
    path:           Path,
    objects:        Vec<SimpleSceneObject>,
}

impl SimpleSceneFile {
    pub fn find(&self, name: &str) -> Option<&SimpleSceneObject> {
        for obj in self.objects.iter() {
            if obj.name.as_slice().eq(name) {
                return Option::Some(obj);
            }
        }

        Option::None
    }

    pub fn from_file(source: &str) -> SimpleSceneFile {
        use cgmath::ToRad;
        use std::str::Lines;
        use std::mem::transmute_copy;
        use std::old_io::{File, Open, Read};

        let path = Path::new(source);
        let mut file = File::open_mode(&path, Open, Read).unwrap();
        let data = file.read_to_string().unwrap();
        let mut lines: Lines = data.lines();
        let lines_a: &mut Lines = &mut lines;
        let lines_b: &mut Lines = unsafe { transmute_copy(&lines_a) }; 
        let lines_c: &mut Lines = unsafe { transmute_copy(&lines_a) }; 

        let mut objects: Vec<SimpleSceneObject> = Vec::new();

        for line in *lines_a {
            let mut parts = line.split_str(" ");
            match parts.next().unwrap() {
                "start" => match parts.next().unwrap() {
                    "object" => {
                        let mut object = SimpleSceneObject {
                            vertices:   Vec::new(),
                            triangles:  Vec::new(),
                            quads:      Vec::new(),
                            name:       String::new(),
                            typ:        String::new(),
                            groups:     Vec::new(),
                            scale:      cgmath::Vector3::new(0.0, 0.0, 0.0),
                            location:   cgmath::Vector3::new(0.0, 0.0, 0.0),
                            rotation:   cgmath::Quaternion::from_sv(0.0, cgmath::Vector3::new(0.0, 0.0, 0.0)),
                        };
                        for line in *lines_b {
                            let mut parts = line.split_str(" ");
                            match parts.next().unwrap() {
                                "location" => object.location = cgmath::Vector3::new(
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                ),
                                "scale" => object.scale = cgmath::Vector3::new(
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                    parts.next().unwrap().parse::<f32>().unwrap(),
                                ),
                                "rotation" => object.rotation = cgmath::Rotation3::<f32>::from_axis_angle(
                                    &cgmath::Vector3::new(
                                        parts.next().unwrap().parse::<f32>().unwrap(),
                                        parts.next().unwrap().parse::<f32>().unwrap(),
                                        parts.next().unwrap().parse::<f32>().unwrap()
                                    ),
                                    (cgmath::Deg { s: parts.next().unwrap().parse::<f32>().unwrap() }).to_rad()
                                ),
                                "type" => object.typ = String::from_str(parts.next().unwrap()),
                                "name" => object.name = String::from_str(parts.next().unwrap()),
                                "end" => break,
                                "start" => match parts.next().unwrap() {
                                    "vertex" => {
                                        for line in *lines_c {
                                            if line.eq("end vertex") {
                                                break;
                                            }
                                            let mut parts = line.split_str(" ");
                                            object.vertices.push(cgmath::Vector3::new(
                                                parts.next().unwrap().parse::<f32>().unwrap(),
                                                parts.next().unwrap().parse::<f32>().unwrap(),
                                                parts.next().unwrap().parse::<f32>().unwrap()
                                            ));
                                        }
                                    },
                                    "polygon" => {
                                        for line in *lines_c {
                                            if line.eq("end polygon") {
                                                break;
                                            }
                                            let mut parts = line.split_str(" ");
                                            match parts.size_hint().0 {
                                                3 => object.triangles.push([
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                ]),
                                                4 => object.quads.push([
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                    parts.next().unwrap().parse::<u16>().unwrap(),
                                                ]),
                                                _ => panic!("unsupported polygon type"),
                                            }
                                        }
                                    },
                                    "group" => {
                                        for line in *lines_c {
                                            if line.eq("end group") {
                                                break;
                                            }
                                            let mut parts = line.split_str(" ");
                                            object.groups.push(String::from_str(parts.next().unwrap()));
                                        }
                                    },
                                    _ => continue,
                                },
                                _ => continue,
                            }
                        }

                        objects.push(object);
                    },
                    _ => continue,
                },
                _ => continue,
            }
        }

        SimpleSceneFile {
            path:           path,
            objects:        objects,
        }
    }
}
