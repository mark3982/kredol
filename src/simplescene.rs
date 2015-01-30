//! It is called simple scene because I envisioned it being easy to work with, but still powerful 
//! enough for my needs. It supports loading from the simple scene format. 
//!
//! At this time I create the simple scene format for my custom Blender export addon. It takes
//! the blender scenes and does a rough export of everything. This gives me the power to easily
//! go in and make changes or fixes as needed instead of relying on a third party. I keep the
//! addon very simple, because I hope to move most of the work outside of Blender. I will likely
//! at some point support converting the simple scene format into a binary format instead of its
//! current textual format (you can open, read, and edit using a text editor). The binary format
//! would be capable of being loaded faster and may support partial loading (only loading what
//! is needed), but that is something for the future.
//!
//! If you want to get something loaded from the simple scene into the engine you should checkout
//! the `drawableobject` module and specifically the `DrawableObject` implementation. It currently
//! supports converting a simple scene mesh object into a DrawableObject which can be drawn to the
//! sceen.
//!
//! Also, not finished yet but in the works is the `skeleton` module, which will allow you to load
//! or create skeletons with animations that can be drawn and manipulated on the screen.

use cgmath;
use std::sync::Arc;
use std::sync::Mutex;

/// Represents a single object. The object may or may not have data depending on its type.
pub struct SimpleSceneObject {
    pub vertices:       Vec<cgmath::Vector3<f32>>,
    pub scale:          cgmath::Vector3<f32>,
    pub triangles:      Vec<[u16;3]>,
    pub quads:          Vec<[u16;4]>,
    pub name:           String,
    pub typ:            String,
    pub groups:         Vec<String>,
    pub location:       cgmath::Vector3<f32>,
    pub rotation:       cgmath::Quaternion<f32>,
    pub parent:         Option<Arc<Mutex<SimpleSceneObject>>>,
    pub child:          Vec<Arc<Mutex<SimpleSceneObject>>>,
}

/// Represents any data, and currently only objects, loaded from the scene.
pub struct SimpleSceneFile {
    pub path:           Path,
    pub objects:        Vec<Arc<Mutex<SimpleSceneObject>>>,
}

impl SimpleSceneFile {
    /// Return a reference to a object by it's name. This is the actual object name set in Blender.
    pub fn find(&self, name: &str) -> Option<&Arc<Mutex<SimpleSceneObject>>> {
        for obj in self.objects.iter() {
            if obj.lock().unwrap().name.as_slice().eq(name) {
                return Option::Some(obj);
            }
        }

        Option::None
    }

    /// Return a simple scene instance by loading it from a file source.
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

        let mut objects: Vec<Arc<Mutex<SimpleSceneObject>>> = Vec::new();
        let mut relocs: Vec<(Arc<Mutex<SimpleSceneObject>>, String)> = Vec::new();

        for line in *lines_a {
            let mut parts = line.split_str(" ");
            match parts.next().unwrap() {
                "start" => match parts.next().unwrap() {
                    "object" => {
                        let mut _object = Arc::new(Mutex::new(SimpleSceneObject {
                            vertices:   Vec::new(),
                            triangles:  Vec::new(),
                            quads:      Vec::new(),
                            name:       String::new(),
                            typ:        String::new(),
                            groups:     Vec::new(),
                            scale:      cgmath::Vector3::new(0.0, 0.0, 0.0),
                            location:   cgmath::Vector3::new(0.0, 0.0, 0.0),
                            rotation:   cgmath::Quaternion::from_sv(0.0, cgmath::Vector3::new(0.0, 0.0, 0.0)),
                            parent:     Option::None,
                            child:      Vec::new(),
                        }));
                        {
                            let mut object = _object.lock().unwrap();
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
                                    "parent" => relocs.push((_object.clone(), String::from_str(parts.next().unwrap()))),
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
                        }
                        objects.push(_object);
                    },
                    _ => continue,
                },
                _ => continue,
            }
        }

        let scene = SimpleSceneFile {
            path:           path,
            objects:        objects,
        };

        // Go through and add parent and children objects. For each
        // unresolved parent we resolve it by finding that object.
        // Then we add it as a parent, and add the object needing the
        // reloc as a child to the parent object.
        for pair in relocs.iter() {
            let ref object = pair.0;
            let ref pname = pair.1;
            match scene.find(pname.as_slice()) {
                Some(pobject) => {
                    object.lock().unwrap().parent = Option::Some(pobject.clone());
                    pobject.lock().unwrap().child.push(object.clone());
                }
                None => panic!("could not find parent"),
            }
        }

        scene
    }
}
