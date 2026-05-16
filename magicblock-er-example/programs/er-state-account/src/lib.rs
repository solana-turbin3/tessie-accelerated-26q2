#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod instructions;
mod state;

use instructions::*;

declare_id!("8XFwahSZpcquR1KVJMyGDvEk35B4THLrKz48qqhZWTmv");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;

        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>, new_data: u64) -> Result<()> {
        ctx.accounts.update(new_data)?;

        Ok(())
    }

    pub fn request_vrf_update(ctx: Context<RequestVrfUpdate>, client_seed: u8) -> Result<()> {
        ctx.accounts.request_vrf_update(client_seed)?;

        Ok(())
    }

    pub fn callback_vrf_update(
        ctx: Context<CallbackVrfUpdate>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.callback_vrf_update(randomness)?;

        Ok(())
    }

    pub fn request_vrf_update_er(ctx: Context<RequestVrfUpdateEr>, client_seed: u8) -> Result<()> {
        ctx.accounts.request_vrf_update_er(client_seed)?;

        Ok(())
    }

    pub fn callback_vrf_update_er(
        ctx: Context<CallbackVrfUpdateEr>,
        randomness: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.callback_vrf_update_er(randomness)?;

        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)?;

        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;

        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;

        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;

        Ok(())
    }
}
