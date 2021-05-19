use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rutt::{SearchContext, GraphSearch};

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

    #[inline]
    fn heuristic<'b: 'a>(&'b self, from: Self::Vertex, to: Self::Vertex) -> Self::Cost {
        // This heuristic is simply manhattan distance
        (from.0 - to.0).abs() + (from.1 - to.1).abs()
    }

    #[inline]
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

fn criterion_benchmark(c: &mut Criterion) {
    let grid = Grid::new(10);

    let mut ctx = SearchContext::default();
    let mut out = vec![];

    let from = (1, 1);
    let to = (8,9);

    c.bench_function("small", |b| b.iter(|| {
        grid.find_path(black_box(from), black_box(to));
    }));

    c.bench_function("small-context", |b| b.iter(|| {
        grid.find_path_with_context(&mut ctx, black_box(from), black_box(to), &mut out);
    }));

    let grid = Grid::new(100);

    let mut ctx = SearchContext::default();
    let mut out = vec![];

    let from = (1, 1);
    let to = (80, 40);

    c.bench_function("large", |b| b.iter(|| {
        grid.find_path(black_box(from), black_box(to));
    }));
    
    c.bench_function("large-context", |b| b.iter(|| {
        grid.find_path_with_context(&mut ctx, black_box(from), black_box(to), &mut out);
    }));
    
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);