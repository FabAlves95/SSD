use std::collections::HashMap;
use std::ops::Index;
use std::sync::{Arc, Mutex};

use log::{debug, error};

use crate::blockchain::block::Block;
use crate::blockchain::consensus::ConsensusAlgorithm;

pub struct Miner {
    consensus: ConsensusAlgorithm,
    pub(self) mining_blocks: Arc<Mutex<HashMap<String, bool>>>, //im mining block id and there is no results (from network)
}

impl Miner {
    pub fn new(consensus: ConsensusAlgorithm) -> Miner {
        let mining = Arc::new(Mutex::new(HashMap::new()));

        Self {
            consensus,
            mining_blocks,
        }
    }

    pub fn mine_block(self, block: Block) -> u128 {
        let mut mining = match self.mining_blocks.lock() {
            Ok(m) => m,
            Err(e) => {
                error!("Unable to decode block string payload: {}", e.to_string());
                panic!("{}", e.to_string());
            }
        };

        mining.insert(block.clone().header.hash, false);

        drop(mining);

        return match self.consensus {
            ConsensusAlgorithm::ProofOfWork => {
                self.proof_of_work(block)
            }
            ConsensusAlgorithm::DelegatedProofOfStake => {
                self.proof_of_stake(block)
            }
        }
    }

    fn proof_of_work(self, block: Block) -> u128 {
        let mut nonce = block.header.nonce.clone();

        while !self.valid_proof(block.clone(), nonce) {
            nonce += 1;

            if nonce % 1000 == 0 {
                let mut mining = match self.mining_blocks.lock() {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Unable to decode block string payload: {}", e.to_string());
                        panic!("{}", e.to_string());
                    }
                };

                if mining.index(&block.header.hash) {
                    break;
                }

                drop(mining)
            }
        }

        nonce
    }

    fn proof_of_stake(self, block: Block) -> u128 {
        let mut nonce = block.header.nonce.clone();

        nonce
    }


    fn valid_proof(&self, block: Block, nonce: u128) -> bool {
        let mut mining_block = block.clone();

        mining_block.header.nonce = nonce;

        mining_block.is_valid()
    }


}