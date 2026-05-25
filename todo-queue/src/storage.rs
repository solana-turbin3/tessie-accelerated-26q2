use std::fs;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::queue::Queue;


pub fn save_to_file<T>(queue: &Queue<T>, filename: &str)
where
    T: BorshSerialize,
{
    let bytes = borsh::to_vec(queue).unwrap();
    fs::write(filename, bytes).unwrap();
}


pub fn load_from_file<T>(filename: &str) -> Queue<T>
where
    T: BorshDeserialize,
{
    let bytes = fs::read(filename);

    match bytes {
        Ok(data) => borsh::from_slice(&data).unwrap(),
        Err(_) => Queue::new(),
    }
}

