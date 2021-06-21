use crate::tws::serde::custom_chrono;
use std::collections::HashSet;

use super::{codec::DecodedMessage, serde::de::Deserializer, serde::error::Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twsapi::core::contract;

type Currency = fixed::FixedI64<fixed::types::extra::U20>;

bitflags! {
    #[derive(Deserialize)]
    pub struct TickAttribute : i32 {
        const CAN_AUTO_EXE = 0b001;
        const PAST_LIMIT = 0b010;
        const PRE_OPEN = 0b100;
    }
}

#[repr(i32)]
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketDataType {
    #[serde(rename = "1")]
    Realtime = 1,
    #[serde(rename = "2")]
    Frozen = 2,
    #[serde(rename = "3")]
    Delayed = 3,
    #[serde(rename = "4")]
    DelayedFrozen = 4,
}

#[repr(i32)]
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickType {
    #[serde(rename = "0")]
    BidSize,
    #[serde(rename = "1")]
    Bid,
    #[serde(rename = "2")]
    Ask,
    #[serde(rename = "3")]
    AskSize,
    #[serde(rename = "4")]
    Last,
    #[serde(rename = "5")]
    LastSize,
    #[serde(rename = "6")]
    High,
    #[serde(rename = "7")]
    Low,
    #[serde(rename = "8")]
    Volume,
    #[serde(rename = "9")]
    Close,
    #[serde(rename = "10")]
    BidOptionComputation,
    #[serde(rename = "11")]
    AskOptionComputation,
    #[serde(rename = "12")]
    LastOptionComputation,
    #[serde(rename = "13")]
    ModelOption,
    #[serde(rename = "14")]
    Open,
    #[serde(rename = "15")]
    Low13Week,
    #[serde(rename = "16")]
    High13Week,
    #[serde(rename = "17")]
    Low26Week,
    #[serde(rename = "18")]
    High26Week,
    #[serde(rename = "19")]
    Low52Week,
    #[serde(rename = "20")]
    High52Week,
    #[serde(rename = "21")]
    AvgVolume,
    #[serde(rename = "22")]
    OpenInterest,
    #[serde(rename = "23")]
    OptionHistoricalVol,
    #[serde(rename = "24")]
    OptionImpliedVol,
    #[serde(rename = "25")]
    OptionBidExch,
    #[serde(rename = "26")]
    OptionAskExch,
    #[serde(rename = "27")]
    OptionCallOpenInterest,
    #[serde(rename = "28")]
    OptionPutOpenInterest,
    #[serde(rename = "29")]
    OptionCallVolume,
    #[serde(rename = "30")]
    OptionPutVolume,
    #[serde(rename = "31")]
    IndexFuturePremium,
    #[serde(rename = "32")]
    BidExch,
    #[serde(rename = "33")]
    AskExch,
    #[serde(rename = "34")]
    AuctionVolume,
    #[serde(rename = "35")]
    AuctionPrice,
    #[serde(rename = "36")]
    AuctionImbalance,
    #[serde(rename = "37")]
    MarkPrice,
    #[serde(rename = "38")]
    BidEfpComputation,
    #[serde(rename = "39")]
    AskEfpComputation,
    #[serde(rename = "40")]
    LastEfpComputation,
    #[serde(rename = "41")]
    OpenEfpComputation,
    #[serde(rename = "42")]
    HighEfpComputation,
    #[serde(rename = "43")]
    LowEfpComputation,
    #[serde(rename = "44")]
    CloseEfpComputation,
    #[serde(rename = "45")]
    LastTimestamp,
    #[serde(rename = "46")]
    Shortable,
    #[serde(rename = "47")]
    FundamentalRatios,
    #[serde(rename = "48")]
    RtVolume,
    #[serde(rename = "49")]
    Halted,
    #[serde(rename = "50")]
    BidYield,
    #[serde(rename = "51")]
    AskYield,
    #[serde(rename = "52")]
    LastYield,
    #[serde(rename = "53")]
    CustOptionComputation,
    #[serde(rename = "54")]
    TradeCount,
    #[serde(rename = "55")]
    TradeRate,
    #[serde(rename = "56")]
    VolumeRate,
    #[serde(rename = "57")]
    LastRthTrade,
    #[serde(rename = "58")]
    RtHistoricalVol,
    #[serde(rename = "59")]
    IbDividends,
    #[serde(rename = "60")]
    BondFactorMultiplier,
    #[serde(rename = "61")]
    RegulatoryImbalance,
    #[serde(rename = "62")]
    NewsTick,
    #[serde(rename = "63")]
    ShortTermVolume3Min,
    #[serde(rename = "64")]
    ShortTermVolume5Min,
    #[serde(rename = "65")]
    ShortTermVolume10Min,
    #[serde(rename = "66")]
    DelayedBid,
    #[serde(rename = "67")]
    DelayedAsk,
    #[serde(rename = "68")]
    DelayedLast,
    #[serde(rename = "69")]
    DelayedBidSize,
    #[serde(rename = "70")]
    DelayedAskSize,
    #[serde(rename = "71")]
    DelayedLastSize,
    #[serde(rename = "72")]
    DelayedHigh,
    #[serde(rename = "73")]
    DelayedLow,
    #[serde(rename = "74")]
    DelayedVolume,
    #[serde(rename = "75")]
    DelayedClose,
    #[serde(rename = "76")]
    DelayedOpen,
    #[serde(rename = "77")]
    RtTrdVolume,
    #[serde(rename = "78")]
    CreditmanMarkPrice,
    #[serde(rename = "79")]
    CreditmanSlowMarkPrice,
    #[serde(rename = "80")]
    DelayedBidOption,
    #[serde(rename = "81")]
    DelayedAskOption,
    #[serde(rename = "82")]
    DelayedLastOption,
    #[serde(rename = "83")]
    DelayedModelOption,
    #[serde(rename = "84")]
    LastExch,
    #[serde(rename = "85")]
    LastRegTime,
    #[serde(rename = "86")]
    FuturesOpenInterest,
    #[serde(rename = "87")]
    AvgOptVolume,
    #[serde(rename = "88")]
    DelayedLastTimestamp,
    #[serde(rename = "89")]
    ShortableShares,
    #[serde(rename = "UNSET_INTEGER")]
    NotSet,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Contract<'a> {
    symbol: &'a str,
    sec_type: &'a str,
    last_trade_date_or_contract_month: &'a str,
    strike: f64,
    right: &'a str,
    multiplier: &'a str,
    exchange: &'a str,
    currency: &'a str,
    local_symbol: &'a str,
    primary_exchange: &'a str,
    trading_class: &'a str,
    include_expired: bool,
    sec_id_type: &'a str,
    sec_id: &'a str,
    combo_legs_description: &'a str,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct OrderContract<'a> {
    con_id: i32,
    symbol: &'a str,
    sec_type: &'a str,
    last_trade_date_or_contract_month: &'a str,
    strike: Currency,
    right: &'a str,
    multiplier: &'a str,
    exchange: &'a str,
    currency: &'a str,
    local_symbol: &'a str,
    //primary_exchange: &'a str,
    trading_class: &'a str,
    //include_expired: bool,
    //sec_id_type: &'a str,
    //sec_id: &'a str,
    //combo_legs_description: &'a str,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum InnerOrderStatus {
    Submitted,
    Cancelled,
}
#[derive(Deserialize, PartialEq, Debug)]
pub struct HistoricalBarData {
    #[serde(with = "custom_chrono")]
    date: DateTime<Utc>,
    open: Currency,
    high: Currency,
    low: Currency,
    close: Currency,
    volume: Option<i64>,
    wap: Option<Currency>,
    trade_count: Option<i32>, //only valid for TRADES req
}

mod historical_bar_data_update {
    use super::{Currency, HistoricalBarData};
    use crate::tws::serde::custom_chrono;
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[derive(Deserialize, PartialEq, Debug)]
    struct HistoricalBarDataUpdate {
        trade_count: i32,
        #[serde(with = "custom_chrono")]
        date: DateTime<Utc>,
        open: Currency,
        close: Currency,
        high: Currency,
        low: Currency,
        wap: Option<Currency>,
        volume: Option<i64>,
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<HistoricalBarData, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = HistoricalBarDataUpdate::deserialize(deserializer)?;

        Ok(HistoricalBarData {
            date: data.date,
            open: data.open,
            high: data.high,
            low: data.low,
            close: data.close,
            volume: data.volume,
            wap: data.wap,
            trade_count: Some(data.trade_count),
        })
    }
}

#[allow(dead_code)]
#[derive(Deserialize, PartialEq, Debug)]
pub enum TWSIncommingMessageImpl<'a> {
    #[serde(rename = "1")]
    TickPrice {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        price: Currency,
        size: i32,
        attrib: TickAttribute,
    },
    #[serde(rename = "2")]
    TickSize {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        size: i32,
    },
    #[serde(rename = "3")]
    OrderStatus {
        id: i32,
        status: &'a str,
        filled_amount: f64,
        remaining_amount: f64,
        avg_fill_price: Currency,
        perm_id: i32,
        parent_id: i32,
        last_fill_price: Currency,
        client_id: i32,
        why_held: &'a str,
        market_cap_price: Currency,
    },
    #[serde(rename = "4")]
    Error {
        msg_version: i32,
        req_id: i32,
        code: i32,
        msg: &'a str,
    },
    #[serde(rename = "5")]
    OpenOrder {
        order_id: i32,
        contract: OrderContract<'a>,
        //order: Order,
        //order_state: OrderState,
    },
    #[serde(rename = "6")]
    AccountValue {
        msg_version: i32,
        key: &'a str,
        value: &'a str,
        currency: &'a str,
        account_name: &'a str,
    },
    #[serde(rename = "7")]
    PortfolioValue {},
    #[serde(rename = "8")]
    AccountUpdateTime {},
    #[serde(rename = "9")]
    NextValidId { msg_version: i32, order_id: i32 },
    #[serde(rename = "10")]
    ContractData {
        msg_version: i32,
        req_id: i32,
        symbol: &'a str,
        sec_type: &'a str,
        last_trade_date_or_contract_month: &'a str,
        strike: Currency,
        right: &'a str,
        exchange: &'a str,
        currency: &'a str,
        local_symbol: &'a str,
        market_name: &'a str,
        trading_class: &'a str,
        con_id: i32,
        min_tick: Currency,
        md_size_multiplier: i32,
        multiplier: &'a str,
        order_types: &'a str,
        valid_exchanges: &'a str,
        price_magnifier: i32,
        under_con_id: i32,
        long_name: &'a str,
        primary_exchange: &'a str,
        contract_month: &'a str,
        industry: &'a str,
        category: &'a str,
        subcategory: &'a str,
        time_zone_id: &'a str,
        trading_hours: &'a str,
        liquid_hours: &'a str,
        ev_rule: &'a str,
        ev_multiplier: &'a str,
        sec_id_list: Vec<(&'a str, &'a str)>,
        agg_group: i32,
        under_symbol: &'a str,
        under_sec_type: &'a str,
        market_rule_ids: &'a str,
        real_expiration_date: &'a str,
    },
    #[serde(rename = "11")]
    ExecutionData {},
    #[serde(rename = "12")]
    MarketDepth {},
    #[serde(rename = "13")]
    MarketDepthL2 {},
    #[serde(rename = "14")]
    NewsBulletins {},
    #[serde(rename = "15")]
    ManagedAccounts {
        msg_version: i32,
        account_list: &'a str,
    },
    #[serde(rename = "16")]
    ReceiveFA {},
    #[serde(rename = "17")]
    HistoricalData {
        req_id: i32,
        start_date: &'a str,
        end_date: &'a str,
        bars: Vec<HistoricalBarData>,
    },
    #[serde(rename = "18")]
    BondContractData {},
    #[serde(rename = "19")]
    ScannerParameters {},
    #[serde(rename = "20")]
    ScannerData {},
    #[serde(rename = "21")]
    TickOptionComputation {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        implied_vol: f64,
        delta: f64,
        price: f64,
        present_value_dividend: f64,
        gamma: f64,
        vega: f64,
        theta: f64,
        underlying_price: f64,
    },
    #[serde(rename = "45")]
    TickGeneric {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        value: f64,
    },
    #[serde(rename = "46")]
    TickString {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        value: &'a str,
    },
    #[serde(rename = "47")]
    TickEFP {
        req_id: i32,
        tick_type: TickType,
        basis_points: f64,
        formatted_basis_points: &'a str,
        implied_future: f64,
        hold_days: i32,
        future_last_trade_date: &'a str,
        dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    },
    #[serde(rename = "49")]
    CurrentTime {},
    #[serde(rename = "50")]
    RealTimeBars {},
    #[serde(rename = "51")]
    FundamentalData {},
    #[serde(rename = "52")]
    ContractDataEnd,
    #[serde(rename = "53")]
    OpenOrderEnd,
    #[serde(rename = "54")]
    AccountDownloadEnd,
    #[serde(rename = "55")]
    ExecutionDataEnd,
    #[serde(rename = "56")]
    DeltaNeutralValidation {},
    #[serde(rename = "57")]
    TickSnapshotEnd,
    #[serde(rename = "58")]
    MarketDataType {
        msg_version: i32,
        req_id: i32,
        data_type: MarketDataType,
    },
    #[serde(rename = "59")]
    CommissionsReport {
        msg_version: i32,
        exec_id: &'a str,
        commission: f64,
        currency: &'a str,
        realized_pnl: f64,
        yield_: f64,
        yield_redemption_date: i32,
    },
    #[serde(rename = "61")]
    Position {},
    #[serde(rename = "62")]
    PositionEnd,
    #[serde(rename = "63")]
    AccountSummary {},
    #[serde(rename = "64")]
    AccountSummaryEnd,
    #[serde(rename = "65")]
    VerifyMessageApi {},
    #[serde(rename = "66")]
    VerifyCompleted {},
    #[serde(rename = "67")]
    DisplayGroupList {},
    #[serde(rename = "68")]
    DisplayGroupUpdated {},
    #[serde(rename = "69")]
    VerifyAndAuthMessageApi {},
    #[serde(rename = "70")]
    VerifyAndAuthCompleted {},
    #[serde(rename = "71")]
    PositionMulti {},
    #[serde(rename = "72")]
    PositionMultiEnd,
    #[serde(rename = "73")]
    AccountUpdateMulti {},
    #[serde(rename = "74")]
    AccountUpdateMultiEnd,
    #[serde(rename = "75")]
    SecurityDefinitionOptionParameter {
        req_id: i32,
        exchange: &'a str,
        underlying_con_id: i32,
        trading_class: &'a str,
        multiplier: &'a str,
        expirations: HashSet<&'a str>,
        strikes: HashSet<&'a str>,
    },
    #[serde(rename = "76")]
    SecurityDefinitionOptionParameterEnd { req_id: i32 },
    #[serde(rename = "77")]
    SoftDollarTier {},
    #[serde(rename = "78")]
    FamilyCodes {},
    #[serde(rename = "79")]
    SymbolSamples {},
    #[serde(rename = "80")]
    MktDepthExchanges {},
    #[serde(rename = "81")]
    TickReqParams {
        req_id: i32,
        min_tick: f64,
        bbo_exchange: &'a str,
        snapshot_permssion: bool,
    },
    #[serde(rename = "82")]
    SmartComponents {},
    #[serde(rename = "83")]
    NewsArticle {},
    #[serde(rename = "84")]
    TickNews {},
    #[serde(rename = "85")]
    NewsProviders {},
    #[serde(rename = "86")]
    HistoricalNews {},
    #[serde(rename = "87")]
    HistoricalNewsEnd,
    #[serde(rename = "88")]
    HeadTimestamp {},
    #[serde(rename = "89")]
    HistogramData { req_id: i32, data: Vec<(f64, i64)> },
    #[serde(rename = "90")]
    HistoricalDataUpdate {
        req_id: i32,
        #[serde(with = "historical_bar_data_update")]
        bar: HistoricalBarData,
    },
    #[serde(rename = "91")]
    RerouteMktDataReq {},
    #[serde(rename = "92")]
    RerouteMktDepthReq {},
    #[serde(rename = "93")]
    MarketRule {},
    #[serde(rename = "94")]
    PnL {},
    #[serde(rename = "95")]
    PnLSingle {},
    #[serde(rename = "96")]
    HistoricalTick {},
    #[serde(rename = "97")]
    HistoricalTickBidAsk {},
    #[serde(rename = "98")]
    HistoricalTickLast {},
    #[serde(rename = "99")]
    TickByTick {},
    #[serde(rename = "100")]
    OrderBound {},
    #[serde(rename = "101")]
    CompletedOrder {},
    #[serde(rename = "102")]
    CompletedOrdersEnd,
}

impl TWSIncommingMessageImpl<'_> {
    pub fn get_req_id(&self) -> Option<i32> {
        match self {
            TWSIncommingMessageImpl::TickPrice { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickSize { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::Error { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickOptionComputation { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickGeneric { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickString { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::MarketDataType { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickReqParams { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::TickEFP { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::SecurityDefinitionOptionParameter { req_id, .. } => {
                Some(*req_id)
            }
            TWSIncommingMessageImpl::SecurityDefinitionOptionParameterEnd { req_id } => {
                Some(*req_id)
            }
            TWSIncommingMessageImpl::HistoricalData { req_id, .. } => Some(*req_id),
            TWSIncommingMessageImpl::HistoricalDataUpdate { req_id, .. } => Some(*req_id),

            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct TWSIncommingMessage {
    data: DecodedMessage,
    msg: TWSIncommingMessageImpl<'static>,
}

impl TWSIncommingMessage {
    pub fn get_msg<'a>(&'a self) -> &'a TWSIncommingMessageImpl<'a> {
        &self.msg
    }

    pub fn from_decoded_message(data: DecodedMessage) -> Result<TWSIncommingMessage> {
        let mut de = Deserializer::from_msg(&data);
        let parsed = TWSIncommingMessageImpl::deserialize(&mut de)?;
        Result::Ok(TWSIncommingMessage {
            data: data.clone(),
            msg: unsafe {
                //extend the lifetime to static but only hand out 'a references to it with get_msg()
                std::mem::transmute::<TWSIncommingMessageImpl<'_>, TWSIncommingMessageImpl<'static>>(
                    parsed,
                )
            },
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(dead_code)]
pub enum HistoricalDataType {
    Trades,
    Midpoint,
    Bid,
    Ask,
    BidAsk,
    HistoricalVolatility,
    OptionImpliedVolatility,
    FeeRate,
    RebateRate,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub enum HistoricalBarSize {
    OneSecond,
    FiveSeconds,
    TenSeconds,
    FifteenSeconds,
    ThirtySeconds,
    OneMinute,
    TwoMinutes,
    ThreeMinutes,
    FiveMinutes,
    TenMinutes,
    FifteenMinutes,
}

#[allow(non_snake_case)]
pub mod BarSize {
    pub trait ValidBarSize {
        const NAME: &'static str;
    }

    macro_rules! impl_valid_bar_size {
        ($type_name:ident, $str_name:literal, $N:literal) => {
            impl ValidBarSize for $type_name<$N> {
                const NAME: &'static str =  if $N == 1 { concat!($N, " ", $str_name) } else { concat!($N, " ", $str_name, "s") };
            }
        };
        ($type_name:ident, $str_name:literal, $N:literal $(, $NS:literal)*) => {
            impl_valid_bar_size!($type_name, $str_name, $N);
            impl_valid_bar_size!($type_name, $str_name $(, $NS)*);
        };
    }

    #[derive(Debug)]
    pub struct Seconds<const N: u8>;
    #[derive(Debug)]
    pub struct Minutes<const N: u8>;
    #[derive(Debug)]
    pub struct Hours<const N: u8>;
    #[derive(Debug)]
    pub struct Day<const N: u8>;
    #[derive(Debug)]
    pub struct Week<const N: u8>;
    #[derive(Debug)]
    pub struct Month<const N: u8>;

    impl_valid_bar_size!(Seconds, "sec", 1, 5, 10, 15, 30);
    impl_valid_bar_size!(Minutes, "min", 1, 2, 3, 5, 10, 15, 20, 30);
    impl_valid_bar_size!(Hours, "hour", 1, 2, 3, 4, 8);
    impl_valid_bar_size!(Day, "day", 1);
    impl_valid_bar_size!(Week, "week", 1);
    impl_valid_bar_size!(Month, "month", 1);
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub enum TWSOutgoingMessage {
    #[serde(rename = "1")]
    RequestMarketData {
        version: i32,
        req_id: i32,
        contract: contract::Contract,
    },
    #[serde(rename = "2")]
    CancelMarketData { version: i32, req_id: i32 },
    #[serde(rename = "3")]
    PlaceOrder {},
    #[serde(rename = "4")]
    CancelOrder {},
    #[serde(rename = "5")]
    RequestOpenOrders {},
    #[serde(rename = "6")]
    RequestAccountData {},
    #[serde(rename = "7")]
    RequestExecutions {},
    #[serde(rename = "8")]
    RequestIds {},
    #[serde(rename = "9")]
    RequestContractData {
        version: i32,
        req_id: i32,
        con_id: i32,
        symbol: String,
        sec_type: String,
        last_trade_date_or_contract_month: String,
        strike: f64,
        right: String,
        multiplier: String,
        exchange: String,
        primary_exchange: String,
        currency: String,
        local_symbol: String,
        trading_class: String,
        include_expired: bool,
        sec_id_type: String,
        sec_id: String,
        //combo_legs_description: &'a str,
    },
    #[serde(rename = "10")]
    RequestMarketDepth {},
    #[serde(rename = "11")]
    CancelMarketDepth {},
    #[serde(rename = "12")]
    RequestNewsBulletins {},
    #[serde(rename = "13")]
    CancelNewsBulletin {},
    #[serde(rename = "14")]
    ChangeServerLog {},
    #[serde(rename = "15")]
    RequestAutoOpenOrders {},
    #[serde(rename = "16")]
    RequestAllOpenOrders {},
    #[serde(rename = "17")]
    RequestManagedAccounts {},
    #[serde(rename = "18")]
    RequestFA {},
    #[serde(rename = "19")]
    ReplaceFA {},
    #[serde(rename = "20")]
    RequestHistoricalData {
        req_id: i32,
        con_id: i32,
        symbol: String,
        sec_type: String,
        last_trade_date_or_contract_month: String,
        strike: f64,
        right: String,
        multiplier: String,
        exchange: String,
        primary_exchange: String,
        currency: String,
        local_symbol: String,
        trading_class: String,
        include_expired: bool,

        end_date_time: String,
        bar_size: String,
        duration: String,
        use_regular_trading_hours: bool,
        what_to_show: HistoricalDataType,
        format_date: i8,

        //todo: add support for combo legs
        keep_up_to_date: bool,
        chart_options: (), //Vec<(String, String)>, //not really described
    },
    #[serde(rename = "21")]
    ExerciseOptions {},
    #[serde(rename = "22")]
    RequestScannerSubscription {},
    #[serde(rename = "23")]
    CancelScannerSubscription {},
    #[serde(rename = "24")]
    RequestScannerParameters {},
    #[serde(rename = "25")]
    CancelHistoricalData {},
    #[serde(rename = "49")]
    RequestCurrentTime {},
    #[serde(rename = "50")]
    RequestRealTimeBars {},
    #[serde(rename = "51")]
    CancelRealTimeBars {},
    #[serde(rename = "52")]
    RequestFundamentalData {},
    #[serde(rename = "53")]
    CancelFundamentalData {},
    #[serde(rename = "54")]
    ReqCalcImpliedVolat {},
    #[serde(rename = "55")]
    ReqCalcOptionPrice {},
    #[serde(rename = "56")]
    CancelImpliedVolatility {},
    #[serde(rename = "57")]
    CancelOptionPrice {},
    #[serde(rename = "58")]
    RequestGlobalCancel {},
    #[serde(rename = "59")]
    RequestMarketDataType {},
    #[serde(rename = "61")]
    RequestPositions {},
    #[serde(rename = "62")]
    RequestAccountSummary {},
    #[serde(rename = "63")]
    CancelAccountSummary {},
    #[serde(rename = "64")]
    CancelPositions {},
    #[serde(rename = "65")]
    VerifyRequest {},
    #[serde(rename = "66")]
    VerifyMessage {},
    #[serde(rename = "67")]
    QueryDisplayGroups {},
    #[serde(rename = "68")]
    SubscribeToGroupEvents {},
    #[serde(rename = "69")]
    UpdateDisplayGroup {},
    #[serde(rename = "70")]
    UnsubscribeFromGroupEvents {},
    #[serde(rename = "71")]
    StartApi {},
    #[serde(rename = "72")]
    VerifyAndAuthRequest {},
    #[serde(rename = "73")]
    VerifyAndAuthMessage {},
    #[serde(rename = "74")]
    RequestPositionsMulti {},
    #[serde(rename = "75")]
    CancelPositionsMulti {},
    #[serde(rename = "76")]
    RequestAccountUpdatesMulti {},
    #[serde(rename = "77")]
    CancelAccountUpdatesMulti {},
    #[serde(rename = "78")]
    RequestSecurityDefinitionOptionalParameters {
        req_id: i32,
        underlying: String,
        exchange: String,
        underlying_sec_type: String,
        underlying_con_id: i32,
    },
    #[serde(rename = "79")]
    RequestSoftDollarTiers {},
    #[serde(rename = "80")]
    RequestFamilyCodes {},
    #[serde(rename = "81")]
    RequestMatchingSymbols {},
    #[serde(rename = "82")]
    RequestMktDepthExchanges {},
    #[serde(rename = "83")]
    RequestSmartComponents {},
    #[serde(rename = "84")]
    RequestNewsArticle {},
    #[serde(rename = "85")]
    RequestNewsProviders {},
    #[serde(rename = "86")]
    RequestHistoricalNews {},
    #[serde(rename = "87")]
    RequestHeadTimestamp {},
    #[serde(rename = "88")]
    RequestHistogramData {
        req_id: i32,
        con_id: i32,
        symbol: String,
        sec_type: String,
        last_trade_date_or_contract_month: String,
        strike: f64,
        right: String,
        multiplier: String,
        exchange: String,
        primary_exchange: String,
        currency: String,
        local_symbol: String,
        trading_class: String,
        include_expired: bool,
        use_regular_trading_hours: bool,
        period: String,
    },
    #[serde(rename = "89")]
    CancelHistogramData {},
    #[serde(rename = "90")]
    CancelHeadTimestamp {},
    #[serde(rename = "91")]
    RequestMarketRule {},
    #[serde(rename = "92")]
    ReqPnL {},
    #[serde(rename = "93")]
    CancelPnL {},
    #[serde(rename = "94")]
    ReqPnLSingle {},
    #[serde(rename = "95")]
    CancelPnLSingle {},
    #[serde(rename = "96")]
    ReqHistoricalTicks {},
    #[serde(rename = "97")]
    ReqTickByTickData {},
    #[serde(rename = "98")]
    CancelTickByTickData {},
    #[serde(rename = "99")]
    ReqCompletedOrders {},
}

#[cfg(test)]
mod tests {
    use super::super::serde::de::Deserializer;
    use super::{HistoricalBarData, TWSIncommingMessageImpl};
    use bytes::Bytes;
    use chrono::{self, TimeZone, Utc};
    use serde::Deserialize;

    use fixed_macro::types::I44F20 as dec;

    #[test]
    fn can_deser_managed_account() {
        let msg = vec!["15".into(), "1".into(), "DU3113049".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            TWSIncommingMessageImpl::ManagedAccounts {
                msg_version: 1,
                account_list: "DU3113049"
            },
            TWSIncommingMessageImpl::deserialize(&mut de).unwrap()
        );
    }

    #[test]
    fn can_deser_historical_dat_update() {
        let msg = vec![
            "90".into(),
            "0".into(),
            "-1".into(),
            "20210419  15:56:15".into(),
            "10.30".into(),
            "10.20".into(),
            "10.35".into(),
            "10.00".into(),
            "-1.0".into(),
            "-1".into(),
        ];
        let mut de = Deserializer::from_msg(&msg);

        let ts = chrono::Local
            .datetime_from_str("20210419  15:56:15", "%Y%m%d  %H:%M:%S")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(
            Ok(TWSIncommingMessageImpl::HistoricalDataUpdate {
                req_id: 0,
                bar: HistoricalBarData {
                    date: ts,
                    open: dec!(10.30),
                    high: dec!(10.35),
                    low: dec!(10.0),
                    close: dec!(10.20),
                    wap: Some(dec!(-1.0)),
                    volume: Some(-1),
                    trade_count: Some(-1)
                }
            }),
            TWSIncommingMessageImpl::deserialize(&mut de)
        );
    }
    //

    #[test]
    fn can_transmute() {
        struct Wrapper {
            #[allow(dead_code)]
            data: Vec<Bytes>,
            msg: TWSIncommingMessageImpl<'static>,
        }

        impl Wrapper {
            pub fn get_msg<'a>(&'a self) -> &'a TWSIncommingMessageImpl<'a> {
                &self.msg
            }
        }

        let bytes = vec!["15".into(), "1".into(), "DU3113049".into()];
        let parsed = {
            let mut de = Deserializer::from_msg(&bytes);
            TWSIncommingMessageImpl::deserialize(&mut de).unwrap()
        };

        let res = Wrapper {
            data: bytes.clone(),
            msg: unsafe {
                std::mem::transmute::<TWSIncommingMessageImpl<'_>, TWSIncommingMessageImpl<'static>>(
                    parsed,
                )
            },
        };

        assert_eq!(
            &TWSIncommingMessageImpl::ManagedAccounts {
                msg_version: 1,
                account_list: "DU3113049"
            },
            res.get_msg()
        );
    }
}
