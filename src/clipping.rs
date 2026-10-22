use crate::{math::Vec4, shaders::VertexOutput};

type ClipPlane = fn(Vec4) -> f32;

const CLIP_PLANES: [ClipPlane; 6] = [
    |position| position.x + position.w,
    |position| position.w - position.x,
    |position| position.y + position.w,
    |position| position.w - position.y,
    |position| position.z + position.w,
    |position| position.w - position.z,
];

/// Sutherland-Hodgman clipping against the six homogeneous clip-space planes.
/// A triangle that intersects the frustum can become a polygon, which is then
/// triangulated again as a fan.
pub fn clip_triangle(triangle: [VertexOutput; 3]) -> Vec<[VertexOutput; 3]> {
    let mut polygon = triangle.to_vec();

    for plane in CLIP_PLANES {
        if polygon.is_empty() {
            break;
        }

        let input = std::mem::take(&mut polygon);
        let mut previous = *input.last().expect("the polygon is not empty");
        let mut previous_distance = plane(previous.clip_position);

        for current in input {
            let current_distance = plane(current.clip_position);
            let previous_inside = previous_distance >= 0.0;
            let current_inside = current_distance >= 0.0;

            if previous_inside != current_inside {
                let amount = previous_distance / (previous_distance - current_distance);
                polygon.push(previous.lerp(current, amount));
            }
            if current_inside {
                polygon.push(current);
            }

            previous = current;
            previous_distance = current_distance;
        }
    }

    if polygon.len() < 3 {
        return Vec::new();
    }

    (1..polygon.len() - 1)
        .map(|index| [polygon[0], polygon[index], polygon[index + 1]])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CLIP_PLANES, clip_triangle};
    use crate::{
        math::Vec4,
        shaders::{Varyings, VertexOutput},
    };

    fn vertex(x: f32, y: f32, z: f32) -> VertexOutput {
        VertexOutput {
            clip_position: Vec4::new(x, y, z, 1.0),
            varyings: Varyings::default(),
        }
    }

    #[test]
    fn clips_triangle_crossing_near_plane() {
        let triangles = clip_triangle([
            vertex(-0.5, -0.5, 0.0),
            vertex(0.5, -0.5, 0.0),
            vertex(0.0, 0.5, -2.0),
        ]);

        assert_eq!(triangles.len(), 2);
        assert!(triangles.iter().flatten().all(|vertex| {
            CLIP_PLANES
                .iter()
                .all(|plane| plane(vertex.clip_position) >= -f32::EPSILON)
        }));
    }

    #[test]
    fn discards_triangles_outside_every_frustum_plane() {
        let outside_triangles = [
            [
                vertex(-2.0, 0.0, 0.0),
                vertex(-2.0, 0.3, 0.0),
                vertex(-2.0, -0.3, 0.0),
            ],
            [
                vertex(2.0, 0.0, 0.0),
                vertex(2.0, 0.3, 0.0),
                vertex(2.0, -0.3, 0.0),
            ],
            [
                vertex(0.0, -2.0, 0.0),
                vertex(0.3, -2.0, 0.0),
                vertex(-0.3, -2.0, 0.0),
            ],
            [
                vertex(0.0, 2.0, 0.0),
                vertex(0.3, 2.0, 0.0),
                vertex(-0.3, 2.0, 0.0),
            ],
            [
                vertex(0.0, 0.0, -2.0),
                vertex(0.3, 0.0, -2.0),
                vertex(-0.3, 0.0, -2.0),
            ],
            [
                vertex(0.0, 0.0, 2.0),
                vertex(0.3, 0.0, 2.0),
                vertex(-0.3, 0.0, 2.0),
            ],
        ];

        assert!(
            outside_triangles
                .into_iter()
                .all(|triangle| clip_triangle(triangle).is_empty())
        );
    }
}
