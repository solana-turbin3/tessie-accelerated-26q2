use pinocchio::{AccountView, error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contributor {
    amount: [u8; 8],
    pub bump: u8,
}

impl Contributor {
    pub const LEN: usize = 8 + 1;

    pub fn from_account_view(account: &mut AccountView) -> Result<&mut Self, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { &mut *(account.borrow_unchecked_mut().as_mut_ptr() as *mut Self) })
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }

    pub fn add_amount(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.set_amount(
            self.amount()
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
        Ok(())
    }
}
