use std::{
	fmt::Display,
	sync::{Arc, Mutex},
	thread::JoinHandle,
};

use arbitrary::Arbitrary;
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Clone)]
pub struct EvolutionSettings<T> {
	/// How many mutants to inject each generation with?
	/// A mutant is a new randomly-generated creature, which will always be included in reproduction.
	pub mutant_count: usize,
	/// How many instances to carry over between generations?
	pub survivor_count: usize,
	/// How many instances to have each generation?
	pub instance_count: usize,
	/// What Rating is "good enough"?
	pub good_enough: T,
}

/// The trait which allows evolving an Evolvable.
pub trait Evolvable: Sized + Send + Sync + Clone + 'static {
	/// A rating for an Evolvable.
	type Rating: PartialOrd + Sized + Send + Sync + Display + Clone + 'static;

	/// Generate a new random instance of Evolvable.
	fn gen() -> Self;
	/// Rate an Evolvable.
	fn rate(&self) -> Self::Rating;
	/// Mix two Evolvable together.
	fn mix(a: &Self, b: &Self) -> Self;

	/// Find the best N population members
	fn best(of: &[Self], amt: usize) -> Vec<(usize, Self::Rating)> {
		let mut of: Vec<(usize, Self::Rating)> = of
			.par_iter()
			.enumerate()
			.map(|(n, v): (usize, &Self)| (n, v.rate()))
			.collect();

		of.par_sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

		of.into_iter().take(amt).collect()
	}
	/// Run a generation for the Evolvable, and return the best Evolvable's rating.
	fn run_generation(
		previous_generation: &mut Vec<Self>,
		settings: &EvolutionSettings<Self::Rating>,
	) -> Self::Rating {
		let mut rng = rand::thread_rng();
		let best = Self::best(&previous_generation, settings.survivor_count);
		// Purge the unfit
		*previous_generation = previous_generation
			.iter()
			.enumerate()
			.filter(|(n, _)| best.iter().any(|(i, _)| i == n))
			.map(|(_, v)| v.clone())
			.collect();
		// Add new mutants
		for _ in 0..settings.mutant_count {
			previous_generation.push(Self::gen())
		}
		let mut new = vec![];
		// Generate all-new creatures
		while new.len() + previous_generation.len() < settings.instance_count {
			let parents: Vec<&Self> = previous_generation.choose_multiple(&mut rng, 2).collect();
			new.push(Self::mix(parents[0], parents[1]))
		}
		previous_generation.append(&mut new);
		best[0].1.clone()
	}
	/// Generate an all-new generation, all mutants.
	fn seed(settings: &EvolutionSettings<Self::Rating>) -> Vec<Self> {
		(0..settings.instance_count)
			.into_iter()
			.map(|_| Self::gen())
			.collect()
	}
	/// Run the evolution until the rating gets to where it should be.
	fn run(
		settings: EvolutionSettings<Self::Rating>,
		r_handle: Option<Arc<Mutex<Self::Rating>>>,
		best_handle: Option<Arc<Mutex<Self>>>,
	) -> JoinHandle<Vec<Self>> {
		std::thread::spawn(move || {
			let mut generation = Self::seed(&settings);
			let mut rating;

			loop {
				rating = Self::run_generation(&mut generation, &settings);
				if let Some(ref r_hand) = r_handle {
					*r_hand.lock().unwrap() = rating.clone();
				}
				if let Some(ref b_hand) = best_handle {
					*b_hand.lock().unwrap() = generation[0].clone();
				}
				if rating <= settings.good_enough {
					break;
				}
			}

			generation
		})
	}
}
