use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint64};
use serde_json;

#[cw_serde]
pub struct Entity {
    pub id: Uint64,
    pub data: Option<serde_json::Value>,
}

#[cw_serde]
pub struct ReadResponse {
    pub entities: Vec<Entity>,
    pub cursor: Option<Binary>,
}
