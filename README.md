# NEAR Escrow Contract with Hash Time Locked Contracts (HTLC)

A sophisticated atomic swap escrow system built on NEAR Protocol that enables secure cross-chain token exchanges using Hash Time Locked Contracts (HTLC). The contract allows makers to deposit tokens and resolvers to fill orders by providing safety deposits, with cryptographic secrets ensuring trustless execution across different blockchain networks.

## 🚀 Features

- **Atomic Swaps**: Secure peer-to-peer token exchanges without intermediaries
- **Hash Time Locked Contracts (HTLC)**: Time-bound transactions with cryptographic guarantees
- **Partial Fill Support**: Orders can be filled in multiple parts using Merkle trees
- **Safety Deposits**: Resolver deposits ensure commitment to fulfill orders
- **Multi-Chain Support**: Designed for cross-chain atomic swaps
- **Timelock Protection**: Multiple timelock layers for withdrawal and cancellation
- **Advanced Safe Transfer**: Automatic NEP-141 registration and storage balance management

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Maker (Alice) │    │ Escrow Contract  │    │Resolver (Bob)   │
│                 │    │                  │    │                 │
│ 1. Creates Order│───▶│ 2. Stores Order  │◄───│ 3. Fills Order  │
│ 4. Waits for    │    │ 5. Validates     │    │ 6. Provides     │
│    Secret       │◄───│    Fill & Locks  │───▶│    Deposit      │
│ 7. Claims Tokens│    │    Tokens        │    │ 8. Gets Secret  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🛡️ Advanced Safe Transfer Implementation

This escrow contract implements a sophisticated safe transfer mechanism that automatically handles NEP-141 token registration and storage balance management for users who haven't registered with fungible token contracts:

### Automatic Registration & Storage Management
```rust
// The contract intelligently checks if users are registered
// and automatically handles registration + storage deposits
pub fn safe_transfer_to(&self, token: &AccountId, to: &AccountId, amount: NearToken) -> Promise {
    // 1. Check if recipient is registered with the token contract
    ext_ft::storage_balance_of(to.clone(), token, NO_DEPOSIT, Gas::from_tgas(5))
        .then(
            // 2. If not registered, automatically register them
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(10))
                .on_storage_balance_checked(token.clone(), to.clone(), amount)
        )
}

// Handles registration and ensures sufficient storage
pub fn on_storage_balance_checked(
    &mut self,
    token: AccountId,
    to: AccountId, 
    amount: NearToken,
    #[callback_result] storage_result: Result<Option<StorageBalance>, PromiseError>
) -> Promise {
    match storage_result {
        Ok(Some(_)) => {
            // User is registered - direct transfer
            ext_ft::ft_transfer(to, amount, None, &token, ONE_YOCTO, Gas::from_tgas(10))
        },
        Ok(None) | Err(_) => {
            // User not registered - register them first, then transfer
            ext_ft::storage_deposit(Some(to.clone()), None, &token, STORAGE_DEPOSIT, Gas::from_tgas(10))
                .then(
                    ext_ft::ft_transfer(to, amount, None, &token, ONE_YOCTO, Gas::from_tgas(10))
                )
        }
    }
}
```

### Why This Matters
- **Seamless UX**: Users don't need to manually register with every token contract
- **Gas Efficiency**: Only registers when necessary, avoiding duplicate registrations
- **Error Prevention**: Prevents failed transfers due to unregistered recipients
- **Storage Cost Handling**: Automatically covers NEP-141 storage requirements
- **Cross-Token Support**: Works with any NEP-141 compliant token

## 🧩 Core Components

### MakerOrder
```rust
pub struct MakerOrder {
    root_hash: String,              // hashlock for single, merkle_root for multi fill
    token: AccountId,               // token used by maker to make exchange
    total_amount: NearToken,        // total tokens maker is putting
    parts: u16,                     // parts the order is divided in (default 1)
    filled_amount: NearToken,       // taker placed amount
    withdrawn_amount: NearToken,    // withdrawn amount
    maker: AccountId,               // maker account
    expiration: u64                 // timestamp beyond which user can run self withdrawal
}
```

### ResolverOrderFill
```rust
pub struct ResolverOrderFill {
    immutables: Immutables          // Contains all swap parameters
}
```

## ⏰ Time Lock Mechanics

