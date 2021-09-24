#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
// #![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Pow;

use spirv_std::glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use spirv_std::{Image, Sampler};

pub struct Light {
    positionx: f32,
    positiony: f32,
    positionz: f32,
    _padding: u32,
    colorx: f32,
    colory: f32,
    colorz: f32,
}

pub struct Camera {
    view_position: Vec4,
    view_proj: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec3,
    in_tex: Vec2,
    in_normal: Vec3,
    in_tangent: Vec3,
    in_bitangent: Vec3,
    model0: Vec4,
    model1: Vec4,
    model2: Vec4,
    model3: Vec4,
    normal0: Vec3,
    normal1: Vec3,
    normal2: Vec3,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Camera,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] light: &Light,
    #[spirv(position)] clip_pos: &mut Vec4,
    tangent_pos: &mut Vec3,
    tangent_light_position: &mut Vec3,
    tangent_view_position: &mut Vec3,
    tex_coords: &mut Vec2,
) {
    let light_position = Vec3::new(light.positionx, light.positiony, light.positionz);

    let model = Mat4::from_cols(model0, model1, model2, model3);
    let world_pos = model * Vec4::from((in_pos, 1.0));

    let normal_mat = Mat3::from_cols(normal0, normal1, normal2);

    //construct the tanget matrix
    let world_normal = (normal_mat * in_normal).normalize();
    let world_tangent = (normal_mat * in_tangent).normalize();
    let world_bitangent = (normal_mat * in_bitangent).normalize();

    let tangent_mat = Mat3::from_cols(world_tangent, world_bitangent, world_normal);

    *clip_pos = camera.view_proj * world_pos;
    *tex_coords = in_tex;

    *tangent_pos = tangent_mat * world_pos.truncate();
    *tangent_view_position = tangent_mat * camera.view_position.truncate();
    *tangent_light_position = tangent_mat * light_position;
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(descriptor_set = 0, binding = 0)] image2d: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] image2d_normal: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 3)] sampler_normal: &Sampler,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Camera,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] light: &Light,
    tangent_pos: Vec3,
    tangent_light_position: Vec3,
    tangent_view_position: Vec3,
    tex_coords: Vec2,
    output: &mut Vec4,
) {
    let light_position = Vec3::new(light.positionx, light.positiony, light.positionz);
    let light_color = Vec3::new(light.colorx, light.colory, light.colorz);

    let obj_color: Vec4 = image2d.sample(*sampler, tex_coords);
    let a = obj_color.w;

    let obj_normal: Vec4 = image2d_normal.sample(*sampler_normal, tex_coords);

    let ambient_strength = 0.1;
    let ambient_color = light_color * ambient_strength;

    let tangent_normal = (obj_normal.truncate() * 2.0 - 1.0).normalize();
    let light_dir = (tangent_light_position - tangent_pos).normalize();

    let diffuse_strength = tangent_normal.dot(light_dir).max(0.0);
    let diffuse_color = light_color * diffuse_strength;

    // Specular
    let view_dir = (tangent_view_position - tangent_pos).normalize();
    let half_dir = (view_dir + light_dir).normalize();

    let specular_strength = tangent_normal.dot(half_dir).max(0.0).pow(32.0);
    let specular_color = specular_strength * light_color;

    let result = (ambient_color + diffuse_color + specular_color) * obj_color.truncate();

    *output = Vec4::new(result.x, result.y, result.z, a);
}

#[spirv(vertex)]
pub fn light_vs(
    in_pos: Vec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera: &Camera,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] light: &Light,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    out_color: &mut Vec3,
) {
    let light_position = Vec3::new(light.positionx, light.positiony, light.positionz);
    let light_color = Vec3::new(light.colorx, light.colory, light.colorz);
    let scale = 0.25;
    let pos = in_pos * scale + Vec3::from(light_position);
    *out_pos = camera.view_proj * Vec4::from((pos, 1.0));
    *out_color = Vec3::from(light_position);
}

#[spirv(fragment)]
pub fn light_fs(in_color: Vec3, output: &mut Vec4) {
    *output = Vec4::from((in_color, 1.0));
}
