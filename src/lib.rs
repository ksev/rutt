mod zero;

use std::{cmp::Ordering, collections::binary_heap::BinaryHeap, ops::Add};

use fxhash::FxHashMap;
use fxhash::FxHashSet;

pub use zero::Zero;

struct Node<T, C>
where
    C: Ord + Eq,
{
    vertex: T,
    distance: C,
    steps: usize,
    parent: Option<usize>,
}

struct Token<C>(usize, C);

impl<C> Ord for Token<C>
where
    C: PartialEq + Ord + PartialOrd,
{
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.1.cmp(&self.1)
    }
}

impl<C> PartialOrd for Token<C>
where
    C: PartialEq + Ord + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.1.cmp(&self.1))
    }
}

impl<C> PartialEq for Token<C>
where
    C: PartialEq + Ord + PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C> Eq for Token<C>
where
    C: PartialEq + Ord + PartialOrd,
{
    fn assert_receiver_is_total_eq(&self) {
        self.0.assert_receiver_is_total_eq()
    }
}

pub struct SearchContext<T, C>
where
    C: Ord,
{
    node_storage: Vec<Node<T, C>>,
    open_set: BinaryHeap<Token<C>>,
    open_set_index: FxHashMap<T, usize>,
    closed_set: FxHashSet<T>,
    neighbours: Vec<(T, C)>,
}

impl<T, C> Default for SearchContext<T, C>
where
    C: Ord,
{
    fn default() -> Self {
        SearchContext {
            node_storage: Vec::new(),
            open_set: BinaryHeap::new(),
            open_set_index: FxHashMap::default(),
            closed_set: FxHashSet::default(),
            neighbours: Vec::new(),
        }
    }
}

impl<T, C> SearchContext<T, C>
where
    C: Ord,
{
    pub fn clear(&mut self) {
        self.open_set.clear();
        self.closed_set.clear();
        self.open_set_index.clear();

        self.node_storage.clear();
    }
}

pub trait GraphSearch<'a> {
    type Vertex: Copy + std::hash::Hash + Eq;
    type Cost: Ord + Copy + Zero + Add<Output = Self::Cost>;

    const MAXITERATIONS: usize = usize::MAX;

    fn heuristic<'b: 'a>(&'b self, start: Self::Vertex, goal: Self::Vertex) -> Self::Cost;

    fn neighbours<'b: 'a>(
        &'b self,
        origin: Self::Vertex,
        neighbours: &mut Vec<(Self::Vertex, Self::Cost)>,
    );

    /// Find the shortest path between the start and the goal
    ///
    /// This is the slower but easier to use version, if you want control over allocation and buffer reuse use [find_path_with_context] instead
    fn find_path<'b: 'a>(&'b self, start: Self::Vertex, goal: Self::Vertex) -> Vec<Self::Vertex> {
        let mut context = SearchContext::default();
        let mut path = vec![];

        self.find_path_with_context(&mut context, start, goal, &mut path);

        path
    }

    fn find_path_with_context<'b: 'a>(
        &'b self,
        context: &mut SearchContext<Self::Vertex, Self::Cost>,
        start: Self::Vertex,
        goal: Self::Vertex,
        path: &mut Vec<Self::Vertex>,
    ) {
        context.clear();
        path.clear();

        let idx = context.node_storage.len();

        context.node_storage.push(Node {
            vertex: start,
            distance: Self::Cost::ZERO,
            parent: None,
            steps: 0,
        });

        context.open_set.push(Token(idx, Self::Cost::ZERO));

        let mut iter = 0;

        while let Some(Token(id, _)) = context.open_set.pop() {
            let current = unsafe { context.node_storage.get_unchecked(id) };

            if current.vertex == goal || iter >= Self::MAXITERATIONS {
                // Fill the path buffer
                // Make sure we have the correct amount
                let buffer_size = current.steps + 1;

                path.reserve(buffer_size);

                let mut next = Some(current);
                let mut count = 0;

                while let Some(current) = next {
                    unsafe {
                        *path.get_unchecked_mut(current.steps) = current.vertex;
                    };

                    next = current
                        .parent
                        .map(|p| unsafe { context.node_storage.get_unchecked(p) });

                    count += 1;
                }

                debug_assert_eq!(count, buffer_size);

                // Safety: We've done our due dilligence with the reserve call uptop
                // And assert checking that we actually do initialize all the memory we've requested
                unsafe {
                    path.set_len(buffer_size);
                }

                return;
            }

            iter += 1;
            context.closed_set.insert(current.vertex);

            let next_steps = current.steps + 1;
            let current_distance = current.distance;

            // Ask the implementor to define the neighbours
            self.neighbours(current.vertex, &mut context.neighbours);

            for (vertex, distance) in context.neighbours.drain(..) {
                if context.closed_set.contains(&vertex) {
                    continue;
                }

                let g = current_distance + distance;
                let cost = g + self.heuristic(vertex, goal);

                match context.open_set_index.get(&vertex) {
                    Some(index) => {
                        let n = unsafe { context.node_storage.get_unchecked_mut(*index) };
                        if g < n.distance {
                            n.distance = g;
                            context.open_set.push(Token(idx, cost));
                        }
                    }

                    None => {
                        let idx = context.node_storage.len();
                        context.node_storage.push(Node {
                            vertex,
                            distance: g,
                            parent: Some(id),
                            steps: next_steps,
                        });

                        context.open_set_index.insert(vertex, idx);
                        context.open_set.push(Token(idx, cost));
                    }
                }
            }
        }
        return;
    }

    /*
    fn find_path_incremental(&self, from: &Self::Vertex, to: &Self::Vertex, path: &mut IncrementalPath<Self::Vertex>) {
        todo!("Not done yet");
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct Grid {
        size: isize,
    }

    impl Grid {
        fn new(size: usize) -> Grid {
            Grid {
                size: size as isize,
            }
        }
    }

    impl<'a> GraphSearch<'a> for Grid {
        type Cost = isize;
        type Vertex = (isize, isize);

        fn heuristic<'b: 'a>(&'b self, from: Self::Vertex, to: Self::Vertex) -> Self::Cost {
            // This heuristic is simply manhattan distance
            (from.0 - to.0).abs() + (from.1 - to.1).abs()
        }

        fn neighbours<'b: 'a>(
            &'b self,
            origin: Self::Vertex,
            neighbours: &mut Vec<(Self::Vertex, Self::Cost)>,
        ) {
            let (x, y) = origin;

            if x > 0 {
                neighbours.push(((x - 1, y), 1));
            }

            if x < self.size {
                neighbours.push(((x + 1, y), 1));
            }

            if y > 0 {
                neighbours.push(((x, y - 1), 1));
            }

            if y < self.size {
                neighbours.push(((x, y + 1), 1));
            }
        }
    }

    #[test]
    fn finds_result() {
        let grid = Grid::new(20);

        let from = (2, 3);
        let to = (3, 2);

        let path = grid.find_path(from, to);

        assert_eq!(path[path.len() - 1], to);
    }
}
