use std::{array, sync::LazyLock};

use anyhow::Context;

use crate::LifeTileState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifeRules {
    states_by_alive_neighbor_count: [LifeRuleState; crate::TILE_NEIGHBOR_COUNT + 1],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifeRuleState {
    /// The cell will be dead the next tick, regardless of current state.
    Dead,
    /// The cell will be alive the next tick, regardless of current state.
    Alive,
    /// The cell will be alive the next tick, if it is currently dead.
    BirthOnly,
    /// The cell will be alive the next tick, if it is currently alive.
    SurvivalOnly,
}

impl LifeRules {
    pub const CONWAY: LazyLock<Self> = LazyLock::new(|| Self::parse_golly("B3/S23").unwrap());
    pub const STILL: LazyLock<Self> = LazyLock::new(|| Self::parse_golly("B/S012345678").unwrap());

    pub const fn new(
        states_by_alive_neighbor_count: [LifeRuleState; crate::TILE_NEIGHBOR_COUNT + 1],
    ) -> Self {
        Self {
            states_by_alive_neighbor_count,
        }
    }

    pub fn parse_golly(golly_notation: &str) -> anyhow::Result<Self> {
        let (birth_part, survival_part) = golly_notation.split_once('/').context("Invalid notation, expected slash-delimited birth and survival part")?;
        let birth_part_stripped = birth_part.strip_prefix('B').context("Invalid notation, expected Golly-like 'B' prefixed birth values before slash-delimiter")?;
        let survival_part_stripped = survival_part.strip_prefix('S').context("Invalid notation, expected Golly-like 'S' prefixed survival values after slash-delimiter")?;

        let birth_values: Vec<_> = parse_numeric_chain(birth_part_stripped.chars()).collect();
        let survival_values: Vec<_> = parse_numeric_chain(survival_part_stripped.chars()).collect();

        let states_by_alive_neighbor_count = array::from_fn(|alive_neighbor_count| {
            if birth_values.contains(&alive_neighbor_count) && survival_values.contains(&alive_neighbor_count) {
                LifeRuleState::Alive
            } else if birth_values.contains(&alive_neighbor_count) {
                LifeRuleState::BirthOnly
            } else if survival_values.contains(&alive_neighbor_count) {
                LifeRuleState::SurvivalOnly
            } else {
                LifeRuleState::Dead
            }
        });

        Ok(Self {
            states_by_alive_neighbor_count,
        })
    }

    pub fn to_golly(&self) -> String {
        let birth_values = self
            .states_by_alive_neighbor_count
            .iter()
            .enumerate()
            .filter(|(_, state)| match **state {
                LifeRuleState::BirthOnly | LifeRuleState::Alive => true,
                LifeRuleState::SurvivalOnly | LifeRuleState::Dead => false,
            })
            .map(|(alive_neighbor_count, _)| alive_neighbor_count);
        let survival_values = self
            .states_by_alive_neighbor_count
            .iter()
            .enumerate()
            .filter(|(_, state)| match **state {
                LifeRuleState::SurvivalOnly | LifeRuleState::Alive => true,
                LifeRuleState::BirthOnly | LifeRuleState::Dead => false,
            })
            .map(|(alive_neighbor_count, _)| alive_neighbor_count);

        let mut output_buffer = String::new();

        output_buffer.push('B');
        for birth_value in birth_values {
            output_buffer.push(numeric_number_as_digit(birth_value));
        }

        output_buffer.push('/');

        output_buffer.push('S');
        for survival_value in survival_values {
            output_buffer.push(numeric_number_as_digit(survival_value));
        }

        output_buffer
    }

    pub fn compute_next_tile_state(
        &self,
        prev_tile_state: LifeTileState,
        neighbor_tile_states: &[LifeTileState; 8],
    ) -> LifeTileState {
        let alive_neighbor_count = neighbor_tile_states
            .iter()
            .filter(|neighbor_tile_alive| **neighbor_tile_alive)
            .count();

        match self.states_by_alive_neighbor_count[alive_neighbor_count] {
            LifeRuleState::Dead => false,
            LifeRuleState::Alive => true,
            LifeRuleState::BirthOnly => !prev_tile_state,
            LifeRuleState::SurvivalOnly => prev_tile_state,
        }
    }
}

fn parse_numeric_chain(chars: impl IntoIterator<Item = char>) -> impl Iterator<Item = usize> {
    chars
        .into_iter()
        .filter(|ch| ch.is_numeric())
        .map(parse_numeric_char)
}

fn parse_numeric_char(ch: char) -> usize {
    // We could abuse numeric character indices here. This is obvious to anyone reading though.
    match ch {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        _ => panic!("Not a numeric '{ch}'"),
    }
}

fn numeric_number_as_digit(number: usize) -> char {
    // We could abuse numeric character indices here. This is obvious to anyone reading though.
    match number {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        _ => panic!("Number {number} doesn't fit into one digit"),
    }
}
