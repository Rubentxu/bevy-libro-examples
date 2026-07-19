// Capítulo 23. UI — Layout calculations, flex math
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width
            && py >= self.y && py <= self.y + self.height
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

/// Simple flex-like layout calculator
pub struct LayoutNode {
    pub direction: Direction,
    pub align: Align,
    pub padding: f32,
    pub gap: f32,
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            align: Align::Start,
            padding: 8.0,
            gap: 4.0,
        }
    }
}

impl LayoutNode {
    /// Lay out children within a container rect
    pub fn layout_children(&self, container: &Rect, child_count: usize, child_size: (f32, f32)) -> Vec<Rect> {
        if child_count == 0 {
            return vec![];
        }

        let inner_x = container.x + self.padding;
        let inner_y = container.y + self.padding;
        let mut results = Vec::with_capacity(child_count);

        match self.direction {
            Direction::Row => {
                let total_gap = self.gap * (child_count - 1).max(0) as f32;
                let total_width = child_count as f32 * child_size.0 + total_gap;

                let start_x = match self.align {
                    Align::Center => inner_x + (container.width - 2.0 * self.padding - total_width) * 0.5,
                    Align::End => container.x + container.width - self.padding - total_width,
                    _ => inner_x,
                };

                for i in 0..child_count {
                    let x = start_x + i as f32 * (child_size.0 + self.gap);
                    let y = match self.align {
                        Align::Stretch => inner_y,
                        Align::Center => inner_y + (container.height - 2.0 * self.padding - child_size.1) * 0.5,
                        _ => inner_y,
                    };
                    let h = if self.align == Align::Stretch {
                        container.height - 2.0 * self.padding
                    } else {
                        child_size.1
                    };
                    results.push(Rect::new(x, y, child_size.0, h));
                }
            }
            Direction::Column => {
                let total_gap = self.gap * (child_count - 1).max(0) as f32;
                let total_height = child_count as f32 * child_size.1 + total_gap;

                let start_y = match self.align {
                    Align::Center => inner_y + (container.height - 2.0 * self.padding - total_height) * 0.5,
                    Align::End => container.y + container.height - self.padding - total_height,
                    _ => inner_y,
                };

                for i in 0..child_count {
                    let y = start_y + i as f32 * (child_size.1 + self.gap);
                    let x = match self.align {
                        Align::Stretch => inner_x,
                        Align::Center => inner_x + (container.width - 2.0 * self.padding - child_size.0) * 0.5,
                        _ => inner_x,
                    };
                    let w = if self.align == Align::Stretch {
                        container.width - 2.0 * self.padding
                    } else {
                        child_size.0
                    };
                    results.push(Rect::new(x, y, w, child_size.1));
                }
            }
        }

        results
    }
}

/// Anchor: where to position a UI element relative to the screen
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub fn anchor_rect(anchor: Anchor, element_size: (f32, f32), screen_size: (f32, f32), margin: f32) -> Rect {
    let (sw, sh) = screen_size;
    let (ew, eh) = element_size;

    let (x, y) = match anchor {
        Anchor::TopLeft => (margin, margin),
        Anchor::TopCenter => ((sw - ew) * 0.5, margin),
        Anchor::TopRight => (sw - ew - margin, margin),
        Anchor::CenterLeft => (margin, (sh - eh) * 0.5),
        Anchor::Center => ((sw - ew) * 0.5, (sh - eh) * 0.5),
        Anchor::CenterRight => (sw - ew - margin, (sh - eh) * 0.5),
        Anchor::BottomLeft => (margin, sh - eh - margin),
        Anchor::BottomCenter => ((sw - ew) * 0.5, sh - eh - margin),
        Anchor::BottomRight => (sw - ew - margin, sh - eh - margin),
    };

    Rect::new(x, y, ew, eh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_point() {
        let r = Rect::new(10.0, 10.0, 50.0, 50.0);
        assert!(r.contains(20.0, 30.0));
        assert!(!r.contains(5.0, 30.0));
        assert!(!r.contains(70.0, 30.0));
    }

    #[test]
    fn rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 50.0);
        let (cx, cy) = r.center();
        assert!((cx - 50.0).abs() < 0.001 && (cy - 25.0).abs() < 0.001);
    }

    #[test]
    fn rect_intersects() {
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(40.0, 40.0, 50.0, 50.0);
        let c = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn layout_column_start() {
        let node = LayoutNode::default();
        let container = Rect::new(0.0, 0.0, 200.0, 400.0);
        let rects = node.layout_children(&container, 3, (100.0, 30.0));

        assert_eq!(rects.len(), 3);
        assert!((rects[0].y - 8.0).abs() < 0.001, "First child at top padding");
        assert!(rects[1].y > rects[0].y, "Second child below first");
    }

    #[test]
    fn layout_row_start() {
        let node = LayoutNode { direction: Direction::Row, ..Default::default() };
        let container = Rect::new(0.0, 0.0, 400.0, 200.0);
        let rects = node.layout_children(&container, 3, (50.0, 30.0));

        assert_eq!(rects.len(), 3);
        assert!(rects[0].x < rects[1].x);
        assert!(rects[1].x < rects[2].x);
    }

    #[test]
    fn layout_center_alignment() {
        let node = LayoutNode {
            direction: Direction::Row,
            align: Align::Center,
            ..Default::default()
        };
        let container = Rect::new(0.0, 0.0, 400.0, 200.0);
        let rects = node.layout_children(&container, 1, (100.0, 30.0));

        let (cx, _) = rects[0].center();
        assert!((cx - 200.0).abs() < 5.0, "Center-aligned should be centered");
    }

    #[test]
    fn anchor_top_left() {
        let r = anchor_rect(Anchor::TopLeft, (50.0, 30.0), (800.0, 600.0), 10.0);
        assert!((r.x - 10.0).abs() < 0.001);
        assert!((r.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn anchor_center() {
        let r = anchor_rect(Anchor::Center, (100.0, 50.0), (800.0, 600.0), 0.0);
        assert!((r.x - 350.0).abs() < 0.001);
        assert!((r.y - 275.0).abs() < 0.001);
    }

    #[test]
    fn anchor_bottom_right() {
        let r = anchor_rect(Anchor::BottomRight, (100.0, 50.0), (800.0, 600.0), 10.0);
        assert!((r.x - 690.0).abs() < 0.001);
        assert!((r.y - 540.0).abs() < 0.001);
    }

    #[test]
    fn layout_respects_gap() {
        let node = LayoutNode { gap: 20.0, ..Default::default() };
        let container = Rect::new(0.0, 0.0, 200.0, 600.0);
        let rects = node.layout_children(&container, 3, (100.0, 30.0));

        let gap = rects[1].y - rects[0].y - rects[0].height;
        assert!((gap - 20.0).abs() < 0.001, "Gap should be 20.0, got {}", gap);
    }

    #[test]
    fn layout_empty_children() {
        let node = LayoutNode::default();
        let container = Rect::new(0.0, 0.0, 200.0, 400.0);
        let rects = node.layout_children(&container, 0, (100.0, 30.0));
        assert!(rects.is_empty());
    }
}
