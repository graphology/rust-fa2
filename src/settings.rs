use num::Float;

#[derive(Debug, Clone)]
pub struct Settings<F: Float> {
    lin_log_mode: bool,
    edge_weight_influence: F,
    gravity: F,
    strong_gravity_mode: bool,
    outbound_attraction_distribution: bool,
    scaling_ratio: F,
    slow_down: F,
}

impl<F: Float> Default for Settings<F> {
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

impl<F: Float> Settings<F> {
    pub fn from_graph_order(order: usize) -> Self {
        let mut settings = Self::default();

        settings.strong_gravity_mode = true;
        settings.gravity = F::from(0.05).unwrap();
        settings.scaling_ratio = F::from(10.0).unwrap();
        settings.slow_down = F::one() + F::from((order as f64).ln()).unwrap();

        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_graph_order() {
        let settings = Settings::<f32>::from_graph_order(32);

        assert_eq!(settings.slow_down, 4.465736);

        let settings = Settings::<f64>::from_graph_order(32);

        assert_eq!(settings.slow_down, 4.465735902799727);
    }
}
