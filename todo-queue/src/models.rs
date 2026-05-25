use borsh_derive::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Todo {
    pub id: u64,
    pub description: String,
    pub created_at: u64,
}
