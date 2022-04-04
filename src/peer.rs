//! The module for peer management. It contains the functionality for creating a
//! peer, adding local trust scores, and calculating the global trust score.

use ark_std::{collections::BTreeMap, fmt::Debug, hash::Hash, One, Zero};

/// Configuration trait for the Peer.
pub trait PeerConfig: Clone {
	/// Type for the Peer index.
	type Index: From<usize> + Eq + Hash + Clone + Ord;
}

/// Peer structure.
#[derive(Clone, Debug)]
pub struct Peer<C: PeerConfig> {
	/// The unique identifier of the peer.
	index: C::Index,
	/// Local trust scores of the peer towards other peers.
	local_trust_scores: BTreeMap<C::Index, f64>,
	/// Global trust score of the peer.
	global_trust_score: f64,
	/// Pre-trust score of the peer.
	pre_trust_score: f64,
	/// Did the peer converge?
	is_converged: bool,
}

impl<C: PeerConfig> Peer<C> {
	/// Create a new peer.
	pub fn new(index: C::Index, global_trust_score: f64, pre_trust_score: f64) -> Self {
		Self {
			index,
			local_trust_scores: BTreeMap::new(),
			global_trust_score,
			pre_trust_score,
			is_converged: false,
		}
	}

	/// Add a local trust score towards another peer.
	pub fn add_neighbor(&mut self, peer_index: C::Index, local_trust_value: f64) {
		self.local_trust_scores
			.insert(peer_index, local_trust_value);
	}

	/// Calculate the global trust score.
	pub fn heartbeat(&mut self, neighbors: &[Peer<C>], delta: f64, pre_trust_weight: f64) {
		if self.is_converged {
			return;
		}

		let mut new_global_trust_score = f64::zero();
		for neighbor_j in neighbors.iter() {
			// Skip if the neighbor is the same peer.
			if self.index == neighbor_j.get_index() {
				continue;
			}

			// Compute ti = `c_1i*t_1(k) + c_ji*t_z(k) + ... + c_ni*t_n(k)`
			// We are going through each neighbor and taking their local trust
			// towards peer `i`, and multiplying it by that neighbor's global trust score.
			// This means that neighbors' opinion about peer i is weighted by their global
			// trust score. If a neighbor has a low trust score (is not trusted by the
			// network), their opinion is not taken seriously, compared to neighbors with a
			// high trust score.
			let neighbor_opinion =
				neighbor_j.get_local_trust_score(&self.index) * neighbor_j.get_global_trust_score();
			new_global_trust_score += neighbor_opinion;
		}

		// (1 - a)*ti + a*p_i
		// The new global trust score (ti) is taken into account.
		// It is weighted by the `pre_trust_weight`, which dictates how seriously the
		// pre-trust score is taken.
		new_global_trust_score = (f64::one() - pre_trust_weight) * new_global_trust_score
			+ pre_trust_weight * self.pre_trust_score;

		// Converge if the difference between the new and old global trust score is less
		// than delta.
		let diff = (new_global_trust_score - self.global_trust_score).abs();
		if diff <= delta {
			self.is_converged = true;
		}

		self.global_trust_score = new_global_trust_score;
	}

	/// Check if the peer has converged.
	pub fn is_converged(&self) -> bool {
		self.is_converged
	}

	/// Get global trust score.
	pub fn get_global_trust_score(&self) -> f64 {
		self.global_trust_score
	}

	/// Get pre trust score.
	pub fn get_pre_trust_score(&self) -> f64 {
		self.pre_trust_score
	}

	/// Get the index of the peer.
	pub fn get_index(&self) -> C::Index {
		self.index.clone()
	}

	/// Get the local trust score of the peer towards another peer.
	pub fn get_local_trust_score(&self, i: &C::Index) -> f64 {
		self.local_trust_scores[i]
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[derive(Clone, Debug, PartialEq)]
	struct TestConfig;
	impl PeerConfig for TestConfig {
		type Index = usize;
	}

	#[test]
	fn test_peer_new() {
		let mut peer = Peer::<TestConfig>::new(0, 0.0, 0.4);
		peer.add_neighbor(1, 0.5);
		assert_eq!(peer.get_index(), 0);
		assert_eq!(peer.get_pre_trust_score(), 0.4);
		assert_eq!(peer.get_global_trust_score(), 0.0);
		assert_eq!(peer.get_local_trust_score(&1), 0.5);
	}
}
