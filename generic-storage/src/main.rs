mod error;
mod models;
mod serializer;
mod storage;

pub use error::StorageError;
pub use models::Person;
pub use serializer::{Borsh, Json, Serializer, StorageData, Wincode};
pub use storage::Storage;

fn main() {
    let person = Person {
        name: "Andre".to_string(),
        age: 30,
    };

    let mut storage = Storage::new(Json);
    storage.save(&person).unwrap();

    let loaded: Person = storage.load().unwrap();
    println!("Loaded: {:?}", loaded);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_person() -> Person {
        Person {
            name: "Andre".to_string(),
            age: 30,
        }
    }

    #[test]
    fn save_and_load_with_borsh() {
        let person = test_person();

        let mut storage = Storage::new(Borsh);
        storage.save(&person).unwrap();

        assert!(storage.has_data());

        let loaded: Person = storage.load().unwrap();
        assert_eq!(loaded, person);
    }

    #[test]
    fn save_and_load_with_json() {
        let person = test_person();

        let mut storage = Storage::new(Json);
        storage.save(&person).unwrap();

        assert!(storage.has_data());

        let loaded: Person = storage.load().unwrap();
        assert_eq!(loaded, person);
    }

    #[test]
    fn save_and_load_with_wincode() {
        let person = test_person();

        let mut storage = Storage::new(Wincode);
        storage.save(&person).unwrap();

        assert!(storage.has_data());

        let loaded: Person = storage.load().unwrap();
        assert_eq!(loaded, person);
    }

    #[test]
    fn load_without_data_should_fail() {
        let storage: Storage<Person, Borsh> = Storage::new(Borsh);

        let result = storage.load();

        assert!(result.is_err());
    }
}
