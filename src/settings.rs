use crate::traits::Float;

#[derive(Debug, Clone)]
pub struct FA2Settings<F: Float> {
    pub(crate) lin_log_mode: bool,
    pub(crate) edge_weight_influence: F,
    pub(crate) gravity: F,
    pub(crate) strong_gravity_mode: bool,
    pub(crate) outbound_attraction_distribution: bool,
    pub(crate) scaling_ratio: F,
    pub(crate) slow_down: F,
}

impl<F: Float> Default for FA2Settings<F> {
    fn default() -> Self {
        Self {
            lin_log_mode: false,
            edge_weight_influence: F::one(),
            gravity: F::one(),
            strong_gravity_mode: false,
            outbound_attraction_distribution: false,
            scaling_ratio: F::one(),
            slow_down: F::one(),
        }
    }
}

impl<F: Float> FA2Settings<F> {
    pub fn from_graph_order(order: usize) -> Self {
        Self {
            strong_gravity_mode: true,
            gravity: F::from(0.05).unwrap(),
            scaling_ratio: F::from(10.0).unwrap(),
            slow_down: F::one() + F::from((order as f64).ln()).unwrap(),
            ..Default::default()
        }
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
