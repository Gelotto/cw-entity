use std::marker::PhantomData;

use cosmwasm_std::{Order, Storage, Uint64};
use cw_storage_plus::Bound;
use serde_json;

use crate::{
    error::ContractError,
    msg::{IndexBound, ReadArgs, ReadTarget},
    responses::{Entity, ReadResponse},
    state::{PropertyIndex, QueryContext, ENTITY, SCHEMA},
};

pub const MAX_PAGE_SIZE: u8 = 50;

pub fn query_read(
    ctx: QueryContext,
    args: ReadArgs,
) -> Result<ReadResponse, ContractError> {
    let QueryContext { deps, .. } = ctx;
    read(deps.storage, args)
}

pub fn read(
    storage: &dyn Storage,
    args: ReadArgs,
) -> Result<ReadResponse, ContractError> {
    let ReadArgs {
        target,
        limit,
        desc,
        select,
    } = args;
    let desc = desc.unwrap_or_default();
    let limit = limit.unwrap_or(10).min(MAX_PAGE_SIZE) as usize;
    let order = if desc { Order::Descending } else { Order::Ascending };

    let mut ids: Vec<Uint64> = Vec::with_capacity(limit);
    let mut next_cursor: Option<(Vec<u8>, u64)> = None;

    match target {
        ReadTarget::Ids(mut target_ids) => {
            if desc {
                target_ids.reverse();
            }
            ids = target_ids
        },
        ReadTarget::Index {
            property: prop_name,
            cursor,
            start,
            stop,
        } => {
            let schema = SCHEMA.load(storage)?;
            let index_name = format!("_ix_{}", prop_name);
            let index = PropertyIndex::new(index_name.as_str());
            let prop = schema
                .properties
                .iter()
                .find(|p| p.name == prop_name)
                .ok_or_else(|| ContractError::ValidationError { reason: format!("") })?;

            let mut tmp: Box<Vec<u8>> = Box::new(vec![]);
            let mut tmp_start: Box<Vec<u8>> = Box::new(vec![]);
            let mut tmp_stop: Box<Vec<u8>> = Box::new(vec![]);

            let mut min = cursor
                .and_then(|(bytes_vec, id)| {
                    *tmp = bytes_vec;
                    Some(Bound::Inclusive((((*tmp).as_slice(), id.u64()), PhantomData)))
                })
                .or_else(|| {
                    start.and_then(|b| {
                        Some(match b {
                            IndexBound::Inclusive(v) => {
                                *tmp_start = prop.to_bytes(&v).unwrap();
                                Bound::Inclusive((((*tmp_start).as_slice(), u64::MIN), PhantomData))
                            },
                            IndexBound::Exclusive(v) => {
                                *tmp_start = prop.to_bytes(&v).unwrap();
                                Bound::Exclusive((((*tmp_start).as_slice(), u64::MIN), PhantomData))
                            },
                        })
                    })
                });

            let mut max = stop.and_then(|b| {
                Some(match b {
                    IndexBound::Inclusive(v) => {
                        *tmp_stop = prop.to_bytes(&v).unwrap();
                        Bound::Inclusive((((*tmp_stop).as_slice(), u64::MAX), PhantomData))
                    },
                    IndexBound::Exclusive(v) => {
                        *tmp_stop = prop.to_bytes(&v).unwrap();
                        Bound::Exclusive((((*tmp_stop).as_slice(), u64::MAX), PhantomData))
                    },
                })
            });

            if desc {
                let tmp = min;
                min = max;
                max = tmp;
            }

            for result in index.keys(storage, min, max, order).take(limit) {
                let (bytes, id) = result?;
                next_cursor = Some((bytes, id));
                ids.push(id.into());
            }
        },
    };

    // Now build vec of IDs and selected entity data, if any
    let mut entities: Vec<Entity> = Vec::with_capacity(ids.len());

    if let Some(selected_prop_names) = select {
        let select_star = selected_prop_names.iter().find(|k| *k == "*").is_some();
        for id in ids {
            // Select all fields or only specific ones
            let entity_value = ENTITY.load(storage, id.u64())?;
            if select_star {
                entities.push(Entity {
                    id,
                    data: Some(entity_value),
                })
            } else {
                let value_map = entity_value.as_object().unwrap();
                let mut filtered_data = serde_json::Map::new();
                for k in selected_prop_names.iter() {
                    if let Some(v) = value_map.get(k) {
                        filtered_data.insert(k.to_owned(), v.to_owned());
                    }
                }
                entities.push(Entity {
                    id,
                    data: Some(serde_json::Value::Object(filtered_data)),
                })
            }
        }
    } else {
        for id in ids {
            entities.push(Entity { id, data: None });
        }
    }

    // Return results and the next cursor
    Ok(ReadResponse {
        cursor: next_cursor.and_then(|(vec, id)| Some((vec, Uint64::from(id)))),
        entities,
    })
}
