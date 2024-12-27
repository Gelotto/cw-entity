use crate::{
    error::ContractError,
    state::{CollectionMetadata, ExecuteContext},
};
use cosmwasm_std::{attr, Addr, Response};

pub fn exec_set_operator(
    mut ctx: ExecuteContext,
    new_operator: Addr,
) -> Result<Response, ContractError> {
    ctx.require_operator()?;
    ctx.set_operator(&new_operator)?;
    Ok(Response::new().add_attributes(vec![attr("action", "set_operator")]))
}
