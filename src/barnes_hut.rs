use rayon::prelude::*;

use crate::settings::FA2Settings;
use crate::traits::Float;

const SUBDIVISION_ATTEMPTS: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum Quadrant {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

impl Quadrant {
    #[inline(always)]
    fn offset(self, index: usize) -> usize {
        match self {
            Self::TopLeft => index,
            Self::BottomLeft => index + 1,
            Self::TopRight => index + 2,
            Self::BottomRight => index + 3,
        }
    }
}

#[derive(Debug, PartialEq)]
enum RegionKind<F: Float> {
    Empty,
    Leaf {
        node: usize,
    },
    Internal {
        first_child: usize,
        mass: F,
        mass_center_x: F,
        mass_center_y: F,
    },
}

#[derive(Debug, PartialEq)]
struct BarnesHutRegion<F: Float> {
    kind: RegionKind<F>,
    center_x: F,
    center_y: F,
    size: F,
    next_sibling: Option<usize>,
}

impl<F: Float> BarnesHutRegion<F> {
    fn new_root(min_x: F, max_x: F, min_y: F, max_y: F) -> Self {
        let two = F::from(2.0).unwrap();

        Self {
            kind: RegionKind::Empty,
            center_x: (min_x + max_x) / two,
            center_y: (min_y + max_y) / two,
            size: (max_x - min_x).max(max_y - min_y),
            next_sibling: None,
        }
    }

    fn split(
        &self,
        from_index: usize,
        parent_sibling: Option<usize>,
        quadrant: Quadrant,
        size: F,
    ) -> Self {
        let (next_sibling, center_x, center_y) = match quadrant {
            Quadrant::TopLeft => (
                Some(from_index + 1),
                self.center_x - size,
                self.center_y - size,
            ),
            Quadrant::BottomLeft => (
                Some(from_index + 2),
                self.center_x - size,
                self.center_y + size,
            ),
            Quadrant::TopRight => (
                Some(from_index + 3),
                self.center_x + size,
                self.center_y - size,
            ),
            Quadrant::BottomRight => (parent_sibling, self.center_x + size, self.center_y + size),
        };

        Self {
            kind: RegionKind::Empty,
            center_x,
            center_y,
            size,
            next_sibling,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BarnesHutTree<F: Float> {
    regions: Vec<BarnesHutRegion<F>>,
}

impl<F: Float> BarnesHutTree<F> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            regions: Vec::with_capacity(capacity),
        }
    }

    pub fn reset_with_extent(&mut self, extent: (F, F, F, F)) {
        self.regions.clear();

        let (mut min_x, mut max_x, mut min_y, mut max_y) = extent;

        // Squarifying bounds
        let dx = max_x - min_x;
        let dy = max_y - min_y;

        let two = F::from(2.0).unwrap();

        if dx > dy {
            min_y -= (dx - dy) / two;
            max_y = min_y + dx;
        } else {
            min_x -= (dy - dx) / two;
            max_x = min_x + dy;
        }

        self.regions
            .push(BarnesHutRegion::new_root(min_x, max_x, min_y, max_y));
    }

    pub fn read(&mut self, xs: &[F], ys: &[F], ms: &[F]) {
        let two = F::from(2.0).unwrap();

        let mut l: usize = 1;

        for n in 0..xs.len() {
            let mut region_index: usize = 0;
            let mut subdivision_attempts = SUBDIVISION_ATTEMPTS;

            let x = xs[n];
            let y = ys[n];
            let m = ms[n];

            loop {
                match self.regions[region_index].kind {
                    RegionKind::Internal {
                        first_child: first_child_index,
                        ..
                    } => {
                        // There are sub-regions, we iterate to delve until we find a leaf
                        let current_region = &mut self.regions[region_index];

                        // Finding the quadrant of n
                        let quadrant = if x < current_region.center_x {
                            if y < current_region.center_y {
                                Quadrant::TopLeft
                            } else {
                                Quadrant::BottomLeft
                            }
                        } else if y < current_region.center_y {
                            Quadrant::TopRight
                        } else {
                            Quadrant::BottomRight
                        };

                        // Update mass
                        if let RegionKind::Internal {
                            ref mut mass,
                            ref mut mass_center_x,
                            ref mut mass_center_y,
                            ..
                        } = current_region.kind
                        {
                            *mass_center_x = (*mass_center_x * *mass + x * m) / (*mass + m);
                            *mass_center_y = (*mass_center_y * *mass + y * m) / (*mass + m);
                            *mass += m;
                        }

                        // Continue on relevant quadrant
                        region_index = quadrant.offset(first_child_index);
                    }

                    RegionKind::Leaf { node: region_node } => {
                        // There is a node in this region so we will need
                        // to create sub-regions
                        let current_region = &mut self.regions[region_index];
                        let half_size = current_region.size / two;

                        let mut top_left = current_region.split(
                            l,
                            current_region.next_sibling,
                            Quadrant::TopLeft,
                            half_size,
                        );

                        let mut bottom_left = current_region.split(
                            l,
                            current_region.next_sibling,
                            Quadrant::BottomLeft,
                            half_size,
                        );

                        let mut top_right = current_region.split(
                            l,
                            current_region.next_sibling,
                            Quadrant::TopRight,
                            half_size,
                        );

                        let mut bottom_right = current_region.split(
                            l,
                            current_region.next_sibling,
                            Quadrant::BottomRight,
                            half_size,
                        );

                        // Now we need to be able to put the two nodes in
                        // different sub-regions

                        // Finding old node's quadrant
                        let old_node_x = xs[region_node];
                        let old_node_y = ys[region_node];
                        let old_node_mass = ms[region_node];

                        let old_node_quadrant = if old_node_x < current_region.center_x {
                            if old_node_y < current_region.center_y {
                                top_left.kind = RegionKind::Leaf { node: region_node };
                                Quadrant::TopLeft
                            } else {
                                bottom_left.kind = RegionKind::Leaf { node: region_node };
                                Quadrant::BottomLeft
                            }
                        } else if old_node_y < current_region.center_y {
                            top_right.kind = RegionKind::Leaf { node: region_node };
                            Quadrant::TopRight
                        } else {
                            bottom_right.kind = RegionKind::Leaf { node: region_node };
                            Quadrant::BottomRight
                        };

                        current_region.kind = RegionKind::Internal {
                            first_child: l,
                            mass: old_node_mass,
                            mass_center_x: old_node_x,
                            mass_center_y: old_node_y,
                        };

                        // Finding the quadrant of n
                        let new_node_quadrant = if x < current_region.center_x {
                            if y < current_region.center_y {
                                Quadrant::TopLeft
                            } else {
                                Quadrant::BottomLeft
                            }
                        } else if y < current_region.center_y {
                            Quadrant::TopRight
                        } else {
                            Quadrant::BottomRight
                        };

                        // Pushing regions
                        self.regions.push(top_left);
                        self.regions.push(bottom_left);
                        self.regions.push(top_right);
                        self.regions.push(bottom_right);

                        l += 4;

                        if old_node_quadrant == new_node_quadrant {
                            // Both nodes fell in the same quadrant
                            subdivision_attempts -= 1;

                            if subdivision_attempts > 0 {
                                region_index = old_node_quadrant.offset(l - 4);
                                continue;
                            } else {
                                // We are out of precision here, we break anyway
                                break;
                            }
                        }

                        // Quadrants are different
                        self.regions[new_node_quadrant.offset(l - 4)].kind =
                            RegionKind::Leaf { node: n };

                        break;
                    }

                    RegionKind::Empty => {
                        // There is no node in this region, we add it
                        self.regions[region_index].kind = RegionKind::Leaf { node: n };
                        break;
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_repulsion_for_node(
        &self,
        settings: &FA2Settings<F>,
        n: usize,
        xs: &[F],
        ys: &[F],
        ms: &[F],
        out_x: &mut F,
        out_y: &mut F,
    ) {
        let coefficient = settings.scaling_ratio;
        let theta_squared = settings.unwrap_barnes_hut_theta().powi(2);
        let four = F::from(4.0).unwrap();

        let mut current_region = &self.regions[0];

        let x = xs[n];
        let y = ys[n];
        let m = ms[n];

        loop {
            match current_region.kind {
                RegionKind::Internal {
                    first_child: first_child_index,
                    mass,
                    mass_center_x,
                    mass_center_y,
                } => {
                    let x_dist = x - mass_center_x;
                    let y_dist = y - mass_center_y;

                    let distance = x_dist * x_dist + y_dist * y_dist;

                    let size = current_region.size;

                    if (four * size * size) / distance < theta_squared {
                        // We treat the region as a single body for repulsion
                        if distance > F::zero() {
                            let factor = (coefficient * m * mass) / distance;

                            *out_x += x_dist * factor;
                            *out_y += y_dist * factor;
                        }

                        // Moving to next sibling
                        if let Some(next_sibling_index) = current_region.next_sibling {
                            current_region = &self.regions[next_sibling_index];
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        // This region is too close, we delve
                        current_region = &self.regions[first_child_index];
                        continue;
                    }
                }

                RegionKind::Leaf { node: region_node } => {
                    if region_node != n {
                        let region_node_x = xs[region_node];
                        let region_node_y = ys[region_node];
                        let region_node_mass = ms[region_node];

                        let x_dist = x - region_node_x;
                        let y_dist = y - region_node_y;

                        let distance = x_dist * x_dist + y_dist * y_dist;

                        if distance > F::zero() {
                            let factor = (coefficient * m * region_node_mass) / distance;

                            *out_x += x_dist * factor;
                            *out_y += y_dist * factor;
                        }
                    }

                    // Moving to next sibling
                    if let Some(next_sibling_index) = current_region.next_sibling {
                        current_region = &self.regions[next_sibling_index];
                        continue;
                    } else {
                        break;
                    }
                }

                RegionKind::Empty => {
                    // Moving to next sibling
                    if let Some(next_sibling_index) = current_region.next_sibling {
                        current_region = &self.regions[next_sibling_index];
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn apply_repulsion(
        &self,
        settings: &FA2Settings<F>,
        xs: &[F],
        ys: &[F],
        ms: &[F],
        out_xs: &mut [F],
        out_ys: &mut [F],
    ) {
        for n in 0..xs.len() {
            self.apply_repulsion_for_node(settings, n, xs, ys, ms, &mut out_xs[n], &mut out_ys[n]);
        }
    }

    pub fn par_apply_repulsion(
        &self,
        settings: &FA2Settings<F>,
        xs: &[F],
        ys: &[F],
        ms: &[F],
        out_xs: &mut [F],
        out_ys: &mut [F],
    ) {
        out_xs
            .par_iter_mut()
            .zip(out_ys.par_iter_mut())
            .enumerate()
            .for_each(|(n, (out_x, out_y))| {
                self.apply_repulsion_for_node(settings, n, xs, ys, ms, out_x, out_y);
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::data::FA2Data;

    use super::*;

    impl<F: Float> BarnesHutTree<F> {
        fn nodes(&self) -> impl Iterator<Item = usize> + '_ {
            self.regions.iter().flat_map(|region| {
                if let RegionKind::Leaf { node } = region.kind {
                    Some(node)
                } else {
                    None
                }
            })
        }
    }

    #[test]
    fn test_construction() {
        let mut data = FA2Data::new();
        data.add_node(1.0, 9.0);
        data.add_node(-1.0, 8.0);
        data.add_node(4.0, 1.0);
        data.add_node(9.0, 10.0);
        data.add_node(3.0, 0.0);

        let extent = data.positions_extent().unwrap();

        let mut tree = BarnesHutTree::with_capacity(5);
        tree.reset_with_extent(extent);
        tree.read(&data.xs, &data.ys, &data.ms);

        assert_eq!(tree.regions.len(), 13);
        assert_eq!(tree.nodes().collect::<Vec<_>>(), [4, 2, 3, 0]);
    }
}