The contract implements a sophisticated timelock system with four distinct phases:

```
Timeline: ────────────────────────────────────────▶
          T0    T1         T2          T3        T4
          │     │          │           │         │
          │     │ Private  │  Public   │ Private │ Public
          │     │Withdraw  │ Withdraw  │ Cancel  │ Cancel
          │     │(Secret)  │(+Secret)  │(Taker)  │(Anyone)
          │     │          │           │         │
     Order│     └─Resolver can withdraw with secret
     Created     └─────────Maker can withdraw WITH SECRET
                           └─────────Taker can cancel order
                                     └─────────Anyone can cancel
```

**Important**: All withdrawal functions require the secret - the "public" withdrawal only means the timelock has expired!

## 🔍 Partial Fill Support

For orders with `parts > 1`, the contract supports partial fills using Merkle trees with **parts + 1** secrets:

```
Total Order: 100 tokens, 4 parts (25 tokens each) = 5 secrets (S0-S4)

Merkle Tree:
              Root
           /        \
       Hash1          Hash2
      /     \        /     \
   Hash3   Hash4  Hash5   Hash6
   /  \     /  \    /  \    |
  S0  S1   S2  S3  S4   -   -

Parts:  [25] [25] [25] [25]
Secrets: S0   S1   S2   S3/S4

- S0-S2: Unlock first three parts (75 tokens)
- S3: Can unlock partial amount from last part
- S4: MUST be used to complete the order entirely
```

**Key Rule**: Whoever completes the order entirely **must use the extra secret (S4)** for the final completion.

## 🔧 How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## 🚀 How to Deploy?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

For debugging purposes:
```bash
cargo near deploy build-non-reproducible-wasm <account-id>
```

For production ready smart contract:
```bash
cargo near deploy build-reproducible-wasm <account-id>
```

## 📚 API Reference

### Main Functions:

- `ft_on_transfer(sender, amount, msg)` - Called by FT contract to create maker orders
- `create_resolver_fill_order(immutables, idx?, merkle_proof?)` - Resolver fills order
- `withdraw(secret, immutables)` - Withdraw with secret revelation
- `public_withdraw(secret, immutables)` - Withdraw after timelock **WITH SECRET**
- `cancel(immutables)` - Cancel order (time-locked)
- `public_cancel(immutables)` - Public cancellation after timeout

### View Functions:

- `check_order(immutables) -> bool` - Check if order exists

### Safe Transfer Functions:

- `safe_transfer_to(token, to, amount)` - Intelligent transfer with auto-registration
- `on_storage_balance_checked()` - Callback handling registration logic

## 🔐 Security Considerations

1. **Secret Management**: Secrets should be generated securely and only revealed when safe
2. **Completion Secret**: The extra secret (S_parts+1) must be protected until final completion
3. **Timelock Ordering**: Ensure timelock values are properly ordered
4. **Safety Deposits**: Resolver deposits ensure commitment
5. **Merkle Proofs**: Validate all merkle proofs for partial fills
6. **Overflow Protection**: All mathematical operations use checked arithmetic
7. **Storage Security**: Safe transfer handles storage deposits without exposing users to failed transactions

## 🛠️ Technical Implementation

Built using NEAR SDK for Rust, the contract leverages Borsh serialization for efficient data storage and retrieval. The core architecture centers around two main data structures: `MakerOrder` storing order details and `ResolverOrderFill` containing immutable swap parameters. For partial fills, the system implements a Merkle tree verification system where orders with N parts require N+1 secrets - the first N secrets unlock individual parts, while the final secret must be used to complete any remaining amount, ensuring proper order completion tracking. The contract integrates with NEAR's fungible token standard through the `ft_on_transfer` callback, automatically processing incoming token transfers and validating maker orders. The sophisticated safe transfer mechanism uses promise chaining to check storage balances, handle automatic registration, and ensure seamless token transfers even for unregistered users. Safety deposits are enforced through NEAR's payable function decorators, and all mathematical operations use checked arithmetic to prevent overflows. The timelock system uses NEAR's block timestamp for time-based validations, while the secret validation relies on SHA-256 hashing provided by the shared utility library. Storage is optimized using NEAR's `LookupMap` for O(1) access to orders, with automatic cleanup when orders are fully withdrawn or cancelled.

## 🔗 Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)

---

**Built with ❤️ on NEAR Protocol**