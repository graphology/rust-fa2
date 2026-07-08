use crate::data::FA2Data;
use crate::layout::FA2Layout;
use crate::traits::Float;

#[derive(Debug, Clone)]
pub(crate) enum RepulsionMode<F: Float> {
    Pairwise,
    BarnesHut { theta: F },
}

/// Struct representing a settings of the FA2 layout algorithm.
#[derive(Debug, Clone)]
pub struct FA2Settings<F: Float> {
    pub(crate) repulsion_mode: RepulsionMode<F>,
    // pub(crate) lin_log_mode: bool,
    pub(crate) edge_weight_influence: F,
    pub(crate) gravity: F,
    pub(crate) strong_gravity_mode: bool,
    // pub(crate) outbound_attraction_distribution: bool,
    pub(crate) scaling_ratio: F,
    pub(crate) slow_down: F,
    pub(crate) parallel: bool,
}

impl<F: Float> Default for FA2Settings<F> {
    /// Instantiate default settings for the FA2 algorithm. That is to say
    /// paiwise repulsion, an `edge_weight_influence` of 1, a `gravity` of 1
    /// no `strong_gravity_mode`, a `scaling_ratio` of 1, a `slow_down` of 1
    /// and single-threaded.
    ///
    // Ref: https://github.com/graphology/graphology/blob/master/src/layout-forceatlas2/defaults.js
    fn default() -> Self {
        Self {
            repulsion_mode: RepulsionMode::Pairwise,
            // lin_log_mode: false,
            edge_weight_influence: F::one(),
            gravity: F::one(),
            strong_gravity_mode: false,
            // outbound_attraction_distribution: false,
            scaling_ratio: F::one(),
            slow_down: F::one(),
            parallel: false,
        }
    }
}

impl<F: Float> FA2Settings<F> {
    /// Automatically inferring suitable settings based on a graph's order (its
    /// number of nodes).
    ///
    // Ref: https://github.com/graphology/graphology/blob/249ec5e668ff5e89bf37a10330981579f8759525/src/layout-forceatlas2/index.js#L87
    pub fn from_graph_order(order: usize) -> Self {
        Self {
            repulsion_mode: if order >= 2048 {
                RepulsionMode::BarnesHut {
                    theta: F::from(0.5).unwrap(),
                }
            } else {
                RepulsionMode::Pairwise
            },
            strong_gravity_mode: true,
            gravity: F::from(0.05).unwrap(),
            scaling_ratio: F::from(10.0).unwrap(),
            slow_down: F::one() + F::from(order).unwrap().ln(),
            ..Default::default()
        }
    }

    /// Set the layout to run in parallel (using multiple threads).
    pub fn parallel(mut self, yes: bool) -> Self {
        self.parallel = yes;
        self
    }

    /// Set the layout to use pairwise repulsion (`O(n²)`).
    pub fn with_pairwise_repulsion(mut self) -> Self {
        self.repulsion_mode = RepulsionMode::Pairwise;
        self
    }

    /// Set the layout to use Barnes-Hut repulsion using the provided theta
    /// parameter (`O(n log(n))`).
    pub fn with_barnes_hut_theta(mut self, theta: F) -> Self {
        self.repulsion_mode = RepulsionMode::BarnesHut { theta };
        self
    }

    /// Set the layout to use Barnes-Hut repulsion using the default `0.5` theta
    /// parameter (`O(n log(n))`).
    pub fn with_barnes_hut(self) -> Self {
        self.with_barnes_hut_theta(F::from(0.5).unwrap())
    }

    /// Set the `edge_weight_influence` setting. Defaults to `1.0`.
    pub fn edge_weight_influence(mut self, edge_weight_influence: F) -> Self {
        self.edge_weight_influence = edge_weight_influence;
        self
    }

    /// Set the `gravity` setting. Defaults to `1.0`.
    pub fn gravity(mut self, gravity: F) -> Self {
        self.gravity = gravity;
        self
    }

    /// Set the `gravity` setting. Defaults to `1.0`.
    pub fn scaling_ratio(mut self, scaling_ratio: F) -> Self {
        self.scaling_ratio = scaling_ratio;
        self
    }

    /// Set the `slow_down` setting. Defaults to `1.0`.
    pub fn slow_down(mut self, slow_down: F) -> Self {
        self.slow_down = slow_down;
        self
    }

    /// Set whether to use stronger gravity. Defaults to no.
    pub fn strong_gravity_mode(mut self, yes: bool) -> Self {
        self.strong_gravity_mode = yes;
        self
    }

    pub(crate) fn unwrap_barnes_hut_theta(&self) -> F {
        match &self.repulsion_mode {
            RepulsionMode::BarnesHut { theta } => *theta,
            _ => panic!("not using barnes hut!"),
        }
    }

    /// Build a [`FA2Layout`] layout runner wrapping given [`FA2Data`].
    pub fn build(self, data: FA2Data<F>) -> FA2Layout<F> {
        FA2Layout::new(self, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_graph_order() {
        let settings = FA2Settings::<f32>::from_graph_order(32);

        assert_eq!(settings.slow_down, 4.465736);

        let settings = FA2Settings::<f64>::from_graph_order(32);

        assert_eq!(settings.slow_down, 4.465735902799727);
    }
}
