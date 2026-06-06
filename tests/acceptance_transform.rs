use wgpu_vulkan_spike::draw_command::{DrawCommand, parse_command};
use wgpu_vulkan_spike::transform::Transform;

#[test]
fn parse_set_transform_command() {
    let input = "(set-transform 1 0.25 0.0 1.0 1.0)";
    let cmd = parse_command(input);
    assert_eq!(
        cmd,
        Some(DrawCommand::SetTransform {
            id: 1,
            tx: 0.25,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0
        })
    );
}

#[test]
fn parse_set_transform_with_negative_values() {
    let input = "(set-transform 2 -0.5 0.5 2.0 2.0)";
    let cmd = parse_command(input);
    assert_eq!(
        cmd,
        Some(DrawCommand::SetTransform {
            id: 2,
            tx: -0.5,
            ty: 0.5,
            sx: 2.0,
            sy: 2.0
        })
    );
}

#[test]
fn parse_set_transform_with_extra_params_fails() {
    let input = "(set-transform 2 -0.5 0.5 2.0 2.0 0.785)";
    let cmd = parse_command(input);
    assert!(cmd.is_none());
}

#[test]
fn parse_clear_still_works() {
    let input = "(clear 1.0 0.0 0.0 1.0)";
    let cmd = parse_command(input);
    assert_eq!(
        cmd,
        Some(DrawCommand::Clear {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0
        })
    );
}

#[test]
fn transform_identity() {
    let t = Transform::identity();
    let matrix = t.to_matrix();
    assert_eq!(t.tx, 0.0);
    assert_eq!(t.ty, 0.0);
    assert_eq!(t.sx, 1.0);
    assert_eq!(t.sy, 1.0);
    assert_eq!(matrix[0][0], 1.0);
    assert_eq!(matrix[1][1], 1.0);
    assert_eq!(matrix[3][3], 1.0);
}

#[test]
fn transform_translation() {
    let t = Transform::new(0.5, -0.25, 1.0, 1.0, 0.0);
    let matrix = t.to_matrix();
    assert_eq!(matrix[0][3], 0.5);
    assert_eq!(matrix[1][3], -0.25);
}

#[test]
fn transform_scale() {
    let t = Transform::new(0.0, 0.0, 2.0, 3.0, 0.0);
    let matrix = t.to_matrix();
    assert_eq!(matrix[0][0], 2.0);
    assert_eq!(matrix[1][1], 3.0);
}
