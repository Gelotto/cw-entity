use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, Timestamp, Uint64};
use cw_storage_plus::{Item, Map};
use serde_json;

use crate::{
    error::ContractError,
    msg::{CreateArgs, DeleteArgs, InstantiateMsg, UpdateArgs},
    schema::{EntityProperty, EntitySchema},
};

pub type ObjectId = u64;
pub type PropertyIndex<'a> = Map<'a, (&'a [u8], ObjectId), u8>;

pub const OPERATOR: Item<Addr> = Item::new("op");
pub const METADATA: Item<CollectionMetadata> = Item::new("meta");
pub const SCHEMA: Item<EntitySchema> = Item::new("schema");
pub const CREATED_AT: Map<ObjectId, Timestamp> = Map::new("tc");
pub const UPDATED_AT: Map<ObjectId, Timestamp> = Map::new("tu");
pub const ENTITY: Map<ObjectId, serde_json::Value> = Map::new("entities");
pub const COUNT: Item<u32> = Item::new("n");

#[cw_serde]
pub struct CollectionMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
}

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
        let InstantiateMsg {
            operator,
            schema,
            metadata,
        } = msg;

        COUNT.save(self.deps.storage, &0)?;
        SCHEMA.save(self.deps.storage, &schema)?;
        METADATA.save(
            self.deps.storage,
            &metadata.unwrap_or_else(|| CollectionMetadata {
                name: None,
                description: None,
                website: None,
            }),
        )?;
        OPERATOR.save(
            self.deps.storage,
            &self
                .deps
                .api
                .addr_validate(operator.unwrap_or(self.info.sender.clone()).as_str())?,
        )?;

        Ok(Response::new().add_attribute("action", "instantiate"))
    }

    pub fn set_collection_metadata(
        &mut self,
        metadata: &CollectionMetadata,
    ) -> Result<(), ContractError> {
        Ok(METADATA.save(self.deps.storage, metadata)?)
    }

    pub fn set_operator(
        &mut self,
        new_operator: &Addr,
    ) -> Result<(), ContractError> {
        OPERATOR.save(
            self.deps.storage,
            &self
                .deps
                .api
                .addr_validate(new_operator.as_str())
                .map_err(|_| ContractError::ValidationError {
                    reason: "invalid new operator address".to_owned(),
                })?,
        )?;
        Ok(())
    }

    pub fn require_operator(&self) -> Result<(), ContractError> {
        if self.info.sender != OPERATOR.load(self.deps.storage)? {
            return Err(ContractError::NotAuthorized {
                reason: "operator required".to_owned(),
            });
        }
        Ok(())
    }

    pub fn create_entity(
        &mut self,
        args: CreateArgs,
    ) -> Result<(), ContractError> {
        let CreateArgs { id, data } = args;
        let id = id.u64();
        if ENTITY.has(self.deps.storage, id) {
            return Err(ContractError::NotAuthorized {
                reason: format!("entity {} already exists", id),
            });
        }
        ENTITY.save(self.deps.storage, id, &data)?;
        CREATED_AT.save(self.deps.storage, id, &self.env.block.time)?;
        COUNT.update(self.deps.storage, |x| -> Result<_, ContractError> {
            x.checked_add(1).ok_or_else(|| ContractError::Unexpected {
                reason: "collection max size reached".to_owned(),
            })
        })?;
        self.update_indices(
            &id.into(),
            &serde_json::Value::Object(serde_json::Map::new()),
            &data,
            &self.load_schema()?,
        )?;
        Ok(())
    }

    pub fn update_entity(
        &mut self,
        args: UpdateArgs,
    ) -> Result<(), ContractError> {
        let UpdateArgs { id, data: new_data } = args;
        if let Ok(old_data) = ENTITY.load(self.deps.storage, id.u64()) {
            let schema = self.load_schema()?;
            self.update_indices(&id, &old_data, &new_data, &schema)?;
            UPDATED_AT.save(self.deps.storage, id.u64(), &self.env.block.time)?;
            Ok(())
        } else {
            Err(ContractError::NotFound {
                reason: format!("entity {} not found", id.u64()),
            })
        }
    }

    pub fn delete_entity(
        &mut self,
        args: DeleteArgs,
    ) -> Result<(), ContractError> {
        let DeleteArgs { id } = args;
        if let Ok(data) = ENTITY.load(self.deps.storage, id.u64()) {
            let schema = self.load_schema()?;
            self.remove_entity_from_indices(id.u64(), &schema, &data)?;
            ENTITY.remove(self.deps.storage, id.u64());
            UPDATED_AT.remove(self.deps.storage, id.u64());
            CREATED_AT.remove(self.deps.storage, id.u64());
            COUNT.update(self.deps.storage, |x| -> Result<_, ContractError> {
                x.checked_sub(1).ok_or_else(|| ContractError::Unexpected {
                    reason: "collection count already zero".to_owned(),
                })
            })?;
            Ok(())
        } else {
            Err(ContractError::NotFound {
                reason: format!("entity {} not found", id.u64()),
            })
        }
    }

    pub fn load_schema(&self) -> Result<EntitySchema, ContractError> {
        Ok(SCHEMA.load(self.deps.storage)?)
    }

    fn remove_entity_from_indices(
        &mut self,
        id: ObjectId,
        schema: &EntitySchema,
        data: &serde_json::Value,
    ) -> Result<(), ContractError> {
        let values = data.as_object().ok_or_else(|| ContractError::Unexpected {
            reason: "entity data not an object".to_owned(),
        })?;
        // Remove all props from index if any
        for prop in schema.properties.iter() {
            let EntityProperty { name, .. } = prop;
            let index_name = format!("_ix_{}", name);
            if let Some(value) = values.get(name) {
                // Get or create index
                let index = PropertyIndex::new(&index_name);
                // Remove from index
                index.remove(self.deps.storage, (&prop.to_bytes(value)?, id));
            }
        }
        Ok(())
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
