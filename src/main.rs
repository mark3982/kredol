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

use std::old_io::timer;
use std::time::duration::Duration;

use std::old_io;
use std::vec::Vec;
use std::path::Path;
use std::sync::Arc;

use glium::{Display, Frame, VertexBuffer, Surface, DrawParameters};
use glium::index_buffer::{IndexBuffer, TrianglesList};
use glium::program::Program;

use cgmath::{Point3, Vector3, Matrix4, Basis3, Matrix3, Quaternion, PerspectiveFov, Deg, Ortho};
use cgmath::ToMatrix4;

use simplescene::SimpleSceneFile;

mod simplescene;

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
        let cfg = DrawParameters {
            depth_function:     glium::DepthFunction::IfLessOrEqual,
            depth_range:        (0.0, 1.0),
            blending_function:  Option::None,
            line_width:         Option::Some(1.0),
            backface_culling:   glium::BackfaceCullingMode::CullClockWise,
            polygon_mode:       glium::PolygonMode::Line,
            multisampling:      false,
            dithering:          false,
            viewport:           Option::None,
            scissor:            Option::None,
        };
        
        let cfg = DrawParameters {
            //depth_function:     glium::DepthFunction::IfLessOrEqual,
            polygon_mode:       glium::PolygonMode::Line,
            .. std::default::Default::default()
        };

        frame.draw(&self.vbuf, &self.tlst, &*self.programs, &self.uniforms, &cfg).unwrap();
    }

    /// Read file in `obj` format and return a new `Mesh` object.
    pub fn new_fromobj(display: &Display, source: &str, programs: &Arc<Program>) -> Vec<DrawableObject> {
        let psource = Path::new(source);
        let mut file = old_io::File::open_mode(&psource, old_io::Open, old_io::Read).unwrap();
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
                let scaler = 0.8;
                vbuf.push(Vertex {
                    position:    [v[0] as f32 * scaler, v[2] as f32 * scaler, v[1] as f32 * scaler],
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
                    tlst.push(c);
                    tlst.push(b);
                    tlst.push(a);
                    tlst.push(d);
                    tlst.push(c);
                    tlst.push(a);
                } else {
                    // triangle
                    tlst.push(c);
                    tlst.push(b);
                    tlst.push(a);
                }
                continue;
            }
        }

        println!("tlst.len():{}", tlst.len());
        println!("vbuf.len():{}", vbuf.len());
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
    use cgmath::ToRad;

    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(32)
        .with_dimensions(360, 360)
        .with_title(format!("Hello World"))
        .build_glium().unwrap();

    let program = Arc::new(glium::Program::from_source(&display,
        // vertex shader
        "   #version 110

            uniform mat4 matrix;

            attribute vec3 position;
            attribute vec3 color;

            varying vec3 v_color;

            void main() {
                gl_Position = matrix * vec4(position, 1.0);
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

    SimpleSceneFile::from_file("data.txt");

    let mut objects = DrawableObject::new_fromobj(&display, "test.obj", &program);

    let mut rv = cgmath::Vector3::new(0.0, 1.0, 0.0);

    let mut q3a: Quaternion<f32> = cgmath::Rotation3::<f32>::from_axis_angle(&cgmath::Vector3::new(0.0, 1.0, 0.0), (cgmath::Deg { s: 22.0 }).to_rad());
    let mut q3b: Quaternion<f32> = cgmath::Rotation3::<f32>::from_axis_angle(&cgmath::Vector3::new(1.0, 0.0, 0.0), (cgmath::Deg { s: 22.0 }).to_rad());
    q3a = q3a.mul_q(&q3b);

    let mut q3b: Quaternion<f32> = cgmath::Rotation3::<f32>::from_axis_angle(&cgmath::Vector3::new(0.0, 0.0, 1.0), (cgmath::Deg { s: 22.0 }).to_rad());
    q3a = q3a.mul_q(&q3b);

    loop {
        for event in display.poll_events() {
            //println!("event {:?}", event);
        }

        let mut target = display.draw();

        target.clear_all((0.0, 0.0, 0.0, 0.0), 0.0, 0);
        objects[0].draw(&mut target);
        target.finish();

        // Will rotate q3a by q3.
        //q3a = q3a.mul_q(&q3b).normalize();

        let r = q3a.to_matrix4();

        let m4: Matrix4<f32> = Matrix4::identity();
        let per = cgmath::perspective(Deg { s: 45.0 }, 1.0, 0.1, 10.0);
        let m4 = m4 * Matrix4::from_translation(&Vector3::new(0.0, 0.0, -5.0));

        objects[0].uniforms.matrix = (per * m4 * r).into_fixed();

        std::old_io::timer::sleep(Duration::milliseconds(17));
    }
}
