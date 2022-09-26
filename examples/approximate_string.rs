use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use rand::{thread_rng, Rng};
use revolve::{EvolutionSettings, Evolvable};

#[derive(Clone)]
struct Guess(String);

impl Evolvable for Guess {
	type Rating = i32;

	fn gen() -> Self {
		let mut rng = thread_rng();
		let len: u8 = rng.gen();
		let len = len as usize;
		Guess(
			(0..len)
				.into_iter()
				.map(|_| rng.gen::<u8>() as char)
				.filter(|c| c.is_ascii() && !c.is_control())
				.collect(),
		)
	}

	fn rate(&self) -> Self::Rating {
		(TRUTH.len() as i32
			- (self
				.0
				.chars()
				.zip(TRUTH.chars())
				.filter(|(a, b)| a == b)
				.count() as i32))
			+ (((TRUTH.len() as i32) - (self.0.len() as i32)).abs())
	}

	fn mix(a: &Self, b: &Self) -> Self {
		let len = a.0.len().max(b.0.len());
		Guess(
			(0..len)
				.into_iter()
				.map(|n| (a.0.chars().nth(n), b.0.chars().nth(n)))
				.flat_map(|n| match n {
					(Some(a), Some(b)) => {
						if thread_rng().gen_bool(0.5) {
							Some(a)
						} else {
							Some(b)
						}
					}
					(a, b) => {
						if thread_rng().gen_bool(0.7) {
							a.or(b)
						} else {
							None
						}
					}
				})
				.collect(),
		)
	}
}

const SETTINGS: EvolutionSettings<i32> = EvolutionSettings {
	mutant_count: 500,
	survivor_count: 50,
	instance_count: 10000,
	good_enough: 0,
};

const TRUTH: &str = "High-level parallel constructs";

fn main() {
	let gen = Arc::new(Mutex::new(Guess::gen()));
	let r = Arc::new(Mutex::new(Default::default()));
	let thread = Guess::run(SETTINGS.clone(), Some(r.clone()), Some(gen.clone()));
	while !thread.is_finished() {
		std::thread::sleep(Duration::from_secs(1));
		println!("Current fitness: {}", r.lock().unwrap());
		println!("Current leader: {}", gen.lock().unwrap().0);
	}
	thread.join().expect("Worker died");
	println!("Final fitness: {}", r.lock().unwrap());
	println!("Final leader: {}", gen.lock().unwrap().0);
}
