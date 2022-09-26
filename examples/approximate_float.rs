use std::sync::{Arc, Mutex};

use rand::{thread_rng, Rng};
use revolve::{EvolutionSettings, Evolvable};

const GOAL: f64 = 69.420;

#[derive(Clone)]
struct Number(f64);

impl Evolvable for Number {
	type Rating = f64;

	fn gen() -> Self {
		Number(thread_rng().gen::<f64>() * 100.0)
	}

	fn rate(&self) -> Self::Rating {
		(self.0 - GOAL).abs()
	}

	fn mix(a: &Self, b: &Self) -> Self {
		Number((a.0 + b.0) / 2.0)
	}
}

const SETTINGS: EvolutionSettings<f64> = EvolutionSettings {
	mutant_count: 1,
	survivor_count: 5,
	instance_count: 15,
	good_enough: 0.01,
};

fn main() {
	let thread = Number::run(SETTINGS.clone(), None, None);
	thread.join().expect("Runner thread died");
}
