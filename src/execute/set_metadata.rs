use crate::{
    error::ContractError,
    state::{CollectionMetadata, ExecuteContext},
};
use cosmwasm_std::{attr, Response};

pub fn exec_set_metadata(
    mut ctx: ExecuteContext,
    new_metadata: CollectionMetadata,
) -> Result<Response, ContractError> {
    ctx.require_operator()?;
    ctx.set_collection_metadata(&new_metadata)?;
    Ok(Response::new().add_attributes(vec![attr("action", "set_metadata")]))
}
