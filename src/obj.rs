use std::fs::read_to_string;
use vek::Vec3 as TVec3;

type Vec3 = TVec3<f32>;

#[repr(C)]
pub struct Tri {
    pub a: Vec3,
    pub _a: f32,
    pub b: Vec3,
    pub _b: f32,
    pub c: Vec3,
    pub _c: f32,

    pub an: Vec3,
    pub _d: f32,
    pub bn: Vec3,
    pub _e: f32,
    pub cn: Vec3,
    pub _f: f32,
}
impl Tri {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, an: Vec3, bn: Vec3, cn: Vec3) -> Self {
        Self { a, _a: 0.0, b, _b: 0.0, c, _c: 0.0, an, _d: 0.0, bn, _e: 0.0, cn, _f: 0.0 }
    }
}

pub struct Obj {
    pub tris: Vec<Tri>,
    pub pos: Vec3,
    pub rot: Vec3,
    pub scale: f32
}
impl Obj {
    pub fn load_from_file(path: &str, pos: Vec3, rot: Vec3, scale: f32) -> Self {
        let mut tris = Vec::new();
        let mut normals = vec![];
        let mut positions = vec![];
        let file = read_to_string(path).unwrap();
        for x in file.split('\n') {
            if x.len() == 0 {
                continue;
            }
            if x.starts_with('#') {
                continue;
            }
            if x.starts_with("vn") {
                let mut v = Vec3::new(0.0, 0.0, 0.0);
                for (i, c) in x.split_ascii_whitespace().skip(1).enumerate() {
                    v[i] = c.parse::<f32>().unwrap();
                }
                normals.push(v.normalized());
            }
            if x.starts_with('v') && !x.contains("vt") {
                let mut v = Vec3::new(0.0, 0.0, 0.0);
                for (i, c) in x.split_ascii_whitespace().skip(1).enumerate() {
                    v[i] = c.parse::<f32>().unwrap();
                }
                positions.push(v);
            }
            if x.starts_with('f') {
                if x.split_ascii_whitespace().skip(1).next().unwrap().chars().fold(0, |c, x| {if x == '/' {c+1} else {c}})>1 {
                    let mut f: Vec<Vertex> = x.split_ascii_whitespace().skip(1).map(|f| f.split('/').map(|x| x.parse::<usize>().unwrap()).collect()).map(|x: Vec<_>| Vertex{ pos: positions[x[0]-1], norm: normals[x[2]-1], }).collect();
                    if f.len()==3 {
                        let t = Tri::new(f[0].pos, f[1].pos, f[2].pos, f[0].norm, f[1].norm, f[2].norm);
                        tris.push(t);
                    } else if f.len()==4 {
                        let t = Tri::new(f[2].pos, f[3].pos, f[0].pos, f[2].norm, f[3].norm, f[0].norm);
                        let t2 = Tri::new(f[0].pos, f[1].pos, f[2].pos, f[0].norm, f[1].norm, f[2].norm);
                        tris.push(t);
                        tris.push(t2);
                    }
                } else {
                    unimplemented!();
                    // let mut f: Vec<Vertex> = x.split_ascii_whitespace().skip(1).map(|f| f.split('/').map(|x| x.parse::<usize>().unwrap()).collect()).map(|x: Vec<_>| Vertex{ pos: positions[x[0]-1], norm: normals[x[2]-1], }).collect();
                    // if f.len()==3 {
                        // let t = Tri::new(f[0].pos, f[1].pos, f[2].pos, f[0].norm, f[1].norm, f[2].norm);
                        // f.iter_mut().for_each(|x| {x.norm = t.normal().into()});
                        // tris.push(t);
                    // } else if f.len()==4 {
                        // let t = Tri::new(f[2].pos, f[3].pos, f[0].pos, f[2].norm, f[3].norm, f[0].norm);
                        // let t2 = Tri::new(f[0].pos, f[1].pos, f[2].pos, f[0].norm, f[1].norm, f[2].norm);
                        // f.iter_mut().for_each(|x| {x.norm = t.normal().into()});
                        // tris.push(t);
                        // tris.push(t2);
                        // tris.extend([f[2], f[3], f[0]]);
                    // }
                }
            }
        }
        // let tris: Vec<_> = tris.iter().map(|x| {[x.pos, x.norm]}).flatten().map(|x| {[x.x, x.y, x.z]}).flatten().collect();

        Self { tris, pos, rot, scale }
    }
    // pub fn get_model_world_matrix(&self) -> Matrix4<f32> {
    //     use crate::camera::{cos, sin};
    //     let cx=cos(self.rot[0]);
    //     let sx=sin(self.rot[0]);
    //     let cy=cos(self.rot[1]);
    //     let sy=sin(self.rot[1]);
    //     let cz=cos(self.rot[2]);
    //     let sz=sin(self.rot[2]);

    //     Matrix4::new(
    //         1.0, 0.0, 0.0, self.pos.x,
    //         0.0, 1.0, 0.0, self.pos.y,
    //         0.0, 0.0, 1.0, self.pos.z,
    //         0.0, 0.0, 0.0, 1.0
    //     )*
    //     Matrix4::new(
    //         1.0, 0.0, 0.0, 0.0,
    //         0.0, 1.0, 0.0, 0.0,
    //         0.0, 0.0, 1.0, 0.0,
    //         0.0, 0.0, 0.0, 1.0/self.scale
    //     )*
    //     Matrix4::new(
    //         cz, -sz, 0.0, 0.0,
    //         sz, cz, 0.0, 0.0,
    //         0.0, 0.0, 1.0, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )*
    //     Matrix4::new(
    //         cy, 0.0, sy, 0.0,
    //         0.0, 1.0, 0.0, 0.0,
    //         -sy, 0.0, cy, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )*
    //     Matrix4::new(
    //         1.0, 0.0, 0.0, 0.0,
    //         0.0, cx, -sx, 0.0,
    //         0.0, sx, cx, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )
    // }
    // pub fn get_model_rotation_matrix(&self) -> Matrix4<f32> {
    //     use crate::camera::{cos, sin};
    //     let cx=cos(self.rot[0]);
    //     let sx=sin(self.rot[0]);
    //     let cy=cos(self.rot[1]);
    //     let sy=sin(self.rot[1]);
    //     let cz=cos(self.rot[2]);
    //     let sz=sin(self.rot[2]);
    //     Matrix4::new(
    //         cz, -sz, 0.0, 0.0,
    //         sz, cz, 0.0, 0.0,
    //         0.0, 0.0, 1.0, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )*
    //     Matrix4::new(
    //         cy, 0.0, sy, 0.0,
    //         0.0, 1.0, 0.0, 0.0,
    //         -sy, 0.0, cy, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )*
    //     Matrix4::new(
    //         1.0, 0.0, 0.0, 0.0,
    //         0.0, cx, -sx, 0.0,
    //         0.0, sx, cx, 0.0,
    //         0.0, 0.0, 0.0, 1.0
    //     )
    // }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub norm: Vec3,
}