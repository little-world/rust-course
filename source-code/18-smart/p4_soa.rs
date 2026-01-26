// Pattern 4: Memory Layout Optimization - Struct of Arrays (SoA)

// Array of Structs: poor locality when accessing single field
#[allow(dead_code)]
struct ParticleAoS {
    position: [f32; 3],
    velocity: [f32; 3],
    mass: f32,
}

#[allow(dead_code)]
fn update_aos(particles: &mut [ParticleAoS]) {
    for p in particles {
        // CPU loads entire struct even though we only need position and velocity
        p.position[0] += p.velocity[0];
    }
}

// Struct of Arrays: excellent locality
struct ParticlesSoA {
    positions_x: Vec<f32>,
    velocities_x: Vec<f32>,
}

impl ParticlesSoA {
    fn new() -> Self {
        ParticlesSoA {
            positions_x: Vec::new(),
            velocities_x: Vec::new(),
        }
    }

    fn add(&mut self, pos_x: f32, vel_x: f32) {
        self.positions_x.push(pos_x);
        self.velocities_x.push(vel_x);
    }

    fn update(&mut self) {
        // positions_x is contiguous; CPU prefetches efficiently
        for i in 0..self.positions_x.len() {
            self.positions_x[i] += self.velocities_x[i];
        }
    }
}

fn main() {
    let mut particles = ParticlesSoA::new();
    particles.add(0.0, 1.0);
    particles.add(5.0, 2.0);
    particles.add(10.0, 3.0);

    println!("Before update: {:?}", particles.positions_x);
    particles.update();
    println!("After update: {:?}", particles.positions_x);

    println!("Struct of Arrays example completed");
}
