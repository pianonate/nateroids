use crate::playfield::{
    boundary::BoundaryFace,
    portals::Portal,
    Boundary,
};
use bevy::prelude::*;

pub fn calculate_intersection_points(
    portal: &Portal,
    boundary: &Boundary,
    overextended_faces: Vec<BoundaryFace>,
) -> Vec<(BoundaryFace, Vec<Vec3>)> {
    let mut intersections = Vec::new();
    let half_size = boundary.transform.scale / 2.0;
    let min = boundary.transform.translation - half_size;
    let max = boundary.transform.translation + half_size;

    for face in overextended_faces {
        let face_points = get_face_points(face, &min, &max);
        let face_intersections = intersect_circle_with_rectangle(portal, &face_points);

        if !face_intersections.is_empty() {
            intersections.push((face, face_intersections));
        }
    }

    intersections
}

fn get_face_points(face: BoundaryFace, min: &Vec3, max: &Vec3) -> [Vec3; 4] {
    match face {
        BoundaryFace::Left => [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(min.x, min.y, max.z),
        ],
        BoundaryFace::Right => [
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(max.x, min.y, max.z),
        ],
        BoundaryFace::Bottom => [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, min.y, max.z),
        ],
        BoundaryFace::Top => [
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ],
        BoundaryFace::Back => [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
        ],
        BoundaryFace::Front => [
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ],
    }
}

fn intersect_circle_with_rectangle(portal: &Portal, rectangle_points: &[Vec3; 4]) -> Vec<Vec3> {
    let mut intersections = Vec::new();

    for i in 0..4 {
        let start = rectangle_points[i];
        let end = rectangle_points[(i + 1) % 4];

        let edge_intersections = intersect_circle_with_line_segment(portal, start, end);
        intersections.extend(edge_intersections);
    }

    intersections
}

fn intersect_circle_with_line_segment(portal: &Portal, start: Vec3, end: Vec3) -> Vec<Vec3> {
    let edge = end - start;
    let center_to_start = start - portal.position;

    let a = edge.dot(edge);
    let b = 2.0 * center_to_start.dot(edge);
    let c = center_to_start.dot(center_to_start) - portal.radius * portal.radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return vec![];
    }

    let mut intersections = Vec::new();
    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

    if (0.0..=1.0).contains(&t1) {
        intersections.push(start + t1 * edge);
    }
    if (0.0..=1.0).contains(&t2) && (t1 - t2).abs() > 1e-6 {
        intersections.push(start + t2 * edge);
    }

    intersections
}
