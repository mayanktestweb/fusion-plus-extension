# NEAR Smart Contract Scripts

This directory contains scripts for interacting with NEAR smart contracts, specifically for creating maker orders in the escrow system.

## Prerequisites

- [NEAR CLI](https://docs.near.org/tools/near-cli) installed and configured
- Bun runtime installed
- Access to the following NEAR accounts:
  - `mayank-hello-world.testnet` (your account)
  - `mayank-token-1.testnet` (token contract)
  - `escrow-src-mayank.testnet` (escrow contract)

## Usage

Follow these steps to create a maker order:

### Step 1: Generate Hex-Encoded Order

First, generate the hex-encoded maker order by running the serialization script:

```bash
bun run serialize-order.ts
```

This will output a hex string that you'll need for Step 3. Copy this hex string.

### Step 2: Register Escrow Contract

Register the escrow contract to receive tokens from the token contract:

```bash
near call mayank-token-1.testnet storage_deposit '{
  "account_id": "escrow-src-mayank.testnet",
  "registration_only": true
}' --accountId mayank-hello-world.testnet --deposit 0.01
```

This command:
- Calls the `storage_deposit` method on the token contract
- Registers the escrow contract to receive tokens
- Pays 0.01 NEAR for storage costs

### Step 3: Create Order

Create the maker order by transferring tokens to the escrow contract:

```bash
near call mayank-token-1.testnet ft_transfer_call '{
  "receiver_id": "escrow-src-mayank.testnet",
  "amount": "1000000000000000000000000",
  "msg": "put_your_hex_encoded_maker_order_here"
}' --accountId mayank-hello-world.testnet --depositYocto 1
```

**Important:** Replace `put_your_hex_encoded_maker_order_here` with the hex string generated in Step 1.

This command:
- Transfers 1 token (1000000000000000000000000 yocto-tokens) to the escrow contract
- Includes the serialized maker order as the `msg` parameter
- Pays 1 yoctoNEAR for storage

## Files

- `serialize-order.ts` - Script to generate hex-encoded maker orders
- `make-order.ts` - Complete script for programmatic order creation (alternative to CLI)

## Troubleshooting

- If Step 2 fails, make sure your account has sufficient NEAR balance
- If Step 3 fails, verify:
  - The hex string from Step 1 is correctly copied
  - Your account has sufficient token balance
  - The escrow contract registration (Step 2) was successful

## Example Output

After running `bun run serialize-order.ts`, you should see output like:

```
Original maker order:
{
  "root_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "token": "mayank-token-1.testnet",
  "total_amount": "1000000000000000000000000",
  "parts": 1,
  "filled_amount": "0",
  "withdrawn_amount": "0",
  "maker": "mayank-hello-world.testnet",
  "expiration": 1754387742654000000
}

Serialized order (hex): 4000000065336230633434323938666331633134396166626634633839393666623932343237616534316534363439623933346361343935393931623738353262383535160000006d6179616e6b2d746f6b656e2d312e746573746e6574000000a1edccce1bc2d3000000000000010000000000000000000000000000000000000000000000000000000000000000001a0000006d6179616e6b2d68656c6c6f2d776f726c642e746573746e657400ec30575bd65818

✅ Successfully generated hex string!
Use this hex string as the msg parameter in ft_transfer_call
```

Copy the hex string from "Serialized order (hex):" and use it in Step 3.

---

## Installation

To install dependencies:

```bash
bun install
```

This project was created using `bun init` in bun v1.1.38. [Bun](https://bun.sh) is a fast all-in-one JavaScript runtime.nstall dependencies:

```bash
bun install
```

To run:

```bash
bun run index.ts
```

This project was created using `bun init` in bun v1.2.18. [Bun](https://bun.sh) is a fast all-in-one JavaScript runtime.
