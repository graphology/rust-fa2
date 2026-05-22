/// This Barnes-Hut implementation has been carefully designed to be
/// as fast as possible and cache-friendly.
///
/// It has adaptative depth because fixed depth is demonstrably worse
/// for layout quality (you would see hideous grid artifacts).
///
/// It also attempts to subdivide a cell at most 3 times before giving up,
/// forgetting some unlucky nodes and not inserting them within a leaf.
/// Mass accumulation would still happen so the node is not completely
/// useless either. This can happen when two nodes are very very close
/// or on the same exact point, and is necessary to avoid an infinite
/// loop when building the tree. The tree is rebuilt per iteration anyway so
/// this situation is never permanent.
///
/// The tree has a hybrid layout using an enum for node type (empty, internal
/// or leaf), an a SoA scheme to store the tree's regions as vectors.
/// Nodes are therefore not individually heap-allocated and pointer chasing is
/// reduced to a minimum, as well as being cache-friendly.
///
/// Some indices are also kept using non-zero usizes to benefit from niche
/// optimization and save memory to fit cache lines.
///
/// Note also that the structure was optimized for f32 in mind, less so for f64.
///
/// Here are some things that were tried but resulted in worse performance:
///     - Full SoA layout (with masses, compact tag array etc.)
///     - DFS reordering of the tree
///     - Leaf buckets containing multiple nodes (e.g. 8 nodes at once)
///
use std::num::NonZeroUsize;

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
pub struct BarnesHutTree<F: Float> {
    kinds: Vec<RegionKind<F>>,
    center_xs: Vec<F>,
    center_ys: Vec<F>,
    sizes: Vec<F>,
    next_siblings: Vec<Option<NonZeroUsize>>,
}

