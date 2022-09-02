// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::NetworkLoadTest;
use anyhow::bail;
use aptos_logger::warn;
use aptos_sdk::types::account_config::CORE_CODE_ADDRESS;
use forge::test_utils::consensus_utils::{
    test_consensus_fault_tolerance, FailPointFailureInjection, NodeState,
};
use forge::{NetworkContext, NetworkTest, Result, Swarm, SwarmExt, Test};
use rand::Rng;
use std::collections::HashSet;
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct ChangingWorkingQuorumTest {
    pub target_tps: usize,
    pub max_down_nodes: usize,
    pub few_large_validators: bool,
    pub add_execution_delay: bool,
}

impl Test for ChangingWorkingQuorumTest {
    fn name(&self) -> &'static str {
        "changing working quorum test"
    }
}

impl NetworkLoadTest for ChangingWorkingQuorumTest {
    fn test(&self, swarm: &mut dyn Swarm, duration: Duration) -> Result<()> {
        let runtime = Runtime::new().unwrap();

        let validators = swarm.get_clients_with_names();
        let num_validators = validators.len();

        let (num_healthy_validators, can_fail_for_quorum, cycle_offset) =
            if self.few_large_validators {
                let validator_set: serde_json::Value = runtime
                    .block_on(
                        validators[0]
                            .1
                            .get_resource(CORE_CODE_ADDRESS, "0x1::stake::ValidatorSet"),
                    )?
                    .into_inner();
                println!("ValidatorSet : {:?}", validator_set);

                let can_fail_for_quorum =
                    std::cmp::min(self.max_down_nodes, (4 * 10 + (num_validators - 4) - 1) / 3);
                let cycle_offset = can_fail_for_quorum / 3 + 1;
                (4, can_fail_for_quorum, cycle_offset)
            } else {
                let can_fail_for_quorum =
                    std::cmp::min(self.max_down_nodes, (validators.len() - 1) / 3);
                let cycle_offset = can_fail_for_quorum / 3 + 1;
                (0, can_fail_for_quorum, cycle_offset)
            };

        if self.add_execution_delay {
            runtime.block_on(async {
                let mut rng = rand::thread_rng();
                for (_, validator) in &validators[num_healthy_validators..num_validators] {
                    let sleep_time = rng.gen_range(20, 500);

                    validator
                        .set_failpoint(
                            "aptos_vm::execution::block_metadata".to_string(),
                            format!("sleep({})", sleep_time),
                        )
                        .await
                        .unwrap();
                }
            });
        }

        // Check that every 27s all nodes make progress,
        // without any failures.
        // (make epoch length (120s) and this duration (27s) not be multiples of one another,
        // to test different timings)
        let check_period_s: usize = 27;
        let target_tps = self.target_tps;

        runtime.block_on(test_consensus_fault_tolerance(
            swarm,
            duration.as_secs() as usize / check_period_s,
            check_period_s as f32,
            1,
            Box::new(FailPointFailureInjection::new(Box::new(move |cycle, part| {
                if part == 0 {
                    let down_indices: Vec<_> = (0..can_fail_for_quorum).map(|i| {
                        num_healthy_validators + (cycle * cycle_offset + i) % num_validators
                    }).collect();

                    (
                        down_indices.iter().flat_map(|i| {
                            [
                                (
                                    *i,
                                    "consensus::send::any".to_string(),
                                    "return".to_string(),
                                ),
                                (
                                    *i,
                                    "consensus::process::any".to_string(),
                                    "return".to_string(),
                                ),
                            ]
                        }).collect(),
                        true,
                    )
                } else {
                    (vec![], false)
                }
            }))),
            Box::new(move |cycle, _, _, _, cur, previous| {
                let down_indices: HashSet<_> = (0..can_fail_for_quorum).map(|i| {
                    num_healthy_validators + (cycle * cycle_offset + i) % num_validators
                }).collect();

                fn split(all: Vec<NodeState>, down_indices: &HashSet<usize>) -> (Vec<NodeState>, Vec<NodeState>) {
                    let (down, active): (Vec<_>, Vec<_>) = all.into_iter().enumerate().partition(|(idx, _state)| down_indices.contains(idx));
                    (down.into_iter().map(|(_idx, state)| state).collect(), active.into_iter().map(|(_idx, state)| state).collect())
                }

                let (cur_down, cur_active) = split(cur, &down_indices);
                let (previous_down, previous_active) = split(previous, &down_indices);

                // Make sure that every active node is making progress, so we compare min(cur) vs max(previous)
                let epochs = cur_active.iter().map(|s| s.epoch).min().unwrap()
                    - previous_active.iter().map(|s| s.epoch).max().unwrap();
                let rounds = cur_active
                    .iter()
                    .map(|s| s.round)
                    .min()
                    .unwrap()
                    .saturating_sub(previous_active.iter().map(|s| s.round).max().unwrap());
                let transactions = cur_active.iter().map(|s| s.version).min().unwrap()
                    - previous_active.iter().map(|s| s.version).max().unwrap();


                if transactions < (target_tps * check_period_s / 2) as u64 {
                    warn!(
                        "no progress with active consensus, only {} transactions, expected >= {}",
                        transactions,
                        (target_tps * check_period_s / 2),
                    );
                }
                if epochs == 0 && rounds < (check_period_s / 2) as u64 {
                    warn!("no progress with active consensus, only {} epochs and {} rounds, expectd >= {}",
                        epochs,
                        rounds,
                        (check_period_s / 2),
                    );
                }

                // Make sure down nodes don't make progress:
                for (cur_state, prev_state) in cur_down.iter().zip(previous_down.iter()) {
                    if cur_state.round > prev_state.round + 3 {
                        warn!("progress on down node from ({}, {}) to ({}, {})", cur_state.epoch, cur_state.round, prev_state.epoch, prev_state.round);
                    }
                }

                Ok(())
            }),
            false,
        ))?;

        Ok(())
    }
}

impl NetworkTest for ChangingWorkingQuorumTest {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        <dyn NetworkLoadTest>::run(self, ctx)
    }
}
