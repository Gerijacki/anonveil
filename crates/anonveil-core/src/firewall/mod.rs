//! nftables ruleset generation: types, rule-building logic, and text
//! rendering. See [`nft`] for the design rationale behind the ruleset
//! shape — read it before changing rule order.

pub mod nft;
pub mod render;
pub mod types;

pub use nft::{build_panic_ruleset, build_ruleset};
pub use render::render;
pub use types::{
    ExcludedInterface, ExcludedTcpPort, FirewallConfig, Ipv6Mode, NftChain, NftHook, NftRuleset,
    NftSet, NftSetElement,
};
