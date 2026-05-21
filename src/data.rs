// Ref: https://github.com/graphology/graphology/blob/master/src/layout-forceatlas2/helpers.js
use crate::traits::Float;

#[derive(Debug)]
pub struct FA2Data<F: Float> {
    pub(crate) xs: Vec<F>,
    pub(crate) ys: Vec<F>,
    pub(crate) ms: Vec<F>,
    pub(crate) delta_xs: Vec<F>,
    pub(crate) delta_ys: Vec<F>,
    pub(crate) old_delta_xs: Vec<F>,
    pub(crate) old_delta_ys: Vec<F>,
    pub(crate) convergences: Vec<F>,
    pub(crate) edges: Vec<(usize, usize, F)>,
}

impl<F: Float> Default for FA2Data<F> {
    fn default() -> Self {
        Self {
            xs: Vec::new(),
            ys: Vec::new(),
            ms: Vec::new(),
            delta_xs: Vec::new(),
            delta_ys: Vec::new(),
            old_delta_xs: Vec::new(),
            old_delta_ys: Vec::new(),
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
            xs: Vec::with_capacity(nodes),
            ys: Vec::with_capacity(nodes),
            ms: Vec::with_capacity(nodes),
            delta_xs: Vec::with_capacity(nodes),
            delta_ys: Vec::with_capacity(nodes),
            old_delta_xs: Vec::with_capacity(nodes),
            old_delta_ys: Vec::with_capacity(nodes),
            convergences: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    pub fn order(&self) -> usize {
        self.xs.len()
    }

    pub fn size(&self) -> usize {
        self.edges.len()
    }

    pub fn add_node(&mut self, x: F, y: F) -> usize {
        let index = self.order();

        self.xs.push(x);
        self.ys.push(y);
        self.ms.push(F::one());

        self.delta_xs.push(F::zero());
        self.delta_ys.push(F::zero());

        self.old_delta_xs.push(F::zero());
        self.old_delta_ys.push(F::zero());

        self.convergences.push(F::one());

        index
    }

    #[inline]
    pub fn add_edge_with_weight(&mut self, i: usize, j: usize, weight: F) {
        self.ms[i] += weight;
        self.ms[j] += weight;

        self.edges.push((i, j, weight));
    }

    #[inline]
    pub fn add_edge(&mut self, i: usize, j: usize) {
        self.add_edge_with_weight(i, j, F::one());
    }

    pub fn positions(&self) -> impl Iterator<Item = (F, F)> + '_ {
        self.xs.iter().zip(self.ys.iter()).map(|(x, y)| (*x, *y))
    }

    pub(crate) fn positions_extent(&self) -> Option<(F, F, F, F)> {
        let mut extent = None;

        for (x, y) in self.xs.iter().copied().zip(self.ys.iter().copied()) {
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
        std::mem::swap(&mut self.delta_xs, &mut self.old_delta_xs);
        std::mem::swap(&mut self.delta_ys, &mut self.old_delta_ys);

        for i in 0..self.delta_xs.len() {
            self.delta_xs[i] = F::zero();
            self.delta_ys[i] = F::zero();
        }
    }

    pub fn apply_circular_layout(&mut self) {
        let tau = F::TAU();
        let order = F::from(self.order()).unwrap();

        let mut i = F::zero();

        for (x, y) in self.xs.iter_mut().zip(self.ys.iter_mut()) {
            let p = (i * tau) / order;

            *x = p.cos();
            *y = p.sin();

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