impl<F: Float> BarnesHutTree<F> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            kinds: Vec::with_capacity(capacity),
            center_xs: Vec::with_capacity(capacity),
            center_ys: Vec::with_capacity(capacity),
            sizes: Vec::with_capacity(capacity),
            next_siblings: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    fn push_region(
        &mut self,
        kind: RegionKind<F>,
        center_x: F,
        center_y: F,
        size: F,
        next_sibling: Option<NonZeroUsize>,
    ) {
        self.kinds.push(kind);
        self.center_xs.push(center_x);
        self.center_ys.push(center_y);
        self.sizes.push(size);
        self.next_siblings.push(next_sibling);
    }

    fn clear(&mut self) {
        self.kinds.clear();
        self.center_xs.clear();
        self.center_ys.clear();
        self.sizes.clear();
        self.next_siblings.clear();
    }

    pub fn reset_with_extent(&mut self, extent: (F, F, F, F)) {
        self.clear();

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

        self.push_region(
            RegionKind::Empty,
            (min_x + max_x) / two,
            (min_y + max_y) / two,
            (max_x - min_x).max(max_y - min_y),
            None,
        );
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
                match self.kinds[region_index] {
                    RegionKind::Internal {
                        first_child: first_child_index,
                        ..
                    } => {
                        // There are sub-regions, we iterate to delve until we find a leaf
                        let center_x = self.center_xs[region_index];
                        let center_y = self.center_ys[region_index];

                        // Finding the quadrant of n
                        let quadrant = if x < center_x {
                            if y < center_y {
                                Quadrant::TopLeft
                            } else {
                                Quadrant::BottomLeft
                            }
                        } else if y < center_y {
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
                        } = self.kinds[region_index]
                        {
                            *mass_center_x = (*mass_center_x * *mass + x * m) / (*mass + m);
                            *mass_center_y = (*mass_center_y * *mass + y * m) / (*mass + m);
                            *mass += m;
                        }

                        // Continue on relevant quadrant
                        region_index = quadrant.offset(first_child_index);
                    }

                    RegionKind::Leaf { node: region_node } => {
                        let center_x = self.center_xs[region_index];
                        let center_y = self.center_ys[region_index];
                        let next_sibling = self.next_siblings[region_index];
                        let half_size = self.sizes[region_index] / two;

                        let old_node_x = xs[region_node];
                        let old_node_y = ys[region_node];
                        let old_node_mass = ms[region_node];

                        let old_node_quadrant = if old_node_x < center_x {
                            if old_node_y < center_y {
                                Quadrant::TopLeft
                            } else {
                                Quadrant::BottomLeft
                            }
                        } else if old_node_y < center_y {
                            Quadrant::TopRight
                        } else {
                            Quadrant::BottomRight
                        };

                        let new_node_quadrant = if x < center_x {
                            if y < center_y {
                                Quadrant::TopLeft
                            } else {
                                Quadrant::BottomLeft
                            }
                        } else if y < center_y {
                            Quadrant::TopRight
                        } else {
                            Quadrant::BottomRight
                        };

                        self.kinds[region_index] = RegionKind::Internal {
                            first_child: l,
                            mass: old_node_mass,
                            mass_center_x: old_node_x,
                            mass_center_y: old_node_y,
                        };

                        // Pushing regions: TopLeft, BottomLeft, TopRight, BottomRight
                        self.push_region(
                            if old_node_quadrant == Quadrant::TopLeft {
                                RegionKind::Leaf { node: region_node }
                            } else {
                                RegionKind::Empty
                            },
                            center_x - half_size,
                            center_y - half_size,
                            half_size,
                            Some(unsafe { NonZeroUsize::new_unchecked(l + 1) }),
                        );
                        self.push_region(
                            if old_node_quadrant == Quadrant::BottomLeft {
                                RegionKind::Leaf { node: region_node }
                            } else {
                                RegionKind::Empty
                            },
                            center_x - half_size,
                            center_y + half_size,
                            half_size,
                            Some(unsafe { NonZeroUsize::new_unchecked(l + 2) }),
                        );
                        self.push_region(
                            if old_node_quadrant == Quadrant::TopRight {
                                RegionKind::Leaf { node: region_node }
                            } else {
                                RegionKind::Empty
                            },
                            center_x + half_size,
                            center_y - half_size,
                            half_size,
                            Some(unsafe { NonZeroUsize::new_unchecked(l + 3) }),
                        );
                        self.push_region(
                            if old_node_quadrant == Quadrant::BottomRight {
                                RegionKind::Leaf { node: region_node }
                            } else {
                                RegionKind::Empty
                            },
                            center_x + half_size,
                            center_y + half_size,
                            half_size,
                            next_sibling,
                        );

                        l += 4;

                        if old_node_quadrant == new_node_quadrant {
                            subdivision_attempts -= 1;

                            if subdivision_attempts > 0 {
                                region_index = old_node_quadrant.offset(l - 4);
                                continue;
                            } else {
                                break;
                            }
                        }

                        self.kinds[new_node_quadrant.offset(l - 4)] = RegionKind::Leaf { node: n };
                        break;
                    }

                    RegionKind::Empty => {
                        // There is no node in this region, we add it
                        self.kinds[region_index] = RegionKind::Leaf { node: n };
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

        let mut region_index = 0;

        let x = xs[n];
        let y = ys[n];
        let m = ms[n];

        loop {
            match self.kinds[region_index] {
                RegionKind::Internal {
                    first_child: first_child_index,
                    mass,
                    mass_center_x,
                    mass_center_y,
                } => {
                    let x_dist = x - mass_center_x;
                    let y_dist = y - mass_center_y;

                    let distance = x_dist * x_dist + y_dist * y_dist;

                    let size = self.sizes[region_index];

                    if (four * size * size) / distance < theta_squared {
                        // We treat the region as a single body for repulsion
                        if distance > F::zero() {
                            let factor = (coefficient * m * mass) / distance;

                            *out_x += x_dist * factor;
                            *out_y += y_dist * factor;
                        }

                        // Moving to next sibling
                        if let Some(next_sibling_index) = self.next_siblings[region_index] {
                            region_index = next_sibling_index.get();
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        // This region is too close, we delve
                        region_index = first_child_index;
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
                    if let Some(next_sibling_index) = self.next_siblings[region_index] {
                        region_index = next_sibling_index.get();
                        continue;
                    } else {
                        break;
                    }
                }

                RegionKind::Empty => {
                    if let Some(next_sibling_index) = self.next_siblings[region_index] {
                        region_index = next_sibling_index.get();
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

    #[inline]
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
            self.kinds.iter().flat_map(|kind| {
                if let RegionKind::Leaf { node } = kind {
                    Some(*node)
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

        assert_eq!(tree.kinds.len(), 13);
        assert_eq!(tree.nodes().collect::<Vec<_>>(), [4, 2, 3, 0]);
    }
}
