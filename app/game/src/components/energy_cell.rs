use crate::components::sunray::Sunray;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnergyCell {
    charge: bool,
}

impl EnergyCell {
    pub fn new() -> Self {
        Self { charge: false }
    }

    pub fn charge(&mut self, _sunray: Sunray) {
        if !self.charge {
            self.charge = true;
        }
    }

    pub fn discharge(&mut self) -> Result<(), String> {
        if self.charge {
            self.charge = false;
            Ok(())
        } else {
            Err("EnergyCell not charged!".into())
        }
    }

    pub fn is_charged(&self) -> bool {
        self.charge
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::sunray::Sunray;

    #[test]
    fn constructor_creates_uncharged_cell() {
        let cell = EnergyCell::new();
        assert!(!cell.is_charged());
    }

    #[test]
    fn charging_sets_state_to_charged() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());
        assert!(cell.is_charged());
    }

    #[test]
    fn discharge_works_when_charged() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());
        let res = cell.discharge();
        assert!(res.is_ok());
        assert!(!cell.is_charged());
    }

    #[test]
    fn discharge_fails_when_empty() {
        let mut cell = EnergyCell::new();
        let res = cell.discharge();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "EnergyCell not charged!");
    }

    #[test]
    fn charging_already_charged_cell_wastes_sunray() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());
        assert!(cell.is_charged());
        cell.charge(Sunray::new());
        assert!(cell.is_charged());
    }

    #[test]
    fn discharge_failure_does_not_change_state() {
        let mut cell = EnergyCell::new();
        let _ = cell.discharge();
        assert!(!cell.is_charged());
    }

    #[test]
    fn binary_state_constraint() {
        let mut cell = EnergyCell::new();
        assert!(!cell.is_charged());
        cell.charge(Sunray::new());
        assert!(cell.is_charged());
        cell.discharge().unwrap();
        assert!(!cell.is_charged());
    }
}
