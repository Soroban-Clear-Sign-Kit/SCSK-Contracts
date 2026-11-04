#![no_std]
use soroban_sdk::{
    Address, Bytes, BytesN, Duration, Env, I256, Map, String, Symbol, Timepoint, U256, Vec,
    contract, contractevent, contractimpl, contracttype,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomEnum {
    Empty,
    Value(u64),
    Pair(String, i128),
    Record(Symbol, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomStruct {
    pub id: u32,
    pub name: String,
    pub payload: Bytes,
}

#[contractevent(topics = ["values", "nested_vec"], data_format = "single-value")]
pub struct NestedVecEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Vec<Vec<String>>,
}

#[contractevent(topics = ["values", "nested_map"], data_format = "single-value")]
pub struct NestedMapEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Map<Symbol, Map<String, i128>>,
}

#[contractevent(topics = ["values", "tuple"], data_format = "single-value")]
pub struct TupleEvent {
    #[topic]
    pub tag: Symbol,
    #[topic]
    pub active: bool,
    pub val: (Address, Symbol, i128, bool),
}

#[contractevent(topics = ["values", "option"], data_format = "single-value")]
pub struct OptionEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Option<String>,
}

#[contractevent(topics = ["values", "enum"], data_format = "single-value")]
pub struct EnumEvent {
    #[topic]
    pub tag: Symbol,
    pub val: CustomEnum,
}

#[contractevent(topics = ["values", "bytes"], data_format = "single-value")]
pub struct BytesEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Bytes,
}

#[contractevent(topics = ["values", "bytes_n"], data_format = "single-value")]
pub struct BytesNEvent {
    #[topic]
    pub tag: Symbol,
    pub val: BytesN<32>,
}

#[contractevent(topics = ["values", "string"], data_format = "single-value")]
pub struct StringEvent {
    #[topic]
    pub tag: Symbol,
    pub val: String,
}

#[contractevent(topics = ["values", "symbol"], data_format = "single-value")]
pub struct SymbolEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Symbol,
}

#[contractevent(topics = ["values", "address"], data_format = "single-value")]
pub struct AddressEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Address,
}

#[contractevent(topics = ["values", "i128"], data_format = "single-value")]
pub struct I128Event {
    #[topic]
    pub tag: Symbol,
    pub val: i128,
}

#[contractevent(topics = ["values", "u128"], data_format = "single-value")]
pub struct U128Event {
    #[topic]
    pub tag: Symbol,
    pub val: u128,
}

#[contractevent(topics = ["values", "i256"], data_format = "single-value")]
pub struct I256Event {
    #[topic]
    pub tag: Symbol,
    pub val: I256,
}

#[contractevent(topics = ["values", "u256"], data_format = "single-value")]
pub struct U256Event {
    #[topic]
    pub tag: Symbol,
    pub val: U256,
}

#[contractevent(topics = ["values", "duration"], data_format = "single-value")]
pub struct DurationEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Duration,
}

#[contractevent(topics = ["values", "timepoint"], data_format = "single-value")]
pub struct TimepointEvent {
    #[topic]
    pub tag: Symbol,
    pub val: Timepoint,
}

#[contractevent(topics = ["values", "struct"], data_format = "single-value")]
pub struct StructEvent {
    #[topic]
    pub tag: Symbol,
    #[topic]
    pub id: u32,
    pub val: CustomStruct,
}

#[contract]
pub struct ValuesContract;

#[contractimpl]
impl ValuesContract {
    pub fn echo_nested_vec(env: Env, tag: Symbol, val: Vec<Vec<String>>) -> Vec<Vec<String>> {
        NestedVecEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_nested_map(
        env: Env,
        tag: Symbol,
        val: Map<Symbol, Map<String, i128>>,
    ) -> Map<Symbol, Map<String, i128>> {
        NestedMapEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_tuple(
        env: Env,
        tag: Symbol,
        val: (Address, Symbol, i128, bool),
    ) -> (Address, Symbol, i128, bool) {
        TupleEvent {
            tag,
            active: val.3,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_option(env: Env, tag: Symbol, val: Option<String>) -> Option<String> {
        OptionEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_enum(env: Env, tag: Symbol, val: CustomEnum) -> CustomEnum {
        EnumEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_bytes(env: Env, tag: Symbol, val: Bytes) -> Bytes {
        BytesEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_bytes_n(env: Env, tag: Symbol, val: BytesN<32>) -> BytesN<32> {
        BytesNEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_string(env: Env, tag: Symbol, val: String) -> String {
        StringEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_symbol(env: Env, tag: Symbol, val: Symbol) -> Symbol {
        SymbolEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_address(env: Env, tag: Symbol, val: Address) -> Address {
        AddressEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_i128(env: Env, tag: Symbol, val: i128) -> i128 {
        I128Event { tag, val }.publish(&env);
        val
    }

    pub fn echo_u128(env: Env, tag: Symbol, val: u128) -> u128 {
        U128Event { tag, val }.publish(&env);
        val
    }

    pub fn echo_i256(env: Env, tag: Symbol, val: I256) -> I256 {
        I256Event {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_u256(env: Env, tag: Symbol, val: U256) -> U256 {
        U256Event {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_duration(env: Env, tag: Symbol, val: Duration) -> Duration {
        DurationEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_timepoint(env: Env, tag: Symbol, val: Timepoint) -> Timepoint {
        TimepointEvent {
            tag,
            val: val.clone(),
        }
        .publish(&env);
        val
    }

    pub fn echo_struct(env: Env, tag: Symbol, val: CustomStruct) -> CustomStruct {
        StructEvent {
            tag,
            id: val.id,
            val: val.clone(),
        }
        .publish(&env);
        val
    }
}

#[cfg(test)]
mod test;
