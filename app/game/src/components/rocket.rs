use crate::components::energy_cell::EnergyCell;
use serde::{Deserialize, Serialize};

/// Planetary defense mechanism against asteroids.
///
/// Rockets are built using charged energy cells and serve as the primary defense
/// mechanism for planets against incoming asteroids. Only planet types A and C can
/// construct and deploy rockets.
///
/// # Construction
///
/// A rocket requires one fully charged [EnergyCell] for construction. The energy cell
/// is consumed (discharged) during the construction process. Construction fails if the
/// provided energy cell is not charged.
///
/// # Deflection Mechanism
///
/// Rockets themselves have no inherent behavior. The actual asteroid deflection logic
/// is handled by the planet's [handle_asteroid()](crate::components::planet::PlanetAI::handle_asteroid)
/// method, which determines whether a rocket is available and returns an owned [Rocket]
/// if deflection is possible.
///
/// # Availability Constraints
///
/// Based on planet type constraints (PDF section 3.7.2):
/// - **Type A**: Rockets allowed (5 energy cells)
/// - **Type B**: Rockets not allowed (1 energy cell)
/// - **Type C**: Rockets allowed (1 energy cell)
/// - **Type D**: Rockets not allowed (5 energy cells)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rocket {
    _private: (),
}

impl Rocket {
    /// Constructs a new [Rocket] from a charged [EnergyCell].
    ///
    /// This method consumes the energy cell's charge during construction. The rocket
    /// is immediately ready for deployment and can be stored in the planet's state.
    ///
    /// # Arguments
    ///
    /// * `energy_cell` - A mutable reference to an [EnergyCell] that must be fully charged.
    ///
    /// # Returns
    ///
    /// - `Ok(Rocket)` if the energy cell was successfully discharged and the rocket was constructed.
    /// - `Err(String)` if the energy cell is not charged (construction fails and the cell remains unchanged).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::components::rocket::Rocket;
    /// use crate::components::energy_cell::EnergyCell;
    /// use crate::components::sunray::Sunray;
    ///
    /// let mut cell = EnergyCell::new();
    /// cell.charge(Sunray::new());
    ///
    /// let rocket = Rocket::new(&mut cell)?;
    /// assert!(!cell.is_charged(), "Cell should be discharged after rocket construction");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(energy_cell: &mut EnergyCell) -> Result<Rocket, String> {
        energy_cell.discharge().map(|_| Rocket { _private: () })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::sunray::Sunray;

    #[test]
    fn test_rocket_construction_success() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());

        let result = Rocket::new(&mut cell);
        assert!(
            result.is_ok(),
            "Rocket construction should succeed with charged cell"
        );
        assert!(
            !cell.is_charged(),
            "Cell should be discharged after rocket construction"
        );
    }

    #[test]
    fn test_rocket_construction_fails_without_charge() {
        let mut cell = EnergyCell::new();

        let result = Rocket::new(&mut cell);
        assert!(
            result.is_err(),
            "Rocket construction should fail with uncharged cell"
        );
        assert!(
            !cell.is_charged(),
            "Cell should remain uncharged after failed construction"
        );
    }

    #[test]
    fn test_rocket_construction_fails_already_discharged() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());
        cell.discharge().unwrap();

        let result = Rocket::new(&mut cell);
        assert!(
            result.is_err(),
            "Rocket construction should fail if cell was already discharged"
        );
    }

    #[test]
    fn test_rocket_is_owned_after_construction() {
        let mut cell = EnergyCell::new();
        cell.charge(Sunray::new());

        let rocket = Rocket::new(&mut cell).expect("Failed to construct rocket");

        assert!(true, "Rocket is owned and ready for deployment");
        drop(rocket);
    }
}
