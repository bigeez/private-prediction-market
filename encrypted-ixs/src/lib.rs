use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct PredictionInput {
        pub yes: bool,
        pub stake_lamports: u64,
    }

    pub struct MarketTotals {
        pub yes_stake_lamports: u64,
        pub no_stake_lamports: u64,
        pub submissions: u64,
    }

    pub struct SettlementSummary {
        pub yes_stake_lamports: u64,
        pub no_stake_lamports: u64,
        pub submissions: u64,
        pub yes_wins: bool,
    }

    #[instruction]
    pub fn tally_prediction(
        prediction_ctxt: Enc<Shared, PredictionInput>,
        totals_ctxt: Enc<Mxe, MarketTotals>,
    ) -> Enc<Mxe, MarketTotals> {
        let prediction = prediction_ctxt.to_arcis();
        let totals = totals_ctxt.to_arcis();

        let yes_stake_lamports =
            totals.yes_stake_lamports + (prediction.stake_lamports * prediction.yes as u64);
        let no_stake_lamports =
            totals.no_stake_lamports + (prediction.stake_lamports * (!prediction.yes) as u64);

        totals_ctxt.owner.from_arcis(MarketTotals {
            yes_stake_lamports,
            no_stake_lamports,
            submissions: totals.submissions + 1,
        })
    }

    #[instruction]
    pub fn init_market_totals() -> Enc<Mxe, MarketTotals> {
        Mxe::get().from_arcis(MarketTotals {
            yes_stake_lamports: 0,
            no_stake_lamports: 0,
            submissions: 0,
        })
    }

    #[instruction]
    pub fn reveal_result(totals_ctxt: Enc<Mxe, MarketTotals>) -> bool {
        let totals = totals_ctxt.to_arcis();
        let yes_wins = totals.yes_stake_lamports >= totals.no_stake_lamports;

        yes_wins.reveal()
    }
}
