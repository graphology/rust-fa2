use crate::traits::Float;

#[derive(Debug)]
pub struct FA2Data<F: Float> {
    pub(crate) nodes: Vec<F>,       // Layout is: (x, y, mass)
    pub(crate) deltas: Vec<F>,      // Layout is: (dx, dy)
    pub(crate) last_deltas: Vec<F>, // Layout is: (old_dx, old_dy)
    pub(crate) edges: Vec<(usize, usize, F)>,
}

impl<F: Float> Default for FA2Data<F> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            deltas: Vec::new(),
            last_deltas: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl<F: Float> FA2Data<F> {
    pub fn new() -> Self {
        Self::default()
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

        index
    }

    pub fn add_edge_with_weight(&mut self, i: usize, j: usize, weight: F) {
        self.nodes[i * 3 + 2] += weight;
        self.nodes[j * 3 + 2] += weight;

        self.edges.push((i, j, weight));
    }

    pub fn add_edge(&mut self, i: usize, j: usize) {
        self.add_edge_with_weight(i, j, F::one());
    }
}
