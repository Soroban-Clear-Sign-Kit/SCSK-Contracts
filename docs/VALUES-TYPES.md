# Soroban Values and ScVal Type Coverage

This document details the comprehensive value type coverage implemented in `contracts/values`. The contract serves as a test suite and reference contract exercising every data type that a Soroban smart contract call can accept, return, and emit via multi-topic events.

---

## 1. Type Coverage Matrix

| Value Type Category | Rust SDK Type | ScVal Discriminant | Function Name | Event Emitted | Test Name |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Nested Vec** | `Vec<Vec<String>>` | `ScVal::Vec` | `echo_nested_vec` | `NestedVecEvent` (3 topics) | `test_echo_nested_vec` |
| **Nested Map** | `Map<Symbol, Map<String, i128>>` | `ScVal::Map` | `echo_nested_map` | `NestedMapEvent` (3 topics) | `test_echo_nested_map` |
| **Tuples** | `(Address, Symbol, i128, bool)` | `ScVal::Vec` | `echo_tuple` | `TupleEvent` (4 topics) | `test_echo_tuple` |
| **Option (Some)** | `Option<String>` | `ScVal::String` | `echo_option` | `OptionEvent` (3 topics) | `test_echo_option_some` |
| **Option (None)** | `Option<String>` | `ScVal::Void` | `echo_option` | `OptionEvent` (3 topics) | `test_echo_option_none` |
| **Enum with Data** | `CustomEnum` | `ScVal::Vec` | `echo_enum` | `EnumEvent` (3 topics) | `test_echo_enum` |
| **Bytes (Dynamic)** | `Bytes` | `ScVal::Bytes` | `echo_bytes` | `BytesEvent` (3 topics) | `test_echo_bytes` |
| **Fixed BytesN** | `BytesN<32>` | `ScVal::Bytes` | `echo_bytes_n` | `BytesNEvent` (3 topics) | `test_echo_bytes_n` |
| **String** | `String` | `ScVal::String` | `echo_string` | `StringEvent` (3 topics) | `test_echo_string` |
| **Symbol** | `Symbol` | `ScVal::Symbol` | `echo_symbol` | `SymbolEvent` (3 topics) | `test_echo_symbol` |
| **Address** | `Address` | `ScVal::Address` | `echo_address` | `AddressEvent` (3 topics) | `test_echo_address` |
| **i128** | `i128` | `ScVal::I128` | `echo_i128` | `I128Event` (3 topics) | `test_echo_i128` |
| **u128** | `u128` | `ScVal::U128` | `echo_u128` | `U128Event` (3 topics) | `test_echo_u128` |
| **i256** | `I256` | `ScVal::I256` | `echo_i256` | `I256Event` (3 topics) | `test_echo_i256` |
| **u256** | `U256` | `ScVal::U256` | `echo_u256` | `U256Event` (3 topics) | `test_echo_u256` |
| **Duration** | `Duration` | `ScVal::Duration` | `echo_duration` | `DurationEvent` (3 topics) | `test_echo_duration` |
| **Timepoint** | `Timepoint` | `ScVal::Timepoint` | `echo_timepoint` | `TimepointEvent` (3 topics) | `test_echo_timepoint` |
| **Struct** | `CustomStruct` | `ScVal::Map` | `echo_struct` | `StructEvent` (4 topics) | `test_echo_struct` |

---

## 2. Multi-Topic Event Specification

Soroban events support up to 4 indexed topics. Each function in `contracts/values` publishes structured events utilizing 3 to 4 topics:

### 3-Topic Events
- **Topic 0**: `Symbol("values")` (contract namespace)
- **Topic 1**: `Symbol("<type_category>")` (e.g. `nested_vec`, `nested_map`, `bytes`, `timepoint`)
- **Topic 2**: `tag: Symbol` (dynamic discriminator passed by caller)
- **Data Payload**: The single-value data payload carrying the returned typed argument.

### 4-Topic Events
- `TupleEvent`:
  - **Topics**: `["values", "tuple", tag: Symbol, active: bool]`
  - **Data Payload**: `(Address, Symbol, i128, bool)`
- `StructEvent`:
  - **Topics**: `["values", "struct", tag: Symbol, id: u32]`
  - **Data Payload**: `CustomStruct { id, name, payload }`

---

## 3. Data Structures & Variants

### `CustomEnum`
Demonstrates data-carrying tagged enums with various payload shapes:
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomEnum {
    Empty,                               // Unit variant
    Value(u64),                          // Single primitive variant
    Pair(String, i128),                  // Multi-element tuple variant
    Record(Symbol, Address),             // Multi-type contract reference variant
}
```

### `CustomStruct`
Demonstrates structured objects combining primitives, strings, and dynamic byte buffers:
```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomStruct {
    pub id: u32,
    pub name: String,
    pub payload: Bytes,
}
```

---

## 4. Test Verification

All 18 tests in `contracts/values/src/test.rs` pass, verifying both value roundtripping (input matches output) and exact event emissions matching the expected multi-topic schemas.
