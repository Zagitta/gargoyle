use chrono::{DateTime, Utc};
use serde::Deserialize;

pub const UNSET_INTEGER: i32 = std::i32::MAX;
pub const UNSET_DOUBLE: f64 = 1.7976931348623157E308_f64;
pub const UNSET_LONG: i64 = std::i64::MAX;

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

#[derive(Debug)]
pub struct TWSMessage {
    pub req_id: i32,
    pub timestamp: DateTime<Utc>,
    //pub data: MessageData,
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
    primary_exch: &'a str,
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
    strike: f64,
    right: &'a str,
    multiplier: &'a str,
    exchange: &'a str,
    currency: &'a str,
    local_symbol: &'a str,
    //primary_exch: &'a str,
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
pub enum TWSIncommingMessage<'a> {
    #[serde(rename = "1")]
    TickPrice {
        msg_version: i32,
        req_id: i32,
        tick_type: TickType,
        price: f64,
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
        avg_fill_price: f64,
        perm_id: i32,
        parent_id: i32,
        last_fill_price: f64,
        client_id: i32,
        why_held: &'a str,
        market_cap_price: f64,
    },
    #[serde(rename = "4")]
    Error {
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
    NextValidId {},
    #[serde(rename = "10")]
    ContractData {
        msg_version: i32,
        req_id: i32,
        symbol: &'a str,
        sec_type: &'a str,
        last_trade_date_or_contract_month: &'a str,
        strike: f64,
        right: &'a str,
        exchange: &'a str,
        currency: &'a str,
        local_symbol: &'a str,
        market_name: &'a str,
        trading_class: &'a str,
        con_id: i32,
        min_tick: f64,
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
    HistoricalData {},
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
    MarketDataType {},
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
        expirations: Vec<&'a str>,
        strikes: Vec<f64>,
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
    TickReqParams {},
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
    HistogramData {},
    #[serde(rename = "90")]
    HistoricalDataUpdate {},
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

impl TWSIncommingMessage<'_> {
    pub fn get_req_id(&self) -> Option<&i32> {
        match self {
            TWSIncommingMessage::TickPrice { req_id, .. } => Some(req_id),
            TWSIncommingMessage::TickSize { req_id, .. } => Some(req_id),
            TWSIncommingMessage::Error { req_id, .. } => Some(req_id),
            TWSIncommingMessage::TickOptionComputation { req_id, .. } => Some(req_id),
            TWSIncommingMessage::TickGeneric { req_id, .. } => Some(req_id),
            TWSIncommingMessage::TickString { req_id, .. } => Some(req_id),
            TWSIncommingMessage::TickEFP { req_id, .. } => Some(req_id),
            TWSIncommingMessage::SecurityDefinitionOptionParameter { req_id, .. } => Some(req_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::serde::Deserializer;
    use super::TWSIncommingMessage;
    use serde::Deserialize;

    #[test]
    fn can_deser_managed_account() {
        let msg = vec!["15".into(), "1".into(), "DU3113049".into()];
        let mut de = Deserializer::from_msg(&msg);

        assert_eq!(
            TWSIncommingMessage::ManagedAccounts {
                msg_version: 1,
                account_list: "DU3113049"
            },
            TWSIncommingMessage::deserialize(&mut de).unwrap()
        );
    }
}
