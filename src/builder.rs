// Ref: https://github.com/graphology/graphology/blob/master/src/layout-forceatlas2/helpers.js
use crate::traits::Float;

#[derive(Debug)]
pub struct FA2Data<F: Float> {
    pub(crate) nodes: Vec<F>,       // Layout is: (x, y, mass)
    pub(crate) deltas: Vec<F>,      // Layout is: (dx, dy)
    pub(crate) last_deltas: Vec<F>, // Layout is: (old_dx, old_dy)
    pub(crate) convergences: Vec<F>,
    pub(crate) edges: Vec<(usize, usize, F)>,
}

impl<F: Float> Default for FA2Data<F> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            deltas: Vec::new(),
            last_deltas: Vec::new(),
            convergences: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl<F: Float> FA2Data<F> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(nodes * 3),
            deltas: Vec::with_capacity(nodes * 2),
            last_deltas: Vec::with_capacity(nodes * 2),
            convergences: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    pub fn order(&self) -> usize {
        self.nodes.len() / 3
    }

    pub fn add_node(&mut self, x: F, y: F) -> usize {
        let index = self.nodes.len() / 3;

        self.nodes.push(x);
        self.nodes.push(y);
        self.nodes.push(F::one());

        self.deltas.push(F::zero());
        self.deltas.push(F::zero());

        self.last_deltas.push(F::zero());
        self.last_deltas.push(F::zero());

        self.convergences.push(F::one());

        index
    }

    #[inline]
    pub fn add_edge_with_weight(&mut self, i: usize, j: usize, weight: F) {
        self.nodes[i * 3 + 2] += weight;
        self.nodes[j * 3 + 2] += weight;

        self.edges.push((i, j, weight));
    }

    #[inline]
    pub fn add_edge(&mut self, i: usize, j: usize) {
        self.add_edge_with_weight(i, j, F::one());
    }

    pub(crate) fn positions_extent(&self) -> Option<(F, F, F, F)> {
        let mut extent = None;

        for node in self.nodes.chunks(3) {
            let x = node[0];
            let y = node[1];

            match extent.as_mut() {
                None => {
                    extent = Some((x, x, y, y));
                }
                Some((min_x, max_x, min_y, max_y)) => {
                    if x < *min_x {
                        *min_x = x;
                    }

                    if x > *max_x {
                        *max_x = x;
                    }

                    if y < *min_y {
                        *min_y = y;
                    }

                    if y > *max_y {
                        *max_y = y;
                    }
                }
            }
        }

        extent
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        std::mem::swap(&mut self.deltas, &mut self.last_deltas);

        for x in self.deltas.iter_mut() {
            *x = F::zero();
        }
    }

    pub fn apply_circular_layout(&mut self) {
        let tau = F::TAU();
        let order = F::from(self.order()).unwrap();

        let mut i = F::zero();

        for node in self.nodes.chunks_mut(3) {
            let p = (i * tau) / order;

            node[0] = p.cos();
            node[1] = p.sin();

            i += F::one();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positions_extent() {
        let mut data = FA2Data::<f32>::new();
        data.add_node(1.0, -3.0);
        data.add_node(-1.0, 4.0);
        data.add_node(6.0, 1.0);
        data.add_node(9.0, 31.0);
        data.add_node(1.0, 3.0);

        let extent = data.positions_extent();

        assert_eq!(extent, Some((-1.0, 9.0, -3.0, 31.0)));
    }
}
