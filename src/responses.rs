use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint64};
use serde_json;

use crate::{schema::EntitySchema, state::CollectionMetadata};

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

#[cw_serde]
pub struct InfoResponse {
    pub metadata: CollectionMetadata,
    pub schema: EntitySchema,
    pub operator: Addr,
    pub size: u32,
}
