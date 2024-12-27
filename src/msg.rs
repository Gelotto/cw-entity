use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint64};
use serde_json;

#[allow(unused_imports)]
use crate::{responses::ReadResponse, schema::EntitySchema};

#[cw_serde]
pub struct InstantiateMsg {
    pub operator: Option<Addr>,
    pub schema: EntitySchema,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    Create(CreateArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[cw_serde]
pub struct CreateArgs {
    pub id: Uint64,
    pub data: serde_json::Value,
}

#[cw_serde]
pub enum IndexBound {
    Inclusive(serde_json::Value),
    Exclusive(serde_json::Value),
}

#[cw_serde]
pub enum ReadTarget {
    Ids(Vec<Uint64>),
    Index {
        property: String,
        cursor: Option<Binary>,
        start: Option<IndexBound>,
        stop: Option<IndexBound>,
        limit: Option<u8>,
    },
}

#[cw_serde]
pub struct ReadArgs {
    pub target: ReadTarget,
    pub desc: Option<bool>,
    pub select: Option<Vec<String>>,
}

#[cw_serde]
pub struct UpdateArgs {
    pub id: String,
}

#[cw_serde]
pub struct DeleteArgs {
    pub id: String,
}

#[cw_serde]
#[derive(cw_orch::QueryFns, QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<ReadResponse>)]
    Read(ReadArgs),
}

#[cw_serde]
pub struct MigrateMsg {}
