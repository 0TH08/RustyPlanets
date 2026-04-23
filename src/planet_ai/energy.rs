// src/planet_ai/energy.rs
//! Energy cell management for Skycartel (RustyPlanet, Type A).
//!
//! Type A has 5 energy cells.
//! - Resource cells: [0..3)
//! - Defense cells:  [3..5)
#![allow(dead_code)]

use std::ops::Range;

use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::PlanetState;
use common_game::components::sunray::Sunray;

use crate::planet_ai::errors::SkycartelError;

const TOTAL_CELLS: usize = 5;
const RESOURCE_CELLS: usize = 3;

const fn resource_range() -> Range<usize> {
    0..RESOURCE_CELLS
}

const fn defense_range() -> Range<usize> {
    RESOURCE_CELLS..TOTAL_CELLS
}

#[derive(Debug, Clone)]
pub(crate) struct EnergyManager {
    total_cells: usize,
    resource_cells: usize,
}

impl EnergyManager {
    pub(crate) fn new(cell_count: usize) -> Result<Self, SkycartelError> {
        if cell_count != TOTAL_CELLS {
            return Err(SkycartelError::Energy(format!(
                "Type A requires exactly {TOTAL_CELLS} energy cells, got {cell_count}"
            )));
        }

        Ok(Self {
            total_cells: cell_count,
            resource_cells: RESOURCE_CELLS,
        })
    }

    pub(crate) fn total_cells(&self) -> usize {
        self.total_cells
    }

    pub(crate) fn charge_with_sunray(
        &self,
        state: &mut PlanetState,
        sunray: Sunray,
    ) -> Result<(), SkycartelError> {
        if let Some(i) = first_discharged_index(state, resource_range()) {
            state.cell_mut(i).charge(sunray);
            return Ok(());
        }

        if let Some(i) = first_discharged_index(state, defense_range()) {
            state.cell_mut(i).charge(sunray);
            return Ok(());
        }

        Err(SkycartelError::Energy(
            "No discharged cells available to charge".to_string(),
        ))
    }

    pub(crate) fn with_resource_cell<F, T>(
        &self,
        state: &mut PlanetState,
        f: F,
    ) -> Result<T, SkycartelError>
    where
        F: FnOnce(&mut EnergyCell) -> Result<T, String>,
    {
        self.with_charged_cell_in(state, resource_range(), f)
            .map_err(SkycartelError::Energy)
    }

    pub(crate) fn with_defense_cell<F, T>(
        &self,
        state: &mut PlanetState,
        f: F,
    ) -> Result<T, SkycartelError>
    where
        F: FnOnce(&mut EnergyCell) -> Result<T, String>,
    {
        self.with_charged_cell_in(state, defense_range(), f)
            .map_err(SkycartelError::Energy)
    }

    pub(crate) fn charged_resource_cells(&self, state: &PlanetState) -> usize {
        (0..self.resource_cells)
            .filter(|&i| state.cell(i).is_charged())
            .count()
    }

    pub(crate) fn charged_defense_cells(&self, state: &PlanetState) -> usize {
        (self.resource_cells..self.total_cells)
            .filter(|&i| state.cell(i).is_charged())
            .count()
    }

    fn with_charged_cell_in<F, T>(
        &self,
        state: &mut PlanetState,
        range: Range<usize>,
        f: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&mut EnergyCell) -> Result<T, String>,
    {
        for i in range {
            let cell = state.cell_mut(i);
            if cell.is_charged() {
                return f(cell);
            }
        }
        Err("No charged energy cells available in selected range".to_string())
    }
}

fn first_discharged_index(state: &PlanetState, range: Range<usize>) -> Option<usize> {
    for i in range {
        if i >= state.cells_count() {
            break;
        }
        if !state.cell(i).is_charged() {
            return Some(i);
        }
    }
    None
}
