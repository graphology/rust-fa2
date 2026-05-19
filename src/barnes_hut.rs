use crate::traits::Float;

const SUBDIVISION_ATTEMPTS: usize = 3;

struct BarnesHutNode<F: Float> {
    node: Option<usize>,
    center_x: F,
    center_y: F,
    size: F,
    next_sibling: Option<usize>,
    first_child: Option<usize>,
    mass: F,
    mass_center_x: F,
    mass_center_y: F,
}

impl<F: Float> BarnesHutNode<F> {
    fn new_root(min_x: F, max_x: F, min_y: F, max_y: F) -> Self {
        let two = F::from(2.0).unwrap();

        Self {
            node: None,
            center_x: (min_x + max_x) / two,
            center_y: (min_y + max_y) / two,
            size: (max_x - min_x).max(max_y - min_y),
            next_sibling: None,
            first_child: None,
            mass: F::zero(),
            mass_center_x: F::zero(),
            mass_center_y: F::zero(),
        }
    }
}

pub struct BarnesHutTree<F: Float> {
    nodes: Vec<BarnesHutNode<F>>,
}

impl<F: Float> BarnesHutTree<F> {
    pub fn new(mut min_x: F, mut max_x: F, mut min_y: F, mut max_y: F) -> Self {
        // Squarifying bounds
        let dx = max_x - min_x;
        let dy = max_y - min_y;

        let two = F::from(2.0).unwrap();

        if dx > dy {
            min_y -= (dx - dy) / two;
            max_y = min_y + dx;
        } else {
            min_x -= (dy - dx) / two;
            max_x = min_x + dy;
        }

        Self {
            nodes: vec![BarnesHutNode::new_root(min_x, max_x, min_y, max_y)],
        }
    }
}
