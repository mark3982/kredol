#![feature(plugin)]
#![allow(unstable)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate cgmath;
extern crate glutin;
extern crate glium;
#[plugin]
extern crate glium_macros;

use std::io::timer;
use std::time::duration::Duration;

use std::io;
use std::vec::Vec;
use std::path::Path;
use std::sync::Arc;

use glium::{Display, Frame, VertexBuffer, Surface};
use glium::index_buffer::{IndexBuffer, TrianglesList};
use glium::program::Program;

use cgmath::{Point3, Vector3, Matrix4, Basis3, Matrix3, Quaternion, PerspectiveFov};
use cgmath::ToMatrix4;

/// Represents an attachment of one object to another.
struct SkeletonObjectAttachment {
    sobject:        SkeletonObject,
    /// location offset from parent object's center + location
    l:              Vector3<f64>,
}

#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position:   [f32; 3],
    color:      [f32; 3],
}

#[uniforms]
struct Uniforms {
    matrix:     [[f32; 4]; 4],
}

/// Contains state data for game calculations and GL data rendering. The
/// GL data must be updated from the game state to synchronize the rendering
/// with the game state.
struct SkeletonObject {
    /// local location
    l:              Vector3<f64>,
    /// local rotation
    r:              Vector3<f64>,
    /// local scale
    s:              Vector3<f64>,
    vertex:         VertexBuffer<Vertex>,
    attachment:     Vec<SkeletonObjectAttachment>,
    /// opengl drawable data
    drawable:       DrawableObject,
}

/// An object used directly by the engine to render an object.
struct DrawableObject {
    name:           String,
    vbuf:           VertexBuffer<Vertex>,
    tlst:           IndexBuffer,
    uniforms:       Uniforms,
    programs:       Arc<Program>,
}

impl DrawableObject {
    pub fn draw(&self, frame: &mut Frame) {
        use glium::Surface;
        frame.draw(&self.vbuf, &self.tlst, &*self.programs, &self.uniforms, &std::default::Default::default()).unwrap();
    }

    /// Read file in `obj` format and return a new `Mesh` object.
    pub fn new_fromobj(display: &Display, source: &str, programs: &Arc<Program>) -> Vec<DrawableObject> {
        let psource = Path::new(source);
        let mut file = io::File::open_mode(&psource, io::Open, io::Read).unwrap();
        let data = file.read_to_string().unwrap();
        let slice = data.as_slice();
        let mut objects: Vec<DrawableObject> = Vec::new();
        let mut lines = data.lines();
        let mut name: Option<String> = Option::None;
        let mut tlst: Vec<u16> = Vec::new();
        let mut vbuf: Vec<Vertex> = Vec::new();
        for line in lines {
            let mut parts = line.split_str(" ");
            let first = parts.next().unwrap();
            if first.eq("o") {
                if name.is_some() {
                    objects.push(DrawableObject {
                        name:     name.unwrap(),
                        vbuf:     VertexBuffer::new(display, vbuf),
                        tlst:     IndexBuffer::new(display, TrianglesList(tlst)),
                        uniforms: Uniforms { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] },
                        programs: programs.clone(),
                    });
                    tlst = Vec::new();
                    vbuf = Vec::new();
                }
                name = Option::Some(String::from_str(parts.next().unwrap()));
                continue;
            }

            if first.eq("v") {
                let v = [
                    parts.next().unwrap().parse::<f64>().unwrap(),
                    parts.next().unwrap().parse::<f64>().unwrap(),
                    parts.next().unwrap().parse::<f64>().unwrap(),
                ];
                vbuf.push(Vertex {
                    position:    [v[0] as f32, v[1] as f32, v[2] as f32],
                    color:       [1.0, 1.0, 1.0],
                });
                continue;
            }

            if first.eq("f") {
                let a = parts.next().unwrap().parse::<u16>().unwrap() - 1;
                let b = parts.next().unwrap().parse::<u16>().unwrap() - 1;
                let c = parts.next().unwrap().parse::<u16>().unwrap() - 1;
                let d = parts.next();
                if d.is_some() {
                    let d = d.unwrap().parse::<u16>().unwrap() - 1;
                    // quad (which we turn into two triangles)
                    tlst.push(a);
                    tlst.push(b);
                    tlst.push(c);
                    tlst.push(a);
                    tlst.push(c);
                    tlst.push(d);
                } else {
                    // triangle
                    tlst.push(a);
                    tlst.push(b);
                    tlst.push(c);
                }
                continue;
            }
        }

        objects.push(DrawableObject {
            name:     name.unwrap(),
            vbuf:     VertexBuffer::new(display, vbuf),
            tlst:     IndexBuffer::new(display, TrianglesList(tlst)),
            uniforms: Uniforms { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] },
            programs: programs.clone(),
        });

        objects
    }
}

fn main() {
    use glium::DisplayBuild;
    use glium::Surface;
    use cgmath::FixedArray;
    use cgmath::BaseFloat;
    use cgmath::Angle;

    let display = glutin::WindowBuilder::new()
        .with_dimensions(360, 360)
        .with_title(format!("Hello World"))
        .with_depth_buffer(32)
        .build_glium().unwrap();

    let program = Arc::new(glium::Program::from_source(&display,
        // vertex shader
        "   #version 110

            uniform mat4 matrix;

            attribute vec3 position;
            attribute vec3 color;

            varying vec3 v_color;

            void main() {
                gl_Position = vec4(position, 1.0) * matrix;
                v_color = color;
            }
        ",

        // fragment shader
        "   #version 110
            varying vec3 v_color;

            void main() {
                gl_FragColor = vec4(v_color, 1.0);
            }
        ",

        // optional geometry shader
        None
    ).unwrap());

    let mut objects = DrawableObject::new_fromobj(&display, "test.obj", &program);

    let mut r3: Quaternion<f32> = Quaternion::from_sv(0.0, Vector3::new(0.0, 1.0, 0.0));

    loop {
        for event in display.poll_events() {
            //println!("event {:?}", event);
        }

        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        objects[0].draw(&mut target);

        r3.s += -0.01;

        let m4: Matrix4<f32> = Matrix4::identity();
        //let m4 = m4 * r3.to_matrix4() * Matrix4::from_translation(&Vector3::new(0.0, 0.0, 1.0));
        let m4 = m4 * Matrix4::from_translation(&Vector3::new(-20.0, 0.0, 0.0));
        let per = (PerspectiveFov { fovy: 45.0, aspect: 1.0, near: 0.1, far: 10.0 }).to_matrix4();

        objects[0].uniforms.matrix = m4.into_fixed();
        target.finish();

        std::io::timer::sleep(Duration::milliseconds(17));
    }
}
