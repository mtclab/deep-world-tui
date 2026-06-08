use crate::model::{Animal, Companion};
use crate::rng::SeedRng;

pub fn settlement_companions(rng: &mut SeedRng, capacity: usize) -> Vec<Companion> {
    let mut companions = Vec::new();
    for _ in 0..capacity {
        let roll = rng.gen_f64();
        let animal = if roll < 0.60 {
            Animal::Hound
        } else if roll < 0.90 {
            Animal::Donkey
        } else {
            Animal::Crow
        };
        let name = companion_name(rng, animal);
        companions.push(Companion::new(animal, name, 0));
    }
    companions
}

fn companion_name(rng: &mut SeedRng, animal: Animal) -> String {
    let hound_names = [
        "Ashen", "Bracken", "Cinder", "Dusk", "Ember", "Fern", "Gravel", "Haze",
    ];
    let donkey_names = [
        "Steady", "Boulder", "Moss", "Thorn", "Pebble", "Root", "Stone", "Drift",
    ];
    let crow_names = [
        "Murmur", "Shadow", "Whisper", "Wing", "Croak", "Fleet", "Quill", "Dusk",
    ];
    let names = match animal {
        Animal::Hound => &hound_names as &[&str],
        Animal::Donkey => &donkey_names as &[&str],
        Animal::Crow => &crow_names as &[&str],
        _ => &hound_names,
    };
    let idx = rng.gen_range(names.len() as u32) as usize;
    names[idx].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SeedRng;

    #[test]
    fn settlement_companions_respects_capacity() {
        let mut rng = SeedRng::new(42);
        let cs = settlement_companions(&mut rng, 2);
        assert_eq!(cs.len(), 2);
    }

    #[test]
    fn settlement_companions_distribution() {
        let mut rng = SeedRng::new(99);
        let mut hounds = 0usize;
        let mut donkeys = 0usize;
        let mut crows = 0usize;
        for _ in 0..300 {
            let cs = settlement_companions(&mut rng, 1);
            match cs[0].animal {
                Animal::Hound => hounds += 1,
                Animal::Donkey => donkeys += 1,
                Animal::Crow => crows += 1,
                _ => {}
            }
        }
        assert!(hounds > 100, "hounds={hounds}");
        assert!(donkeys > 30, "donkeys={donkeys}");
        assert!(crows > 10, "crows={crows}");
    }

    #[test]
    fn hamlet_has_no_companions() {
        let mut rng = SeedRng::new(42);
        let cs = settlement_companions(&mut rng, 0);
        assert!(cs.is_empty());
    }

    #[test]
    fn same_seed_same_companions() {
        let mut r1 = SeedRng::new(77);
        let mut r2 = SeedRng::new(77);
        let cs1 = settlement_companions(&mut r1, 3);
        let cs2 = settlement_companions(&mut r2, 3);
        assert_eq!(cs1.len(), cs2.len());
        for (a, b) in cs1.iter().zip(cs2.iter()) {
            assert_eq!(a.animal, b.animal);
            assert_eq!(a.name, b.name);
        }
    }
}
