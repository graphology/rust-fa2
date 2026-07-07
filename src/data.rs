// Ref: https://github.com/graphology/graphology/blob/master/src/layout-forceatlas2/helpers.js
use crate::traits::Float;

/// A struct holding the data necessary to represent a graph that will be
/// spatialized by the FA2 algorithm.
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
    /// Create an empty [`FA2Data`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty [`FA2Data`] instance fitting a given number of nodes
    /// before resizing inner allocations.
    pub fn with_node_capacity(nodes: usize) -> Self {
        Self {
            xs: Vec::with_capacity(nodes),
            ys: Vec::with_capacity(nodes),
            ms: Vec::with_capacity(nodes),
            delta_xs: Vec::with_capacity(nodes),
            delta_ys: Vec::with_capacity(nodes),
            old_delta_xs: Vec::with_capacity(nodes),
            old_delta_ys: Vec::with_capacity(nodes),
            convergences: Vec::with_capacity(nodes),
            edges: Vec::new(),
        }
    }

    /// Create an empty [`FA2Data`] instance fitting a given number of nodes
    /// and edges before resizing inner allocations.
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

    /// Return the order of represented graph, i.e. its number of nodes.
    pub fn order(&self) -> usize {
        self.xs.len()
    }

    /// Return the size of represented graph, i.e. its number of edges.
    pub fn size(&self) -> usize {
        self.edges.len()
    }

    /// Add a node with given position.
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

    /// Set target node's position.
    #[inline]
    pub fn set_node_position(&mut self, n: usize, x: F, y: F) {
        self.xs[n] = x;
        self.ys[n] = y;
    }

    /// Add an edge with given weight.
    #[inline]
    pub fn add_edge_with_weight(&mut self, i: usize, j: usize, weight: F) {
        self.ms[i] += weight;
        self.ms[j] += weight;

        self.edges.push((i, j, weight));
    }

    /// Add an edge with a default weight of `1`.
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

/// A romantic name for a variant of CSR matrix.
#[derive(Debug)]
pub(crate) struct NeighborhoodIndex<F: Float> {
    node_offsets: Vec<usize>,
    edge_stubs: Vec<(usize, F)>,
}

impl<'a, F: Float> From<&'a FA2Data<F>> for NeighborhoodIndex<F> {
    fn from(data: &'a FA2Data<F>) -> Self {
        // NOTE: we reserve an additional slot for convenience later when
        // we need to lookup node adjacency lists.
        let mut node_offsets = vec![0; data.order() + 1];

        // First we aggregate degrees
        for (source, target, _) in data.edges.iter() {
            node_offsets[*source] += 1;
            node_offsets[*target] += 1;
        }

        // We compute the cumulative sum
        let mut cumsum: usize = 0;

        for degree in node_offsets.iter_mut() {
            cumsum += *degree;
            *degree = cumsum;
        }

        // Then we populate stubs
        let mut edge_stubs = vec![(0, F::zero()); data.size() * 2];

        for (source, target, weight) in data.edges.iter() {
            node_offsets[*source] -= 1;
            node_offsets[*target] -= 1;

            let source_index = node_offsets[*source];
            let target_index = node_offsets[*target];

            edge_stubs[source_index] = (*target, *weight);
            edge_stubs[target_index] = (*source, *weight);
        }

        debug_assert!(matches!(node_offsets.last(), Some(d) if *d == data.size() * 2));

        Self {
            node_offsets,
            edge_stubs,
        }
    }
}

impl<F: Float> NeighborhoodIndex<F> {
    #[inline]
    pub(crate) fn stubs(&self, node: usize) -> &[(usize, F)] {
        debug_assert!(node < self.node_offsets.len().saturating_sub(1));

        &self.edge_stubs[self.node_offsets[node]..self.node_offsets[node + 1]]
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

    #[test]
    fn test_neighborhood_index() {
        let mut data = FA2Data::<f32>::new();

        for _ in 0..5 {
            data.add_node(0.0, 0.0);
        }

        data.add_edge(0, 1);
        data.add_edge(0, 2);
        data.add_edge_with_weight(0, 3, 4.0);
        data.add_edge(2, 0);
        data.add_edge(4, 1);
        data.add_edge(2, 3);

        let index = NeighborhoodIndex::from(&data);

        assert_eq!(index.stubs(0), [(2, 1.0), (3, 4.0), (2, 1.0), (1, 1.0)]);
        assert_eq!(index.stubs(1), [(4, 1.0), (0, 1.0)]);
        assert_eq!(index.stubs(2), [(3, 1.0), (0, 1.0), (0, 1.0)]);
        assert_eq!(index.stubs(3), [(2, 1.0), (0, 4.0)]);
        assert_eq!(index.stubs(4), [(1, 1.0)]);
    }
}
