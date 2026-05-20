// use rayon::prelude::*;

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

pub struct FA2Layout<'d, F: Float> {
    settings: FA2Settings<F>,
    data: &'d mut FA2Data<F>,
    repulsion_index: RepulsionIndex<F>,
}

impl<'d, F: Float> FA2Layout<'d, F> {
    pub(crate) fn new(settings: FA2Settings<F>, data: &'d mut FA2Data<F>) -> Self {
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
                apply_pairwise_repulsion(&self.settings, &self.data.nodes, &mut self.data.deltas);
            }
            RepulsionIndex::BarnesHut(tree) => {
                let extent = self.data.positions_extent().unwrap();
                tree.reset_with_extent(extent);
                tree.read(&self.data.nodes);
                tree.apply_repulsion(&self.settings, &self.data.nodes, &mut self.data.deltas);
            }
        };

        apply_gravity(&self.settings, &self.data.nodes, &mut self.data.deltas);

        if self.settings.parallel {
            let _chunk_size =
                (self.data.size() / rayon::current_num_threads()).min(self.data.size());

            todo!()
        } else {
            apply_attraction(
                &self.settings,
                &self.data.nodes,
                &self.data.edges,
                &mut self.data.deltas,
            );
        }

        apply_forces(
            &self.settings,
            &mut self.data.nodes,
            &self.data.deltas,
            &self.data.last_deltas,
            &mut self.data.convergences,
        )
    }

    pub fn data(&self) -> &FA2Data<F> {
        self.data
    }

    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.epoch();
        }
    }
}
