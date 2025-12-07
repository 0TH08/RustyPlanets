use crate::components::asteroid::{Asteroid, Vec2};
use crate::components::sunray::Sunray;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref ALREADY_CREATED: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug)]
pub struct Forge {
    _private: (),
}

impl Forge {
    pub fn new() -> Result<Self, String> {
        let mut check = ALREADY_CREATED.lock().unwrap();
        if !*check {
            *check = true;
            Ok(Forge { _private: () })
        } else {
            Err("Another Forge has already been created".into())
        }
    }

    pub fn generate_sunray(&self) -> Sunray {
        Sunray::new()
    }

    pub fn generate_asteroid(&self) -> Asteroid {
        Asteroid::new(Vec2::default(), Vec2::default(), 1.0, 1.0)
    }
}

impl Drop for Forge {
    fn drop(&mut self) {
        let mut check = ALREADY_CREATED.lock().unwrap();
        *check = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_first_creation_succeeds() {
        {
            let mut created = ALREADY_CREATED.lock().unwrap();
            *created = false;
        }

        let forge = Forge::new();
        assert!(forge.is_ok());
    }

    #[test]
    fn test_forge_second_creation_fails() {
        {
            let mut created = ALREADY_CREATED.lock().unwrap();
            *created = false;
        }

        let forge0 = Forge::new();
        assert!(forge0.is_ok());

        let forge1 = Forge::new();
        assert!(forge1.is_err());

        drop(forge0);
    }

    #[test]
    fn test_forge_generates_sunray() {
        {
            let mut created = ALREADY_CREATED.lock().unwrap();
            *created = false;
        }

        let forge = Forge::new().expect("Failed to create forge");
        let _sunray = forge.generate_sunray();
        drop(forge);
    }

    #[test]
    fn test_forge_generates_asteroid() {
        {
            let mut created = ALREADY_CREATED.lock().unwrap();
            *created = false;
        }

        let forge = Forge::new().expect("Failed to create forge");
        let _asteroid = forge.generate_asteroid();
        assert!(true, "generated asteroid");
        drop(forge);
    }

    #[test]
    fn test_forge_drop_resets_singleton() {
        {
            let mut created = ALREADY_CREATED.lock().unwrap();
            *created = false;
        }

        {
            let forge = Forge::new().expect("First forge failed");
            assert!(Forge::new().is_err());
            drop(forge);
        }

        let forge2 = Forge::new();
        assert!(forge2.is_ok());
    }
}
