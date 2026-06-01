use gpui::{Background, Bounds, PathBuilder, Pixels, Point, Window, fill, point, px, rgb, size};

use super::style::{DagNodeStyle, NodeFill, NodeShape};

const LINE_WIDTH: f32 = 1.5;

pub(super) fn stroke_line(
    window: &mut Window,
    x0: Pixels,
    y0: Pixels,
    x1: Pixels,
    y1: Pixels,
    color: u32,
) {
    let mut pb = PathBuilder::stroke(px(LINE_WIDTH));
    pb.move_to(point(x0, y0));
    pb.line_to(point(x1, y1));
    if let Ok(path) = pb.build() {
        window.paint_path(path, rgb(color));
    }
}

pub(super) fn stroke_curve(
    window: &mut Window,
    sx: Pixels,
    sy: Pixels,
    ex: Pixels,
    ey: Pixels,
    color: u32,
) {
    // Vertical drop to mid-y, then quadratic curve out to the target lane.
    let mid_y = sy + (ey - sy) * 0.4;
    let mut pb = PathBuilder::stroke(px(LINE_WIDTH));
    pb.move_to(point(sx, sy));
    pb.line_to(point(sx, mid_y));
    pb.curve_to(point(ex, ey), point(ex, mid_y));
    if let Ok(path) = pb.build() {
        window.paint_path(path, rgb(color));
    }
}

pub(super) fn paint_node(window: &mut Window, cx_x: Pixels, cy_y: Pixels, style: DagNodeStyle) {
    let r = px(style.radius);
    match style.shape {
        NodeShape::Circle => match style.fill {
            NodeFill::Filled(c) => {
                let b = Bounds::new(point(cx_x - r, cy_y - r), size(r * 2., r * 2.));
                let bg: Background = rgb(c).into();
                window.paint_quad(fill(b, bg).corner_radii(r));
            }
            NodeFill::Outlined(c, lw) => {
                let b = Bounds::new(point(cx_x - r, cy_y - r), size(r * 2., r * 2.));
                let q = gpui::quad(
                    b,
                    r,
                    gpui::transparent_black(),
                    gpui::Edges::all(px(lw)),
                    rgb(c),
                    gpui::BorderStyle::Solid,
                );
                window.paint_quad(q);
            }
        },
        NodeShape::Diamond => {
            let pts = [
                Point::new(cx_x, cy_y - r),
                Point::new(cx_x + r, cy_y),
                Point::new(cx_x, cy_y + r),
                Point::new(cx_x - r, cy_y),
            ];
            match style.fill {
                NodeFill::Filled(c) => {
                    if let Some(path) = diamond_path(pts) {
                        window.paint_path(path, rgb(c));
                    }
                }
                NodeFill::Outlined(c, lw) => {
                    if let Some(path) = diamond_outline_path(pts, lw) {
                        window.paint_path(path, rgb(c));
                    }
                }
            }
        }
    }
}

fn diamond_path(pts: [Point<Pixels>; 4]) -> Option<gpui::Path<Pixels>> {
    let mut pb = PathBuilder::fill();
    pb.move_to(pts[0]);
    pb.line_to(pts[1]);
    pb.line_to(pts[2]);
    pb.line_to(pts[3]);
    pb.line_to(pts[0]);
    pb.build().ok()
}

fn diamond_outline_path(pts: [Point<Pixels>; 4], width: f32) -> Option<gpui::Path<Pixels>> {
    let mut pb = PathBuilder::stroke(px(width));
    pb.move_to(pts[0]);
    pb.line_to(pts[1]);
    pb.line_to(pts[2]);
    pb.line_to(pts[3]);
    pb.line_to(pts[0]);
    pb.build().ok()
}
