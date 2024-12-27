use crate::error::ContractError;
use crate::execute::create::exec_create;
use crate::execute::delete::exec_delete;
use crate::execute::update::exec_update;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::info::query_info;
use crate::query::read::query_read;
use crate::state::{ExecuteContext, QueryContext};
use cosmwasm_std::{entry_point, to_json_binary, Env};
use cosmwasm_std::{Binary, Deps, DepsMut, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-entity";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let mut ctx = ExecuteContext::new(deps, env, info);
    ctx.instantiate(msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = ExecuteContext::new(deps, env, info);
    match msg {
        ExecuteMsg::Create(args) => exec_create(ctx, args),
        ExecuteMsg::Update(args) => exec_update(ctx, args),
        ExecuteMsg::Delete(args) => exec_delete(ctx, args),
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = QueryContext { deps, env };
    let result = match msg {
        QueryMsg::Info {} => to_json_binary(&query_info(ctx)?),
        QueryMsg::Read(args) => to_json_binary(&query_read(ctx, args)?),
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
