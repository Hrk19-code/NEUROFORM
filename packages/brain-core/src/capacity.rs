//! Capacity model — fixed capacity per tier (DESIGN.md §4.0).
//!
//! A Brain File is created with a fixed maximum capacity. The ledger accounts
//! bytes per shard and slot budgets per store. In M0 the ledger performs
//! accounting and admission *flags*; enforcement (pruning) arrives with the
//! memory stores in M1.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierName {
    Prototype,
    Standard,
    Advanced,
    Experimental,
}

impl TierName {
    pub fn rank(&self) -> u8 {
        match self {
            TierName::Prototype => 0,
            TierName::Standard => 1,
            TierName::Advanced => 2,
            TierName::Experimental => 3,
        }
    }

    pub fn next(&self) -> Option<TierName> {
        match self {
            TierName::Prototype => Some(TierName::Standard),
            TierName::Standard => Some(TierName::Advanced),
            TierName::Advanced => Some(TierName::Experimental),
            TierName::Experimental => None,
        }
    }

    pub fn from_rank(r: u8) -> TierName {
        match r {
            0 => TierName::Prototype,
            2 => TierName::Advanced,
            3 => TierName::Experimental,
            _ => TierName::Standard,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TierName::Prototype => "prototype",
            TierName::Standard => "standard",
            TierName::Advanced => "advanced",
            TierName::Experimental => "experimental",
        }
    }

    pub fn from_str(s: &str) -> Option<TierName> {
        match s {
            "prototype" => Some(TierName::Prototype),
            "standard" => Some(TierName::Standard),
            "advanced" => Some(TierName::Advanced),
            "experimental" => Some(TierName::Experimental),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tier {
    pub name: &'static str,
    pub episodic_slots: u64,
    pub semantic_nodes: u64,
    pub semantic_edges: u64,
    pub procedural_units: u64,
    pub dream_log: u64,
    pub latent_dim: usize,
    pub file_cap_bytes: u64,
}

impl Tier {
    pub fn get(t: TierName) -> Tier {
        match t {
            TierName::Prototype => Tier {
                name: "prototype",
                episodic_slots: 6_000,
                semantic_nodes: 2_000,
                semantic_edges: 10_000,
                procedural_units: 1_500,
                dream_log: 500,
                latent_dim: 192,
                file_cap_bytes: 64 * 1024 * 1024,
            },
            TierName::Standard => Tier {
                name: "standard",
                episodic_slots: 50_000,
                semantic_nodes: 20_000,
                semantic_edges: 100_000,
                procedural_units: 10_000,
                dream_log: 5_000,
                latent_dim: 256,
                file_cap_bytes: 512 * 1024 * 1024,
            },
            TierName::Advanced => Tier {
                name: "advanced",
                episodic_slots: 200_000,
                semantic_nodes: 80_000,
                semantic_edges: 400_000,
                procedural_units: 40_000,
                dream_log: 20_000,
                latent_dim: 384,
                file_cap_bytes: 2 * 1024 * 1024 * 1024,
            },
            TierName::Experimental => Tier {
                name: "experimental",
                episodic_slots: 800_000,
                semantic_nodes: 320_000,
                semantic_edges: 1_600_000,
                procedural_units: 160_000,
                dream_log: 80_000,
                latent_dim: 512,
                file_cap_bytes: 8 * 1024 * 1024 * 1024,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShardUsage {
    pub shard_id: String,
    pub bytes: u64,
    pub budget_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CapacityLedger {
    pub tier: String,
    pub total_bytes: u64,
    pub total_budget: u64,
    pub shards: Vec<ShardUsage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Ok,
    Flag,     // > 90% of budget — flag for next sleep prune
    Critical, // at/over budget — admission control engages
}

impl CapacityLedger {
    pub fn new(tier: &Tier) -> Self {
        CapacityLedger {
            tier: tier.name.to_string(),
            total_bytes: 0,
            total_budget: tier.file_cap_bytes,
            shards: Vec::new(),
        }
    }

    pub fn register(&mut self, shard_id: &str, bytes: u64, budget_bytes: u64) {
        self.total_bytes += bytes;
        match self.shards.iter_mut().find(|s| s.shard_id == shard_id) {
            Some(s) => {
                self.total_bytes = self.total_bytes.saturating_sub(s.bytes);
                s.bytes = bytes;
                s.budget_bytes = budget_bytes;
                self.total_bytes = self.total_bytes.saturating_add(bytes);
            }
            None => self.shards.push(ShardUsage {
                shard_id: shard_id.to_string(),
                bytes,
                budget_bytes,
            }),
        }
    }

    pub fn fullness(&self) -> f32 {
        if self.total_budget == 0 {
            return 1.0;
        }
        self.total_bytes as f32 / self.total_budget as f32
    }

    pub fn check_write(&self, shard_id: &str, bytes: u64) -> Admission {
        let projected = self.total_bytes + bytes;
        if projected >= self.total_budget {
            Admission::Critical
        } else if projected as f32 / self.total_budget as f32 > 0.9 {
            Admission::Flag
        } else {
            let _ = shard_id;
            Admission::Ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_accounts_and_flags() {
        let tier = Tier::get(TierName::Prototype);
        let mut l = CapacityLedger::new(&tier);
        l.register("STATE", 4096, tier.file_cap_bytes / 8);
        assert_eq!(l.total_bytes, 4096);
        assert!(matches!(l.check_write("STATE", 1024), Admission::Ok));
        // Register near-cap and expect flags
        let mut big = CapacityLedger::new(&tier);
        big.register("STATE", tier.file_cap_bytes * 89 / 100, tier.file_cap_bytes / 8);
        assert!(matches!(big.check_write("STATE", tier.file_cap_bytes * 2 / 100), Admission::Flag));
        assert!(matches!(big.check_write("STATE", tier.file_cap_bytes / 4), Admission::Critical));
    }

    #[test]
    fn tier_table_matches_spec() {
        let s = Tier::get(TierName::Standard);
        assert_eq!(s.episodic_slots, 50_000);
        assert_eq!(s.semantic_nodes, 20_000);
        assert_eq!(s.latent_dim, 256);
        assert_eq!(s.file_cap_bytes, 512 * 1024 * 1024);
    }
}
