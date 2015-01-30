//! 
//! The drawable object can be drawn by the engine to the screen. It may or may not be visible but it
//! does have the capabilities. For example it could be hidden. Also by drawable it means that most of
//! the work, at least, has been done to move the data into the graphics card memory. This is, at the
//! moment, the most optimized and flexible form of a object that can be drawn to the screen.
//!
//! _To manipulate the object you need to set a 4x4 transformation matrix, which at the moment, includes
//! the the perspective transformation. (potentially outdated information)_

use glium::{Display, Frame, VertexBuffer, Surface, DrawParameters};
use glium::index_buffer::{IndexBuffer, TrianglesList};
use glium::program::Program;

use std::sync::Arc;

use super::Vertex;
use super::Uniform;

use glium;

use simplescene::SimpleSceneFile;

/// An object used directly by the engine to render an object.
pub struct DrawableObject {
    name:               String,
    vbuf:               VertexBuffer<Vertex>,
    tlst:               IndexBuffer,
    uniform:            Uniform,
    program:            Arc<Program>,
}

impl DrawableObject {
    pub fn set_uniform_matrix(&mut self, m: [[f32;4];4]) {
        self.uniform.matrix = m;
    }

    pub fn get_uniform_matrix(&self) -> [[f32;4];4] {
        self.uniform.matrix
    }

    pub fn draw(&self, frame: &mut Frame) {
        use glium::Surface;
        use std::default::Default;

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
            .. Default::default()
        };

        frame.draw(&self.vbuf, &self.tlst, &*self.program, &self.uniform, &cfg).unwrap();
    }

    pub fn from_simplescene(display: &Display, scene: &SimpleSceneFile, name: &str, program: Arc<Program>) -> Option<DrawableObject> {
        let found = scene.find(name);

        if found.is_none() {
            Option::None
        } else {
            let found = found.unwrap().lock().unwrap();
            let mut vindex: Vec<u16> = Vec::new();
            let mut vertices: Vec<Vertex> = Vec::new();

            for t in found.triangles.iter() {
                vindex.push(t[2]);
                vindex.push(t[1]);
                vindex.push(t[0]);
            }

            for t in found.quads.iter() {
                vindex.push(t[2]);
                vindex.push(t[1]);
                vindex.push(t[0]);
                vindex.push(t[3]);
                vindex.push(t[2]);
                vindex.push(t[0]);
            }

            for v in found.vertices.iter() {
                vertices.push(Vertex {
                    position:   [v.x, v.y, v.z],
                    color:      [1.0, 1.0, 1.0],
                });
            }

            Option::Some(DrawableObject {
                name:     String::from_str(name),
                vbuf:     VertexBuffer::new(display, vertices),
                tlst:     IndexBuffer::new(display, TrianglesList(vindex)),
                uniform:  Uniform { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] },
                program:  program,
            })
        }
    }

    /// Read file in `obj` format and return a new `Mesh` object.
    pub fn from_obj(display: &Display, source: &str, program: Arc<Program>) -> Vec<DrawableObject> {
        use std::old_io::{File, Open, Read};

        let psource = Path::new(source);
        let mut file = File::open_mode(&psource, Open, Read).unwrap();
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
                        uniform:  Uniform { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] },
                        program:  program.clone(),
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
            uniform:  Uniform { matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] },
            program:  program,
        });

        objects
    }
}
