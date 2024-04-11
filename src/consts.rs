use once_cell::sync::Lazy;
use primitive_types::U256;

pub const ZERO: U256 = U256::zero();
pub const ONE: U256 = U256::one();
pub static TWO: Lazy<U256> = Lazy::new(|| 2.into());
pub static PRIME: Lazy<U256> = Lazy::new(|| (1u128 + 407 * (1 << 119)).into());
pub static GENERATOR: Lazy<U256> = Lazy::new(|| 85408008396924667383611388730472331217u128.into());
