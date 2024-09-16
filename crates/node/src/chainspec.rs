//! Alphanet chainspec parsing logic.

use once_cell::sync::Lazy;
use reth_chainspec::{
    BaseFeeParams, BaseFeeParamsKind, Chain, ChainHardforks, ChainSpec, EthereumHardfork,
    ForkCondition, OptimismHardfork,
};
use reth_cli::chainspec::ChainSpecParser;
use reth_node_core::args::utils::parse_custom_chain_spec;
use reth_primitives::{b256, constants::ETHEREUM_BLOCK_GAS_LIMIT, U256};
use std::sync::Arc;

/// Alphanet forks.
pub static ALPHANET_FORKS: Lazy<ChainHardforks> = Lazy::new(|| {
    ChainHardforks::new(vec![
        (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Dao.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
        (
            EthereumHardfork::Paris.boxed(),
            ForkCondition::TTD { fork_block: None, total_difficulty: U256::ZERO },
        ),
        (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
        (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Regolith.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Bedrock.boxed(), ForkCondition::Block(0)),
        (OptimismHardfork::Ecotone.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Canyon.boxed(), ForkCondition::Timestamp(0)),
        (EthereumHardfork::Prague.boxed(), ForkCondition::Timestamp(0)),
    ])
});

/// Alphanet dev testnet specification.
pub static ALPHANET_DEV: Lazy<Arc<ChainSpec>> = Lazy::new(|| {
    ChainSpec {
        chain: Chain::dev(),
        genesis: serde_json::from_str(include_str!("../../../etc/alphanet-genesis.json"))
            .expect("Can't deserialize alphanet genesis json"),
        genesis_hash: Some(b256!(
            "2f980576711e3617a5e4d83dd539548ec0f7792007d505a3d2e9674833af2d7c"
        )),
        paris_block_and_final_difficulty: Some((0, U256::from(0))),
        hardforks: ALPHANET_FORKS.clone(),
        base_fee_params: BaseFeeParamsKind::Constant(BaseFeeParams::ethereum()),
        deposit_contract: None,
        ..Default::default()
    }
    .into()
});

/// Alphanet main chain specification.
pub static ALPHANET_MAINNET: Lazy<Arc<ChainSpec>> = Lazy::new(|| {
    ChainSpec {
        chain: Chain::optimism_mainnet(),
        // genesis contains empty alloc field because state at first bedrock block is imported
        // manually from trusted source
        genesis: serde_json::from_str(include_str!("../../../etc/alphanet-genesis.json"))
            .expect("Can't deserialize alphanet genesis json"),
        genesis_hash: Some(b256!(
            "2f980576711e3617a5e4d83dd539548ec0f7792007d505a3d2e9674833af2d7c"
        )),
        paris_block_and_final_difficulty: Some((0, U256::from(0))),
        hardforks: ALPHANET_FORKS.clone(),
        base_fee_params: BaseFeeParamsKind::Variable(
            vec![
                (EthereumHardfork::London.boxed(), BaseFeeParams::optimism()),
                (OptimismHardfork::Canyon.boxed(), BaseFeeParams::optimism_canyon()),
            ]
            .into(),
        ),
        max_gas_limit: ETHEREUM_BLOCK_GAS_LIMIT,
        prune_delete_limit: 10000,
        ..Default::default()
    }
    .into()
});

/// Alphanet chain specification parser.
#[derive(Debug, Clone, Default)]
pub struct AlphanetChainSpecParser;

impl ChainSpecParser for AlphanetChainSpecParser {
    type ChainSpec = ChainSpec;

    const SUPPORTED_CHAINS: &'static [&'static str] = &["alphanet", "dev"];

    fn parse(s: &str) -> eyre::Result<Arc<Self::ChainSpec>> {
        Ok(match s {
            "alphanet" => ALPHANET_MAINNET.clone(),
            "dev" => ALPHANET_DEV.clone(),
            s => {
                let mut chainspec = parse_custom_chain_spec(s)?;

                // NOTE(onbjerg): This is a temporary workaround until we figure out a better way to
                // activate Prague based on a custom fork name. Currently there does not seem to be
                // a good way to do it.
                chainspec.hardforks.insert(EthereumHardfork::Prague, ForkCondition::Timestamp(0));

                Arc::new(chainspec)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use reth_chainspec::EthereumHardforks;
    use reth_cli::chainspec::ChainSpecParser;

    use super::AlphanetChainSpecParser;

    #[test]
    fn chainspec_parser_adds_prague() {
        let mut chainspec_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        chainspec_path.push("../../etc/alphanet-genesis.json");

        let chain_spec = AlphanetChainSpecParser::parse(&chainspec_path.to_string_lossy())
            .expect("could not parse chainspec");

        assert!(
            chain_spec.is_prague_active_at_timestamp(0),
            "prague should be active at timestamp 0"
        );
    }
}
