use anchor_lang::prelude::*;
use std::mem::size_of;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const MAX_SCORES: usize = 50; // Define the maximum number of scores

#[program]
pub mod tip_board {
    use super::*;

    // Initializes the tipboard
    pub fn initialize_tipboard(ctx: Context<InitializeTipboard>) -> Result<()> {
        let tipboard = &mut ctx.accounts.tipboard;
        tipboard.authority = *ctx.accounts.signer.key;
        tipboard.tips = Vec::new(); // Initialize the scores vector
        Ok(())
    }

    // Function to add a new tip to the tipboard
    pub fn add_score(ctx: Context<AddTipContext>, amount: u64, timestamp: i64, nft_mint: String) -> Result<()> {
        let tipboard = &mut ctx.accounts.tipboard;
        let tipper = ctx.accounts.signer.key();
        let new_tip = Tip { tipper, amount, timestamp, nft_mint };
        
        // CHECK: The signer is the player who's score is being added
        if ctx.accounts.signer.key() != new_tip.tipper {
            return Err(ErrorCode::WrongSigner.into());
        }

        // CHECK: The tipoard is not full
        if tipboard.tips.len() == MAX_SCORES {
            return Err(ErrorCode::TipboardFull.into());
        }

        // Find the position to insert the new tip
        let position = tipboard.tips.iter()
                            .position(|x| x.amount <= new_tip.amount)
                            .unwrap_or(tipboard.tips.len());
    
        // Insert the new score at the found position
        tipboard.tips.insert(position, new_tip);
    
        Ok(())
    }
    
    // Function to reset tipboard
    pub fn reset_tipboard(ctx: Context<ResetTipboardContext>) -> Result<()> {
        if ctx.accounts.signer.key() != ctx.accounts.tipboard.authority {
            return Err(ErrorCode::Unauthorized.into());
        }
        let tipboard = &mut ctx.accounts.tipboard;
        tipboard.tips = Vec::new(); // Reset the tip vector
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTipboard<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + (8 + size_of::<Tip>() * MAX_SCORES),
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
    /// CHECK: This is not dangerous because the signer is checked in the program
    #[account(signer)]
    pub signer: AccountInfo<'info>,
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




