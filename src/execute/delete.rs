use crate::{error::ContractError, msg::DeleteArgs, state::ExecuteContext};
use cosmwasm_std::{attr, Response};

pub fn exec_delete(
    ctx: ExecuteContext,
    args: DeleteArgs,
) -> Result<Response, ContractError> {
    let ExecuteContext { .. } = ctx;
    let DeleteArgs { .. } = args;
    Ok(Response::new().add_attributes(vec![attr("action", "delete")]))
}
