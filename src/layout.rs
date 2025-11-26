use crate::builder::FA2Data;
use crate::gravity::apply_gravity;
use crate::repulsion::apply_pairwise_repulsion;
use crate::settings::FA2Settings;
use crate::traits::Float;

pub struct FA2Layout<F: Float> {
    settings: FA2Settings<F>,
    data: FA2Data<F>,
}

impl<F: Float> FA2Layout<F> {
    pub fn with_settings(settings: FA2Settings<F>, data: FA2Data<F>) -> Self {
        Self { settings, data }
    }

    fn epoch(&mut self) {
        apply_pairwise_repulsion(&self.settings, &self.data.nodes, &mut self.data.deltas);
        apply_gravity(&self.settings, &self.data.nodes, &mut self.data.deltas);
    }

    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.epoch();
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = (F, F)> + '_ {
        self.data.nodes.chunks(3).map(|w| (w[0], w[1]))
    }
}
