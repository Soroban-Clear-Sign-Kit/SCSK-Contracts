# fixture

A `Forwarder` contract that moves tokens between accounts by calling a token contract. It exists to
produce realistic nested authorization trees and argument shapes for the Clear-Sign Kit.

| Function | Auth tree shape | Purpose |
| --- | --- | --- |
| `forward(from, token, to, amount)` | `forward` -> `transfer` | Simple nested auth. |
| `forward_with_meta(from, token, to, amount, meta, mode)` | `forward_with_meta` -> `transfer` | User-defined types (`Meta` struct, `Mode` enum). |
| `forward_many(from, token, tos, amount)` | `forward_many` -> N x `transfer` | Wide authorization tree. |

All entrypoints require authorization from `from`.
