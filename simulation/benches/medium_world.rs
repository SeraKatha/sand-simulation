use criterion::{criterion_group, criterion_main, Criterion};
use macroquad::math::ivec2;

fn benchmark(c: &mut Criterion) {
    let world_size = ivec2(128, 128);
    if let Ok(mut simulation) = simulation::Simulation::new(world_size, simulation::EmptyWorldGenerator::new()) {
        for y in 0..world_size.y {
            for x in 0..world_size.x {
                let cell = match rand::random::<u32>() % 4 {
                    0 => simulation::Cell::Air,
                    1 => simulation::Cell::Sand,
                    2 => simulation::Cell::Stone,
                    3 => simulation::Cell::Water,
                    _ => simulation::Cell::Air
                };
                simulation.write_cell(ivec2(x, y), cell);
            }
        }
        let id = format!("world_size={}x{}", world_size.x, world_size.y);
        c.bench_function(&id, |b| b.iter(||simulation.tick()));
    }
}

criterion_main!(benches);
criterion_group!(benches, benchmark);