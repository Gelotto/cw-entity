use crate::{error::ContractError, msg::UpdateArgs, state::ExecuteContext};
use cosmwasm_std::{attr, Response};

pub fn exec_update(
    mut ctx: ExecuteContext,
    args: UpdateArgs,
) -> Result<Response, ContractError> {
    let ExecuteContext { .. } = ctx;
    let UpdateArgs { id, .. } = args;

    ctx.require_operator()?;
    ctx.update_entity(args)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update"), attr("id", id)]))
}
