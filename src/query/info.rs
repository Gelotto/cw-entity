use crate::{
    error::ContractError,
    responses::InfoResponse,
    state::{QueryContext, COUNT, METADATA, OPERATOR, SCHEMA},
};

pub fn query_info(ctx: QueryContext) -> Result<InfoResponse, ContractError> {
    let QueryContext { deps, .. } = ctx;
    Ok(InfoResponse {
        metadata: METADATA.load(deps.storage)?,
        operator: OPERATOR.load(deps.storage)?,
        schema: SCHEMA.load(deps.storage)?,
        size: COUNT.load(deps.storage)?,
    })
}
