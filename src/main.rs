#![feature(plugin)]
#![allow(unstable)]
#![allow(dead_code)]
#![allow(unused_variables)]


extern crate glutin;
extern crate glium;
#[plugin]
extern crate glium_macros;

use std::io::timer;
use std::time::duration::Duration;

use std::io;
use std::vec::Vec;
use std::path::Path;

use glium::{Display, Frame, VertexBuffer};
use glium::index_buffer::{IndexBuffer, TrianglesList};
use glium::vertex;
use glium::vertex::AttributeType;

struct SkeletonObjectAttachment {
    sobject:        SkeletonObject,
    l:              [f64; 3],
}

#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position:   [f32; 3],
    texcoords:  [f32; 2],
}

struct SkeletonObject {
    l:              [f64; 3],
    r:              [f64; 3],
    vertex:         VertexBuffer<Vertex>,
    attachment:     Vec<SkeletonObjectAttachment>
}

struct DrawableObject {
    name:           String,
    vbuf:           VertexBuffer<Vertex>,
    tlst:           IndexBuffer,
}

impl DrawableObject {
    pub fn draw(frame: &Frame) {

    }

    /// Read file in `obj` format and return a new `Mesh` object.
    pub fn new_fromobj(display: &Display, source: &str) -> Vec<DrawableObject> {
        let psource = Path::new(source);
        let mut file = io::File::open_mode(&psource, io::Open, io::Read).unwrap();
        let data = file.read_to_string().unwrap();
        let slice = data.as_slice();
        let objects: Vec<DrawableObject> = Vec::new();
        let mut lines = data.lines();
        let mut object: Option<DrawableObject> = Option::None; 
        let mut name: Option<String> = Option::None;
        let mut tlst: Vec<u16> = Vec::new();
        let mut vbuf: Vec<Vertex> = Vec::new();
        for line in lines {
            let mut parts = line.split_str(" ");
            let first = parts.next().unwrap();
            if first.eq("o") {
                if name.is_some() {
                    object = Option::Some(DrawableObject {
                        name:   name.unwrap(),
                        vbuf:   VertexBuffer::new(display, vbuf),
                        tlst:   IndexBuffer::new(display, TrianglesList(tlst)),
                    });
                    tlst = Vec::new();
                    vbuf = Vec::new();
                }
                name = Option::Some(String::from_str(parts.next().unwrap()));
                continue;
            }

            if first.eq("v") && object.is_some() {
                let obj = object.expect("No Object Found Yet");
                let v = [
                    parts.next().unwrap().parse::<f64>().unwrap(),
                    parts.next().unwrap().parse::<f64>().unwrap(),
                    parts.next().unwrap().parse::<f64>().unwrap(),
                ];
                vbuf.push(Vertex {
                    position:    [v[0] as f32, v[1] as f32, v[2] as f32],
                    texcoords:   [0.0, 0.0],
                });
                object = Option::Some(obj);
                continue;
            }

            if first.eq("f") && object.is_some() {
                let obj = object.expect("No Object Found Yet");
                let a = parts.next().unwrap().parse::<u16>().unwrap();
                let b = parts.next().unwrap().parse::<u16>().unwrap();
                let c = parts.next().unwrap().parse::<u16>().unwrap();
                let d = parts.next();
                if d.is_some() {
                    let d = parts.next().unwrap().parse::<u16>().unwrap();
                    // quad
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
                object = Option::Some(obj);
                continue;
            }
        }

        objects
    }
}

fn main() {
    use glium::DisplayBuild;
    use glium::Surface;

    let display = glutin::WindowBuilder::new()
        .with_dimensions(360, 360)
        .with_title(format!("Hello World"))
        .build_glium().unwrap();

    DrawableObject::new_fromobj(&display, "test.obj");

    if true {
        return;
    }

    loop {
        for event in display.poll_events() {
            println!("event {:?}", event);
        }

        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.finish();

        std::io::timer::sleep(Duration::milliseconds(17));
    }
}
