use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

const COMP_DEF_OFFSET_INIT_MARKET_TOTALS: u32 = comp_def_offset("init_market_totals");
const COMP_DEF_OFFSET_TALLY_PREDICTION: u32 = comp_def_offset("tally_prediction");
const COMP_DEF_OFFSET_REVEAL_RESULT: u32 = comp_def_offset("reveal_result");

declare_id!("BETjBPwEVkArkWbFLVxnQ8jJUbRhxBm8SJMeaN8VuWJp");

#[arcium_program]
pub mod private_market {
    use super::*;

    pub fn init_market_totals_comp_def(ctx: Context<InitMarketTotalsCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        computation_offset: u64,
        question: String,
        closes_at: i64,
    ) -> Result<()> {
        require!(
            question.len() <= Market::MAX_QUESTION_BYTES,
            ErrorCode::QuestionTooLong
        );
        require!(
            closes_at > Clock::get()?.unix_timestamp,
            ErrorCode::InvalidCloseTime
        );

        let market = &mut ctx.accounts.market;
        market.bump = ctx.bumps.market;
        market.creator = ctx.accounts.payer.key();
        market.tally_state = [[0; 32]; 3];
        market.nonce = 0;
        market.question = question;
        market.closes_at = closes_at;
        market.submissions = 0;
        market.closed = false;
        market.settled = false;

        let args = ArgBuilder::new().build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![InitMarketTotalsCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.market.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "init_market_totals")]
    pub fn init_market_totals_callback(
        ctx: Context<InitMarketTotalsCallback>,
        output: SignedComputationOutputs<InitMarketTotalsOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(InitMarketTotalsOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.market.tally_state = o.ciphertexts;
        ctx.accounts.market.nonce = o.nonce;

        Ok(())
    }

    pub fn init_tally_prediction_comp_def(ctx: Context<InitTallyPredictionCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn submit_prediction(
        ctx: Context<SubmitPrediction>,
        computation_offset: u64,
        encrypted_side: [u8; 32],
        encrypted_stake_lamports: [u8; 32],
        pub_key: [u8; 32],
        prediction_nonce: u128,
    ) -> Result<()> {
        let market = &ctx.accounts.market;
        require!(!market.closed, ErrorCode::MarketClosed);
        require!(
            Clock::get()?.unix_timestamp < market.closes_at,
            ErrorCode::MarketClosed
        );

        let args = ArgBuilder::new()
            .x25519_pubkey(pub_key)
            .plaintext_u128(prediction_nonce)
            .encrypted_bool(encrypted_side)
            .encrypted_u64(encrypted_stake_lamports)
            .plaintext_u128(ctx.accounts.market.nonce)
            .account(
                ctx.accounts.market.key(),
                Market::TALLY_STATE_OFFSET,
                32 * 3,
            )
            .build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![TallyPredictionCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.market.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "tally_prediction")]
    pub fn tally_prediction_callback(
        ctx: Context<TallyPredictionCallback>,
        output: SignedComputationOutputs<TallyPredictionOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(TallyPredictionOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.market.tally_state = o.ciphertexts;
        ctx.accounts.market.nonce = o.nonce;
        ctx.accounts.market.submissions = ctx.accounts.market.submissions.saturating_add(1);

        emit!(PrivatePredictionAccepted {
            market: ctx.accounts.market.key(),
            submissions: ctx.accounts.market.submissions,
        });

        Ok(())
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.market.creator,
            ErrorCode::InvalidAuthority
        );
        require!(
            Clock::get()?.unix_timestamp >= ctx.accounts.market.closes_at,
            ErrorCode::MarketStillOpen
        );

        ctx.accounts.market.closed = true;
        Ok(())
    }

    pub fn init_reveal_result_comp_def(ctx: Context<InitRevealResultCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn reveal_result(ctx: Context<RevealResult>, computation_offset: u64) -> Result<()> {
        require!(ctx.accounts.market.closed, ErrorCode::MarketStillOpen);
        require!(
            !ctx.accounts.market.settled,
            ErrorCode::MarketAlreadySettled
        );

        let args = ArgBuilder::new()
            .plaintext_u128(ctx.accounts.market.nonce)
            .account(
                ctx.accounts.market.key(),
                Market::TALLY_STATE_OFFSET,
                32 * 3,
            )
            .build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![RevealResultCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.market.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "reveal_result")]
    pub fn reveal_result_callback(
        ctx: Context<RevealResultCallback>,
        output: SignedComputationOutputs<RevealResultOutput>,
    ) -> Result<()> {
        let yes_wins = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(RevealResultOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.market.settled = true;

        emit!(MarketSettled {
            market: ctx.accounts.market.key(),
            submissions: ctx.accounts.market.submissions,
            yes_wins,
        });

        Ok(())
    }
}

#[queue_computation_accounts("init_market_totals", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_MARKET_TOTALS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        init,
        payer = payer,
        space = Market::SPACE,
        seeds = [b"market", payer.key().as_ref(), computation_offset.to_le_bytes().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,
}

#[callback_accounts("init_market_totals")]
#[derive(Accounts)]
pub struct InitMarketTotalsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_MARKET_TOTALS))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: checked by Arcium callback constraints.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions sysvar.
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[init_computation_definition_accounts("init_market_totals", payer)]
#[derive(Accounts)]
pub struct InitMarketTotalsCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: initialized by the Arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: checked by the Arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Address Lookup Table program.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[queue_computation_accounts("tally_prediction", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct SubmitPrediction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_TALLY_PREDICTION))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("tally_prediction")]
#[derive(Accounts)]
pub struct TallyPredictionCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_TALLY_PREDICTION))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: checked by Arcium callback constraints.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions sysvar.
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[init_computation_definition_accounts("tally_prediction", payer)]
#[derive(Accounts)]
pub struct InitTallyPredictionCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: initialized by the Arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: checked by the Arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Address Lookup Table program.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[queue_computation_accounts("reveal_result", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct RevealResult<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by the Arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REVEAL_RESULT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("reveal_result")]
#[derive(Accounts)]
pub struct RevealResultCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_REVEAL_RESULT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: checked by Arcium callback constraints.
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions sysvar.
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[init_computation_definition_accounts("reveal_result", payer)]
#[derive(Accounts)]
pub struct InitRevealResultCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: initialized by the Arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: checked by the Arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: Address Lookup Table program.
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Market {
    pub bump: u8,
    pub creator: Pubkey,
    pub tally_state: [[u8; 32]; 3],
    pub nonce: u128,
    pub question: String,
    pub closes_at: i64,
    pub submissions: u64,
    pub closed: bool,
    pub settled: bool,
}

impl Market {
    pub const MAX_QUESTION_BYTES: usize = 160;
    pub const TALLY_STATE_OFFSET: usize = 8 + 1 + 32;
    pub const SPACE: usize =
        8 + 1 + 32 + (32 * 3) + 16 + 4 + Self::MAX_QUESTION_BYTES + 8 + 8 + 1 + 1;
}

#[event]
pub struct PrivatePredictionAccepted {
    pub market: Pubkey,
    pub submissions: u64,
}

#[event]
pub struct MarketSettled {
    pub market: Pubkey,
    pub submissions: u64,
    pub yes_wins: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Question is too long")]
    QuestionTooLong,
    #[msg("Close time must be in the future")]
    InvalidCloseTime,
    #[msg("Market is closed")]
    MarketClosed,
    #[msg("Market is still open")]
    MarketStillOpen,
    #[msg("Market has already been settled")]
    MarketAlreadySettled,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("The cluster is not set")]
    ClusterNotSet,
}
