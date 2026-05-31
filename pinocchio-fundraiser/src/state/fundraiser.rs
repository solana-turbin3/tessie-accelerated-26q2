use pinocchio::{AccountView, Address, error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fundraiser {
    maker: [u8; 32],
    mint_to_raise: [u8; 32],
    amount_to_raise: [u8; 8],
    current_amount: [u8; 8],
    time_started: [u8; 8],
    pub duration: u8,
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1;

    pub fn from_account_view(account: &mut AccountView) -> Result<&mut Self, ProgramError> {
        if account.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(unsafe { &mut *(account.borrow_unchecked_mut().as_mut_ptr() as *mut Self) })
    }

    pub fn maker(&self) -> &Address {
        unsafe { &*(&self.maker as *const [u8; 32] as *const Address) }
    }

    pub fn set_maker(&mut self, maker: &Address) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn mint_to_raise(&self) -> &Address {
        unsafe { &*(&self.mint_to_raise as *const [u8; 32] as *const Address) }
    }

    pub fn set_mint_to_raise(&mut self, mint_to_raise: &Address) {
        self.mint_to_raise.copy_from_slice(mint_to_raise.as_ref());
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount: u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn current_amount(&self) -> u64 {
        u64::from_le_bytes(self.current_amount)
    }

    pub fn set_current_amount(&mut self, amount: u64) {
        self.current_amount = amount.to_le_bytes();
    }

    pub fn add_current_amount(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.set_current_amount(
            self.current_amount()
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
        Ok(())
    }

    pub fn sub_current_amount(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.set_current_amount(
            self.current_amount()
                .checked_sub(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
        Ok(())
    }

    pub fn time_started(&self) -> i64 {
        i64::from_le_bytes(self.time_started)
    }

    pub fn set_time_started(&mut self, timestamp: i64) {
        self.time_started = timestamp.to_le_bytes();
    }
}
