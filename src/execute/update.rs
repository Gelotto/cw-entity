use crate::{error::ContractError, msg::UpdateArgs, state::ExecuteContext};
use cosmwasm_std::{attr, Response};

pub fn exec_update(
    ctx: ExecuteContext,
    args: UpdateArgs,
) -> Result<Response, ContractError> {
    let ExecuteContext { .. } = ctx;
    let UpdateArgs { .. } = args;
    Ok(Response::new().add_attributes(vec![attr("action", "update")]))
}
