use crate::builder::FA2Data;
use crate::layout::FA2Layout;
use crate::traits::Float;

#[derive(Debug, Clone)]
pub enum RepulsionMode<F: Float> {
    Pairwise,
    BarnesHut { theta: F },
}

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
}

impl<F: Float> Default for FA2Settings<F> {
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
        }
    }
}

impl<F: Float> FA2Settings<F> {
    // Ref: https://github.com/graphology/graphology/blob/249ec5e668ff5e89bf37a10330981579f8759525/src/layout-forceatlas2/index.js#L87
    pub fn from_graph_order(order: usize) -> Self {
        Self {
            strong_gravity_mode: true,
            gravity: F::from(0.05).unwrap(),
            scaling_ratio: F::from(10.0).unwrap(),
            slow_down: F::one() + F::from(order).unwrap().ln(),
            ..Default::default()
        }
    }

    pub fn with_barnes_hut_with_theta(mut self, theta: F) -> Self {
        self.repulsion_mode = RepulsionMode::BarnesHut { theta };
        self
    }

    pub fn with_barnes_hut(self) -> Self {
        self.with_barnes_hut_with_theta(F::from(0.5).unwrap())
    }

    pub fn build<'d>(self, data: &'d mut FA2Data<F>) -> FA2Layout<'d, F> {
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
