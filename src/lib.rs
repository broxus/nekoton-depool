pub mod abi;
pub mod models;

use anyhow::Result;
use models::{DePoolInfo, ParticipantInfo};
use nekoton::{
    core::InternalMessage,
    transport::{
        models::{ExistingContract, RawContractState},
        Transport,
    },
};
use nekoton_abi::*;
use nekoton_utils::TrustMe;
use std::{convert::TryInto, sync::Arc};
use ton_block::MsgAddressInt;

pub async fn get_participant_info(
    transport: Arc<dyn Transport>,
    address: MsgAddressInt,
    wallet_address: MsgAddressInt,
) -> Result<ParticipantInfo> {
    let raw_state = transport.get_contract_state(&address).await?;
    let contract = match raw_state {
        RawContractState::NotExists => return Err(DePoolError::UnknownContract.into()),
        RawContractState::Exists(contract) => contract,
    };

    let state = DePoolContractState(&contract);
    let participant_info = state.get_participant_info(wallet_address)?;

    Ok(participant_info)
}

pub async fn get_depool_info(
    transport: Arc<dyn Transport>,
    address: MsgAddressInt,
) -> Result<DePoolInfo> {
    let raw_state = transport.get_contract_state(&address).await?;
    let contract = match raw_state {
        RawContractState::NotExists => return Err(DePoolError::UnknownContract.into()),
        RawContractState::Exists(contract) => contract,
    };

    let state = DePoolContractState(&contract);
    let depool_info = state.get_depool_info()?;

    Ok(depool_info)
}

pub struct DePoolContractState<'a>(pub &'a ExistingContract);

impl<'a> DePoolContractState<'a> {
    pub fn get_participant_info(&self, address: MsgAddressInt) -> Result<ParticipantInfo> {
        let function = abi::depool_v3().function("getParticipantInfo").trust_me();

        let mut inputs = Vec::<ton_abi::Token>::new();
        inputs.push(address.token_value().named("addr"));

        let data = self.0.run_local(function, &inputs)?.try_into()?;

        Ok(data)
    }

    pub fn get_depool_info(&self) -> Result<DePoolInfo> {
        let function = abi::depool_v3().function("getDePoolInfo").trust_me();

        let data = self.0.run_local(function, &[])?.try_into()?;

        Ok(data)
    }
}

pub fn prepare_add_ordinary_stake(
    depool: MsgAddressInt,
    depool_fee: u64,
    stake: u64,
) -> Result<InternalMessage> {
    let (function, input) = MessageBuilder::new(abi::depool_v3(), "addOrdinaryStake")
        .trust_me()
        .arg(stake)
        .build();

    let body = function
        .encode_input(&Default::default(), &input, true, None)?
        .into();

    Ok(InternalMessage {
        source: None,
        destination: depool,
        amount: stake + depool_fee,
        bounce: false,
        body,
    })
}

pub fn prepare_withdraw_part(
    depool: MsgAddressInt,
    depool_fee: u64,
    withdraw_value: u64,
) -> Result<InternalMessage> {
    let (function, input) = MessageBuilder::new(abi::depool_v3(), "withdrawPart")
        .trust_me()
        .arg(withdraw_value)
        .build();

    let body = function
        .encode_input(&Default::default(), &input, true, None)?
        .into();

    Ok(InternalMessage {
        source: None,
        destination: depool,
        amount: depool_fee,
        bounce: false,
        body,
    })
}

trait ExistingContractExt {
    fn run_local(
        &self,
        function: &ton_abi::Function,
        input: &[ton_abi::Token],
    ) -> Result<Vec<ton_abi::Token>>;
}

impl ExistingContractExt for ExistingContract {
    fn run_local(
        &self,
        function: &ton_abi::Function,
        input: &[ton_abi::Token],
    ) -> Result<Vec<ton_abi::Token>> {
        let output = function.run_local(
            self.account.clone(),
            self.timings,
            &self.last_transaction_id,
            input,
        )?;
        output
            .tokens
            .ok_or_else(|| DePoolError::NonZeroResultCode.into())
    }
}

#[derive(thiserror::Error, Debug)]
enum DePoolError {
    #[error("Unknown contract")]
    UnknownContract,
    #[error("Non zero result code")]
    NonZeroResultCode,
}
