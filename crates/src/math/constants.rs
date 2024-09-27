// PRECISIONS
pub const AMM_RESERVE_PRECISION: u128 = 1_000_000_000; //expo = -9;
pub const AMM_RESERVE_PRECISION_I128: i128 = (AMM_RESERVE_PRECISION) as i128;
pub const BASE_PRECISION: u128 = AMM_RESERVE_PRECISION; //expo = -9;
pub const BASE_PRECISION_I128: i128 = AMM_RESERVE_PRECISION_I128;
pub const BASE_PRECISION_U64: u64 = AMM_RESERVE_PRECISION as u64; //expo = -9;
pub const BASE_PRECISION_I64: i64 = AMM_RESERVE_PRECISION_I128 as i64; //expo = -9;
pub const PERP_DECIMALS: u32 = 9;

pub const PRICE_PRECISION: u128 = 1_000_000; //expo = -6;
pub const PRICE_PRECISION_I128: i128 = PRICE_PRECISION as i128;
pub const PRICE_PRECISION_U64: u64 = 1_000_000; //expo = -6;
pub const PRICE_PRECISION_I64: i64 = 1_000_000; //expo = -6;

pub const PEG_PRECISION: u128 = 1_000_000; //expo = -6
pub const PEG_PRECISION_I128: i128 = PEG_PRECISION as i128; //expo = -6

pub const QUOTE_PRECISION: u128 = 1_000_000; // expo = -6
pub const QUOTE_PRECISION_I128: i128 = 1_000_000; // expo = -6
pub const QUOTE_PRECISION_I64: i64 = 1_000_000; // expo = -6
pub const QUOTE_PRECISION_U64: u64 = 1_000_000; // expo = -6

pub const FUNDING_RATE_BUFFER: u128 = 1_000; // expo = -3
pub const FUNDING_RATE_BUFFER_I128: i128 = FUNDING_RATE_BUFFER as i128; // expo = -3

pub const MARGIN_PRECISION: u32 = 10_000; // expo = -4
pub const MARGIN_PRECISION_U128: u128 = 10_000; // expo = -4
pub const SPOT_WEIGHT_PRECISION: u32 = MARGIN_PRECISION; // expo = -4
pub const SPOT_WEIGHT_PRECISION_U128: u128 = SPOT_WEIGHT_PRECISION as u128; // expo = -4
pub const SPOT_WEIGHT_PRECISION_I128: i128 = SPOT_WEIGHT_PRECISION as i128; // expo = -4

pub const LIQUIDATION_PCT_PRECISION: u128 = 10_000;

pub const SPOT_BALANCE_PRECISION: u128 = 1_000_000_000; // expo = -9
pub const SPOT_BALANCE_PRECISION_U64: u64 = 1_000_000_000; // expo = -9
pub const SPOT_CUMULATIVE_INTEREST_PRECISION: u128 = 10_000_000_000; // expo = -10

pub const PERCENTAGE_PRECISION: u128 = 1_000_000; // expo -6 (represents 100%)
pub const PERCENTAGE_PRECISION_I128: i128 = PERCENTAGE_PRECISION as i128;
pub const PERCENTAGE_PRECISION_U64: u64 = PERCENTAGE_PRECISION as u64;
pub const PERCENTAGE_PRECISION_I64: i64 = PERCENTAGE_PRECISION as i64;
pub const TEN_BPS: i128 = PERCENTAGE_PRECISION_I128 / 1000;
pub const TEN_BPS_I64: i64 = TEN_BPS as i64;
pub const TWO_PT_TWO_PCT: i128 = 22_000;

pub const BID_ASK_SPREAD_PRECISION: u64 = PERCENTAGE_PRECISION as u64; // expo = -6
pub const BID_ASK_SPREAD_PRECISION_I64: i64 = (BID_ASK_SPREAD_PRECISION) as i64;
pub const BID_ASK_SPREAD_PRECISION_U128: u128 = BID_ASK_SPREAD_PRECISION as u128; // expo = -6
pub const BID_ASK_SPREAD_PRECISION_I128: i128 = BID_ASK_SPREAD_PRECISION as i128; // expo = -6

pub const CONCENTRATION_PRECISION: u128 = PERCENTAGE_PRECISION; // expo 6
pub const IF_FACTOR_PRECISION: u128 = PERCENTAGE_PRECISION; // expo 6

pub const SPOT_UTILIZATION_PRECISION: u128 = PERCENTAGE_PRECISION; // expo = -6
pub const SPOT_UTILIZATION_PRECISION_U32: u32 = PERCENTAGE_PRECISION as u32; // expo = -6
pub const SPOT_RATE_PRECISION: u128 = PERCENTAGE_PRECISION; // expo = -6
pub const SPOT_RATE_PRECISION_U32: u32 = PERCENTAGE_PRECISION as u32; // expo = -6
pub const LIQUIDATION_FEE_PRECISION: u32 = PERCENTAGE_PRECISION as u32; // expo = -6
pub const LIQUIDATION_FEE_PRECISION_U128: u128 = LIQUIDATION_FEE_PRECISION as u128; // expo = -6
pub const SPOT_IMF_PRECISION: u32 = PERCENTAGE_PRECISION as u32; // expo = -6

pub const LAMPORTS_PER_SOL_I64: i64 = 1_000_000_000;