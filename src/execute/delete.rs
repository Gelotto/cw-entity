use crate::{error::ContractError, msg::DeleteArgs, state::ExecuteContext};
use cosmwasm_std::{attr, Response};

pub fn exec_delete(
    mut ctx: ExecuteContext,
    args: DeleteArgs,
) -> Result<Response, ContractError> {
    let ExecuteContext { .. } = ctx;
    let DeleteArgs { id } = args;

    ctx.require_operator()?;
    ctx.delete_entity(args)?;

    Ok(Response::new().add_attributes(vec![attr("action", "delete"), attr("id", id)]))
}
