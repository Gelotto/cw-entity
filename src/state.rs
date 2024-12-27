use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, Uint64};
use cw_storage_plus::{Item, Map};
use serde_json;

use crate::{
    error::ContractError,
    msg::{CreateArgs, InstantiateMsg},
    schema::{EntityProperty, EntitySchema},
};

pub type ObjectId = u64;
pub type PropertyIndex<'a> = Map<'a, (&'a [u8], ObjectId), u8>;

pub const CONFIG: Item<Config> = Item::new("config");
pub const OPERATOR: Item<Addr> = Item::new("operator");
pub const SCHEMA: Item<EntitySchema> = Item::new("schema");
pub const ENTITY: Map<ObjectId, serde_json::Value> = Map::new("entity");

#[cw_serde]
pub struct Config {}

pub struct ExecuteContext<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}

pub struct QueryContext<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl<'a> ExecuteContext<'a> {
    pub fn new(
        deps: DepsMut<'a>,
        env: Env,
        info: MessageInfo,
    ) -> Self {
        Self { deps, env, info }
    }
    /// Top-level initialization of contract state
    pub fn instantiate(
        &mut self,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let InstantiateMsg { operator, schema } = msg;

        OPERATOR.save(
            self.deps.storage,
            &self
                .deps
                .api
                .addr_validate(operator.unwrap_or(self.info.sender.clone()).as_str())?,
        )?;

        SCHEMA.save(self.deps.storage, &schema)?;

        Ok(Response::new().add_attribute("action", "instantiate"))
    }
    pub fn create_entity(
        &mut self,
        args: CreateArgs,
    ) -> Result<(), ContractError> {
        let CreateArgs { id, data: entity } = args;
        if ENTITY.has(self.deps.storage, id.u64()) {
            return Err(ContractError::NotAuthorized {
                reason: format!("entity {} already exists", id),
            });
        }
        ENTITY.save(self.deps.storage, id.u64(), &entity)?;
        Ok(())
    }

    pub fn load_schema(&self) -> Result<EntitySchema, ContractError> {
        Ok(SCHEMA.load(self.deps.storage)?)
    }

    pub fn update_indices(
        &mut self,
        id: &Uint64,
        old_entity: &serde_json::Value,
        new_entity: &serde_json::Value,
        schema: &EntitySchema,
    ) -> Result<(), ContractError> {
        let old_entity_value_map = old_entity.as_object().ok_or_else(|| ContractError::ValidationError {
            reason: "existing entity is not an object".to_owned(),
        })?;

        let entity_value_map = new_entity.as_object().ok_or_else(|| ContractError::ValidationError {
            reason: "updated entity is not an object".to_owned(),
        })?;

        for prop in schema.properties.iter() {
            let EntityProperty {
                name,
                required,
                indexed,
                ..
            } = prop;
            let index_name = format!("_ix_{}", name);
            if let Some(value) = entity_value_map.get(name) {
                // Get or create index
                if indexed.unwrap_or(false) {
                    let index = PropertyIndex::new(&index_name);
                    // Remove old node in index
                    if let Some(old_value) = old_entity_value_map.get(name) {
                        if *old_value == *value {
                            continue; // Skip updating
                        }
                        index.remove(self.deps.storage, (&prop.to_bytes(old_value)?, id.u64()));
                    }
                    // Set new node in index
                    let bytes = prop.to_bytes(value)?;
                    index.save(self.deps.storage, (&bytes, id.u64()), &0)?;
                }
            } else if required.unwrap_or(false) {
                return Err(ContractError::ValidationError {
                    reason: format!("{} required", name),
                });
            }
        }
        Ok(())
    }
}
