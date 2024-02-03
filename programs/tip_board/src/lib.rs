use anchor_lang::prelude::*;
use pyth_sdk_solana::*;
use std::mem::size_of;
declare_id!("7vZPMfghSw2rQWhvCs1XW6CDLunP36jB253bQVWWMUmu");

const MAX_TIPS: usize = 20; // Define the maximum number of tips
const MAX_TIPBOARDS: usize = 20; // Define the maximum number of scoreboards the program can hold
// This is the price feed id for SOL/USD on devnet
// https://pyth.network/developers/price-feed-ids#solana-devnet
const SOL_USD_PRICEFEED_ID : &str = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";
const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds

#[program]
pub mod tip_board {
    use super::*;
    use std::str::FromStr;
    use anchor_lang::solana_program::{system_instruction, native_token::LAMPORTS_PER_SOL, program::invoke};

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
        let tipper = ctx.accounts.signer.key();  

        // CHECK:
        if ctx.accounts.signer.key() != tipper {
            return Err(ErrorCode::WrongSigner.into());
        }

        if Pubkey::from_str(SOL_USD_PRICEFEED_ID) != Ok(ctx.accounts.sol_usd_price_account.key()){
            return Err(error!(ErrorCode::WrongPriceFeedId))
        };

        // CHECK: The tipoard is not full
        if ctx.accounts.tipboard.tips.len() == MAX_TIPS {
            // delete the last tip
            ctx.accounts.tipboard.tips.pop();
        }

        let sol_usd_price_feed = load_price_feed_from_account_info(&ctx.accounts.sol_usd_price_account).unwrap();
        let current_timestamp = Clock::get()?.unix_timestamp;
        let current_price: Price = sol_usd_price_feed.get_price_no_older_than(current_timestamp, STALENESS_THRESHOLD).unwrap();
        let amount_in_lamports = amount *  LAMPORTS_PER_SOL * 10u64.pow(u32::try_from(-current_price.expo).unwrap()) / (u64::try_from(current_price.price).unwrap());

        let transfer_instruction = system_instruction::transfer(&ctx.accounts.signer.key(), &ctx.accounts.to.key(), amount_in_lamports);
        invoke(&transfer_instruction, &ctx.accounts.to_account_infos())?;

        let new_tip = Tip {
            tipper: tipper,
            amount: amount_in_lamports,
            timestamp: timestamp,
            nft_mint: nft_mint,
        };

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
    /// CHECK : We will manually check this against the Pubkey of the price feed
    pub sol_usd_price_account : AccountInfo<'info>,
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

    #[msg("Price feed id is not correct.")]
    WrongPriceFeedId,
}




