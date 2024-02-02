use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_instruction, program::invoke};
use std::mem::size_of;
declare_id!("7vZPMfghSw2rQWhvCs1XW6CDLunP36jB253bQVWWMUmu");

const MAX_TIPS: usize = 20; // Define the maximum number of tips
const MAX_TIPBOARDS: usize = 20; // Define the maximum number of scoreboards the program can hold
// cost 2.9531191 to deploy

#[program]
pub mod tip_board {
    use super::*;

    // Initializes the tipboard
    pub fn initialize_tipboard(ctx: Context<InitializeTipboard>) -> Result<()> {
        let tipboard_account = &mut ctx.accounts.tipboard_account;
        let tipboard = &mut ctx.accounts.tipboard;
        let signer = &ctx.accounts.signer.key();

        tipboard_account.tipboards.push(signer.clone());

        tipboard.authority = signer.clone(); // Set the authority to the signer

        tipboard.tips = Vec::new(); // Initialize the tip vector

        Ok(())
    }

    // Function to add a new tip to the tipboard
    pub fn add_tip(ctx: Context<AddTipContext>, amount: u64, timestamp: i64, nft_mint: String) -> Result<()> {
        // let tipboard = &mut ctx.accounts.tipboard;
        let tipper = ctx.accounts.signer.key();
        let new_tip = Tip { tipper, amount, timestamp, nft_mint };        

        // CHECK: The signer is the player who's score is being added
        if ctx.accounts.signer.key() != new_tip.tipper {
            return Err(ErrorCode::WrongSigner.into());
        }

        // CHECK: The tipoard is not full
        if ctx.accounts.tipboard.tips.len() == MAX_TIPS {
            // delete the last tip
            ctx.accounts.tipboard.tips.pop();
        }

        let transfer_instruction = system_instruction::transfer(&ctx.accounts.signer.key(), &ctx.accounts.to.key(), amount);
        invoke(&transfer_instruction, &ctx.accounts.to_account_infos())?;

        // Find the position to insert the new tip
        let position = ctx.accounts.tipboard.tips.iter()
                            .position(|x| x.amount <= new_tip.amount)
                            .unwrap_or(ctx.accounts.tipboard.tips.len());
    
        // Insert the new score at the found position
        ctx.accounts.tipboard.tips.insert(position, new_tip);
    
        Ok(())
    }   
}

#[derive(Accounts)]
pub struct InitializeTipboard<'info> {
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 32 + (8 + size_of::<Tip>() * MAX_TIPBOARDS),
        seeds = [b"tipboard"],
        bump
    )]
    pub tipboard_account: Account<'info, TipboardAccount>,
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + (8 + size_of::<Tip>() * MAX_TIPS),
        seeds = [b"tipboard", signer.key().as_ref()],
        bump
    )]
    pub tipboard: Account<'info, Tipboard>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddTipContext<'info> {
    #[account(mut)]
    pub tipboard: Account<'info, Tipboard>,
    /// CHECK:
    #[account(mut)]
    pub to: AccountInfo<'info>,
    /// CHECK: This is not dangerous because the signer is checked in the program
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResetTipboardContext<'info> {
    #[account(mut)]
    pub tipboard: Account<'info, Tipboard>,
    /// CHECK: This is not dangerous because the signer is checked in the program
    #[account(signer)]
    pub signer: AccountInfo<'info>,
}

#[account]
pub struct TipboardAccount {
    pub tipboards: Vec<Pubkey>,
}

#[account]
pub struct Tipboard {
    pub authority: Pubkey,
    pub tips: Vec<Tip>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Tip {
    pub tipper: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub nft_mint: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,

    #[msg("Signer does not match player.")]
    WrongSigner,

    #[msg("Tipboard is full.")]
    TipboardFull,
}




