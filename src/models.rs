use anyhow::Result;
use nekoton_abi::*;
use nekoton_utils::serde_address;
use nekoton_utils::serde_vec_address;
use serde::Serialize;
use std::{collections::BTreeMap, convert::TryFrom};
use ton_abi::{ParamType, TokenValue, Uint};

#[derive(UnpackAbiPlain)]
pub struct GetParticipantInfoOutput {
    #[abi(uint64)]
    pub total: u64,
    #[abi(uint64, name = "withdrawValue")]
    pub withdraw_value: u64,
    #[abi(bool)]
    pub reinvest: bool,
    #[abi(uint64)]
    pub reward: u64,
    #[abi(unpack_with = "stakes_unpacker")]
    pub stakes: BTreeMap<u64, u64>,
    #[abi(with = "map_integer_tuple")]
    pub vestings: BTreeMap<u64, StakesTuple>,
    #[abi(with = "map_integer_tuple")]
    pub locks: BTreeMap<u64, StakesTuple>,
    #[abi(address, name = "vestingDonor")]
    pub vesting_donor: ton_block::MsgAddressInt,
    #[abi(address, name = "lockDonor")]
    pub lock_donor: ton_block::MsgAddressInt,
}

#[derive(UnpackAbi)]
pub struct StakesTuple {
    #[abi(uint64, name = "remainingAmount")]
    pub remaining_amount: u64,
    #[abi(uint64, name = "lastWithdrawalTime")]
    pub last_withdrawal_time: u64,
    #[abi(uint32, name = "withdrawalPeriod")]
    pub withdrawal_period: u32,
    #[abi(uint64, name = "withdrawalValue")]
    pub withdrawal_value: u64,
    #[abi(address)]
    pub owner: ton_block::MsgAddressInt,
}

fn stakes_unpacker(value: &TokenValue) -> UnpackerResult<BTreeMap<u64, u64>> {
    match value {
        TokenValue::Map(ParamType::Uint(64), ParamType::Uint(64), values) => {
            let mut map = BTreeMap::<u64, u64>::new();
            for (key, value) in values {
                let key = key.parse::<u64>().map_err(|_| UnpackerError::InvalidAbi)?;
                let value = match value {
                    TokenValue::Uint(Uint {
                        number: value,
                        size: 64,
                    }) => {
                        num_traits::ToPrimitive::to_u64(value).ok_or(UnpackerError::InvalidAbi)?
                    }
                    _ => return Err(UnpackerError::InvalidAbi),
                };
                map.insert(key, value);
            }
            Ok(map)
        }
        _ => Err(UnpackerError::InvalidAbi),
    }
}

#[derive(Serialize)]
pub struct ParticipantInfo {
    pub total: u64,
    pub withdraw_value: u64,
    pub reinvest: bool,
    pub reward: u64,
    pub stakes: BTreeMap<u64, u64>,
    pub vestings: BTreeMap<u64, ParticipantStake>,
    pub locks: BTreeMap<u64, ParticipantStake>,
    #[serde(with = "serde_address")]
    pub vesting_donor: ton_block::MsgAddressInt,
    #[serde(with = "serde_address")]
    pub lock_donor: ton_block::MsgAddressInt,
}

#[derive(Serialize)]
pub struct ParticipantStake {
    pub remaining_amount: u64,
    pub last_withdrawal_time: u64,
    pub withdrawal_period: u32,
    pub withdrawal_value: u64,
    #[serde(with = "serde_address")]
    pub owner: ton_block::MsgAddressInt,
}

impl TryFrom<Vec<ton_abi::Token>> for ParticipantInfo {
    type Error = UnpackerError;

    fn try_from(tokens: Vec<ton_abi::Token>) -> Result<Self, Self::Error> {
        let data: GetParticipantInfoOutput = tokens.unpack()?;

        let mut vestings = BTreeMap::<u64, ParticipantStake>::new();
        for (key, value) in data.vestings.into_iter() {
            let value = ParticipantStake {
                remaining_amount: value.remaining_amount,
                last_withdrawal_time: value.last_withdrawal_time,
                withdrawal_period: value.withdrawal_period,
                withdrawal_value: value.withdrawal_value,
                owner: value.owner,
            };
            vestings.insert(key, value);
        }

        let mut locks = BTreeMap::<u64, ParticipantStake>::new();
        for (key, value) in data.locks.into_iter() {
            let value = ParticipantStake {
                remaining_amount: value.remaining_amount,
                last_withdrawal_time: value.last_withdrawal_time,
                withdrawal_period: value.withdrawal_period,
                withdrawal_value: value.withdrawal_value,
                owner: value.owner,
            };
            locks.insert(key, value);
        }

        Ok(Self {
            total: data.total,
            withdraw_value: data.withdraw_value,
            reinvest: data.reinvest,
            reward: data.reward,
            stakes: data.stakes,
            vestings: vestings,
            locks: locks,
            vesting_donor: data.vesting_donor,
            lock_donor: data.lock_donor,
        })
    }
}

#[derive(UnpackAbiPlain)]
pub struct GetDePoolInfoOutput {
    #[abi(bool, name = "poolClosed")]
    pub pool_closed: bool,
    #[abi(uint64, name = "minStake")]
    pub min_stake: u64,
    #[abi(uint64, name = "validatorAssurance")]
    pub validator_assurance: u64,
    #[abi(uint8, name = "participantRewardFraction")]
    pub participant_reward_fraction: u8,
    #[abi(uint8, name = "validatorRewardFraction")]
    pub validator_reward_fraction: u8,
    #[abi(uint64, name = "balanceThreshold")]
    pub balance_threshold: u64,
    #[abi(address, name = "validatorWallet")]
    pub validator_wallet: ton_block::MsgAddressInt,
    #[abi(address, array)]
    pub proxies: Vec<ton_block::MsgAddressInt>,
    #[abi(uint64, name = "stakeFee")]
    pub stake_fee: u64,
    #[abi(uint64, name = "retOrReinvFee")]
    pub ret_or_reinv_fee: u64,
    #[abi(uint64, name = "proxyFee")]
    pub proxy_fee: u64,
}

#[derive(Serialize)]
pub struct DePoolInfo {
    pub pool_closed: bool,
    pub min_stake: u64,
    pub validator_assurance: u64,
    pub participant_reward_fraction: u8,
    pub validator_reward_fraction: u8,
    pub balance_threshold: u64,
    #[serde(with = "serde_address")]
    pub validator_wallet: ton_block::MsgAddressInt,
    #[serde(with = "serde_vec_address")]
    pub proxies: Vec<ton_block::MsgAddressInt>,
    pub stake_fee: u64,
    pub ret_or_reinv_fee: u64,
    pub proxy_fee: u64,
}

impl TryFrom<Vec<ton_abi::Token>> for DePoolInfo {
    type Error = UnpackerError;

    fn try_from(tokens: Vec<ton_abi::Token>) -> Result<Self, Self::Error> {
        let data: GetDePoolInfoOutput = tokens.unpack()?;

        Ok(Self {
            pool_closed: data.pool_closed,
            min_stake: data.min_stake,
            validator_assurance: data.validator_assurance,
            participant_reward_fraction: data.participant_reward_fraction,
            validator_reward_fraction: data.validator_reward_fraction,
            balance_threshold: data.balance_threshold,
            validator_wallet: data.validator_wallet,
            proxies: data.proxies,
            stake_fee: data.stake_fee,
            ret_or_reinv_fee: data.ret_or_reinv_fee,
            proxy_fee: data.proxy_fee,
        })
    }
}
