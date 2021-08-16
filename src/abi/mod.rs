pub use nekoton_abi::{
    BuildTokenValue, PackAbi, TokenValueExt, UnpackAbi, UnpackAbiPlain, UnpackerError,
    UnpackerResult,
};
pub use once_cell::sync::OnceCell;
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use ton_abi::{Contract, Param, ParamType};

macro_rules! declare_abi {
    ($($contract:ident => $source:literal),*$(,)?) => {$(
        pub fn $contract() -> &'static Contract {
            static ABI: OnceCell<Contract> = OnceCell::new();
            ABI.load(include_bytes!($source))
        }
    )*};
}

declare_abi! {
    depool_v3 => "./DePool.abi.json",
}

trait OnceCellExt {
    fn load(&self, data: &[u8]) -> &Contract;
}

impl OnceCellExt for OnceCell<Contract> {
    fn load(&self, data: &[u8]) -> &Contract {
        self.get_or_init(|| Contract::load(&mut std::io::Cursor::new(data)).expect("Trust me"))
    }
}
