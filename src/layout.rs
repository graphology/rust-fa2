use crate::attraction::apply_attraction;
use crate::barnes_hut::BarnesHutTree;
use crate::data::FA2Data;
use crate::forces::apply_forces;
use crate::gravity::apply_gravity;
use crate::repulsion::apply_pairwise_repulsion;
use crate::settings::{FA2Settings, RepulsionMode};
use crate::traits::Float;

enum RepulsionIndex<F: Float> {
    None,
    BarnesHut(BarnesHutTree<F>),
}

impl<F: Float> RepulsionIndex<F> {
    fn new(mode: &RepulsionMode<F>, order: usize) -> Self {
        match mode {
            RepulsionMode::Pairwise => Self::None,
            RepulsionMode::BarnesHut { .. } => Self::BarnesHut(BarnesHutTree::with_capacity(order)),
        }
    }
}

pub struct FA2Layout<F: Float> {
    settings: FA2Settings<F>,
    data: FA2Data<F>,
    repulsion_index: RepulsionIndex<F>,
}

impl<F: Float> FA2Layout<F> {
    pub(crate) fn new(settings: FA2Settings<F>, data: FA2Data<F>) -> Self {
        let repulsion_index = RepulsionIndex::new(&settings.repulsion_mode, data.order());

        Self {
            settings,
            data,
            repulsion_index,
        }
    }

    pub fn epoch(&mut self) -> F {
        self.data.reset();

        match &mut self.repulsion_index {
            RepulsionIndex::None => {
                apply_pairwise_repulsion(
                    &self.settings,
                    &self.data.xs,
                    &self.data.ys,
                    &self.data.ms,
                    &mut self.data.delta_xs,
                    &mut self.data.delta_ys,
                );
            }
            RepulsionIndex::BarnesHut(tree) => {
                let extent = self.data.positions_extent().unwrap();
                tree.reset_with_extent(extent);
                tree.read(&self.data.xs, &self.data.ys, &self.data.ms);

                if self.settings.parallel {
                    tree.par_apply_repulsion(
                        &self.settings,
                        &self.data.xs,
                        &self.data.ys,
                        &self.data.ms,
                        &mut self.data.delta_xs,
                        &mut self.data.delta_ys,
                    );
                } else {
                    tree.apply_repulsion(
                        &self.settings,
                        &self.data.xs,
                        &self.data.ys,
                        &self.data.ms,
                        &mut self.data.delta_xs,
                        &mut self.data.delta_ys,
                    );
                }
            }
        };

        apply_gravity(
            &self.settings,
            &self.data.xs,
            &self.data.ys,
            &self.data.ms,
            &mut self.data.delta_xs,
            &mut self.data.delta_ys,
        );

        apply_attraction(
            &self.settings,
            &self.data.xs,
            &self.data.ys,
            &self.data.edges,
            &mut self.data.delta_xs,
            &mut self.data.delta_ys,
        );

        apply_forces(
            &self.settings,
            &mut self.data.xs,
            &mut self.data.ys,
            &self.data.ms,
            &self.data.delta_xs,
            &self.data.delta_ys,
            &self.data.old_delta_xs,
            &self.data.old_delta_ys,
            &mut self.data.convergences,
        )
    }

    pub fn data(&self) -> &FA2Data<F> {
        &self.data
    }

    pub fn into_data(self) -> FA2Data<F> {
        self.data
    }

    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.epoch();
        }
    }
}
