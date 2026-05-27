use rayon::prelude::*;

use crate::attraction::{apply_attraction, apply_nodewise_attraction};
use crate::barnes_hut::BarnesHutTree;
use crate::data::{FA2Data, NeighborhoodIndex};
use crate::forces::apply_forces;
use crate::gravity::{apply_gravity, apply_nodewise_gravity};
use crate::repulsion::{apply_nodewise_repulsion, apply_pairwise_repulsion};
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
    neighborhood_index: Option<NeighborhoodIndex<F>>,
    repulsion_index: RepulsionIndex<F>,
}

impl<F: Float> FA2Layout<F> {
    pub(crate) fn new(settings: FA2Settings<F>, data: FA2Data<F>) -> Self {
        let repulsion_index = RepulsionIndex::new(&settings.repulsion_mode, data.order());

        let neighborhood_index = settings.parallel.then(|| NeighborhoodIndex::from(&data));

        Self {
            settings,
            data,
            neighborhood_index,
            repulsion_index,
        }
    }

    pub fn epoch(&mut self) -> F {
        self.data.reset();

        if let RepulsionIndex::BarnesHut(tree) = &mut self.repulsion_index {
            let extent = self.data.positions_extent().unwrap();
            tree.reset_with_extent(extent);
            tree.rebuild(&self.data.xs, &self.data.ys, &self.data.ms);
        }

        // Parallel path
        if self.settings.parallel {
            let neighborhood_index = self.neighborhood_index.as_ref().unwrap();

            self.data
                .delta_xs
                .par_iter_mut()
                .zip(self.data.delta_ys.par_iter_mut())
                .enumerate()
                .for_each(|(n, (out_x, out_y))| {
                    match &self.repulsion_index {
                        RepulsionIndex::None => {
                            apply_nodewise_repulsion(
                                &self.settings,
                                self.data.xs[n],
                                self.data.ys[n],
                                self.data.ms[n],
                                &self.data.xs,
                                &self.data.ys,
                                &self.data.ms,
                                out_x,
                                out_y,
                            );
                        }
                        RepulsionIndex::BarnesHut(tree) => {
                            tree.apply_nodewise_repulsion(
                                &self.settings,
                                n,
                                &self.data.xs,
                                &self.data.ys,
                                &self.data.ms,
                                out_x,
                                out_y,
                            );
                        }
                    };

                    apply_nodewise_gravity(
                        &self.settings,
                        self.data.xs[n],
                        self.data.ys[n],
                        self.data.ms[n],
                        out_x,
                        out_y,
                    );

                    apply_nodewise_attraction(
                        &self.settings,
                        neighborhood_index,
                        n,
                        &self.data.xs,
                        &self.data.ys,
                        out_x,
                        out_y,
                    );
                });

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
        // Sequential path
        else {
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
                    for n in 0..self.data.xs.len() {
                        tree.apply_nodewise_repulsion(
                            &self.settings,
                            n,
                            &self.data.xs,
                            &self.data.ys,
                            &self.data.ms,
                            &mut self.data.delta_xs[n],
                            &mut self.data.delta_ys[n],
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
