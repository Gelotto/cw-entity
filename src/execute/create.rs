use crate::{error::ContractError, msg::CreateArgs, state::ExecuteContext};
use cosmwasm_std::{attr, Response};

pub fn exec_create(
    mut ctx: ExecuteContext,
    args: CreateArgs,
) -> Result<Response, ContractError> {
    let ExecuteContext { .. } = ctx;
    let id = args.id.clone();

    ctx.create_entity(args)?;

    Ok(Response::new().add_attributes(vec![attr("action", "create"), attr("id", id)]))
}
