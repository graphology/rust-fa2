use crate::attraction::apply_attraction;
use crate::builder::FA2Data;
use crate::forces::apply_forces;
use crate::gravity::apply_gravity;
use crate::repulsion::apply_pairwise_repulsion;
use crate::settings::FA2Settings;
use crate::traits::Float;

pub struct FA2Layout<'d, F: Float> {
    settings: FA2Settings<F>,
    data: &'d mut FA2Data<F>,
}

impl<'d, F: Float> FA2Layout<'d, F> {
    pub(crate) fn new(settings: FA2Settings<F>, data: &'d mut FA2Data<F>) -> Self {
        Self { settings, data }
    }

    pub fn epoch(&mut self) -> F {
        self.data.reset();

        apply_pairwise_repulsion(&self.settings, &self.data.nodes, &mut self.data.deltas);
        apply_gravity(&self.settings, &self.data.nodes, &mut self.data.deltas);
        apply_attraction(
            &self.settings,
            &self.data.nodes,
            &self.data.edges,
            &mut self.data.deltas,
        );

        apply_forces(
            &self.settings,
            &mut self.data.nodes,
            &self.data.deltas,
            &self.data.last_deltas,
            &mut self.data.convergences,
        )
    }

    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.epoch();
        }
    }
}
