#[derive(Debug, Clone)]
pub struct Settings {
    lin_log_mode: bool,
    edge_weight_influence: f64,
    gravity: f64,
    strong_gravity_mode: bool,
    outbound_attraction_distribution: bool,
    scaling_ratio: f64,
    slow_down: f64
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lin_log_mode: false,
            edge_weight_influence: 1.0,
            gravity: 1.0,
            strong_gravity_mode: false,
            outbound_attraction_distribution: false,
            scaling_ratio: 1.0,
            slow_down: 1.0
        }
    }
}

impl Settings {
    pub fn from_graph_order(order: usize) -> Self {
        let mut settings = Self::default();

        settings.strong_gravity_mode = true;
        settings.gravity = 0.05;
        settings.scaling_ratio = 10.0;
        settings.slow_down = 1.0 + (order as f64).ln();

        settings
    }
}