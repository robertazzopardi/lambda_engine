use super::WHITE;
use lambda_space::space::{Indices, Pos3, Vertex, Vertices};
use nalgebra::{vector, Matrix4, Point3, Vector2};
use std::{
    collections::HashMap,
    ops::{Mul, Sub},
};

pub trait Transformation {
    fn rotate_x(&mut self, angle: f32);
    fn rotate_y(&mut self, angle: f32);
    fn rotate_z(&mut self, angle: f32);
    fn translate_x(&mut self, distance: f32);
    fn translate_y(&mut self, distance: f32);
    fn translate_z(&mut self, distance: f32);
}

#[inline]
pub fn scaled_axis_matrix_4(axis: Pos3, angle: f32) -> Matrix4<f32> {
    Matrix4::from_scaled_axis(*axis * angle)
}

#[inline]
pub fn translate(distance: Pos3) -> Matrix4<f32> {
    Matrix4::new_translation(&distance)
}

pub(crate) fn scale(model: &mut [Vertex], radius: f32) {
    model.iter_mut().for_each(|face| {
        face.pos = face.pos.mul(radius);
    });
}

pub(crate) fn calculate_normals(model: &mut [Vertex]) {
    let normal = normal(model[0].pos, model[1].pos, model[2].pos);

    model.iter_mut().for_each(|point| {
        point.normal = normal.coords;
    });
}

fn normal(p1: Point3<f32>, p2: Point3<f32>, p3: Point3<f32>) -> Point3<f32> {
    let a = p3.sub(p2);
    let b = p1.sub(p2);
    Point3::from(a.cross(&b))
}

pub(crate) fn make_point(
    angle: &mut f32,
    radius: f32,
    step: f32,
    length: f32,
    tex_coord: Vector2<f32>,
    pos: &Pos3,
) -> Vertex {
    let x = (angle.to_radians().cos() * radius) + pos.x;
    let y = (angle.to_radians().sin() * radius) + pos.y;

    *angle += step;

    let pos = vector![x, y, pos.z];

    Vertex::new(pos.into(), WHITE, pos.mul(length), tex_coord)
}

pub(crate) fn calculate_indices(vertices: &Vertices) -> Indices {
    let mut unique_vertices: HashMap<String, u16> = HashMap::new();
    let mut indices = Vec::new();
    let mut v = Vec::new();

    vertices.iter().for_each(|vertex| {
        let vertex_hash = &format!("{:p}", vertex);

        if !unique_vertices.contains_key(vertex_hash) {
            unique_vertices.insert(vertex_hash.to_string(), v.len() as u16);
            v.push(vertex);
        }

        indices.push(unique_vertices[vertex_hash]);
    });

    indices.into()
}

pub(crate) fn spherical_indices(sector_count: u32, stack_count: u32) -> Indices {
    let mut k1: u32;
    let mut k2: u32;

    let mut indices = Vec::new();

    for i in 0..stack_count {
        k1 = i * (sector_count + 1);
        k2 = k1 + sector_count + 1;

        for _j in 0..sector_count {
            if i != 0 {
                indices.push(k1 as u16);
                indices.push(k2 as u16);
                indices.push(k1 as u16 + 1);
            }

            if i != (stack_count - 1) {
                indices.push(k1 as u16 + 1);
                indices.push(k2 as u16);
                indices.push(k2 as u16 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    indices.into()
}
