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
use drawableobject::DrawableObject;

pub mod simplescene;
pub mod drawableobject;

/// Represents an attachment of one object to another.
struct SkeletonObjectAttachment {
    sobject:        SkeletonObject,
    /// location offset from parent object's center + location
    l:              Vector3<f64>,
}

#[vertex_format]
#[derive(Copy)]
pub struct Vertex {
    position:   [f32; 3],
    color:      [f32; 3],
}

#[uniforms]
pub struct Uniform {
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

    //let mut objects = DrawableObject::from_obj(&display, "test.obj", program.clone());

    let scene = SimpleSceneFile::from_file("data.txt");
    let mut dobject = DrawableObject::from_simplescene(&display, &scene, "Grape", program.clone()).unwrap();

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
        dobject.draw(&mut target);
        target.finish();

        // Will rotate q3a by q3.
        //q3a = q3a.mul_q(&q3b).normalize();

        let r = q3a.to_matrix4();

        let m4: Matrix4<f32> = Matrix4::identity();
        let per = cgmath::perspective(Deg { s: 45.0 }, 1.0, 0.1, 10.0);
        let m4 = m4 * Matrix4::from_translation(&Vector3::new(0.0, 0.0, -5.0));

        dobject.set_uniform_matrix((per * m4 * r).into_fixed());

        std::old_io::timer::sleep(Duration::milliseconds(17));
    }
}
