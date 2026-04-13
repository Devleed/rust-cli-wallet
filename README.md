# rust-cli-wallet

A secure, locally-run Ethereum CLI wallet built in Rust. Manage multiple accounts, send tokens, and interact with EVM-compatible networks — all from your terminal, with no data leaving your machine.

## Features

- **Multi-account management** — create, import, and switch between wallets
- **BIP-39 seed phrase generation** — generate new wallets with a 12-word mnemonic
- **Private key import** — import existing wallets via seed phrase or raw private key
- **Encrypted keystore** — accounts are stored locally using Web3 keystore format (AES encryption)
- **Send native tokens** — send ETH or MATIC with gas estimation and USD cost preview
- **ERC-20 token support** — add any ERC-20 token by contract address, check balances, and send
- **Beneficiary management** — save, list, and delete named recipient addresses
- **Multi-network support** — switch between networks without restarting
- **Fiat conversion** — live USD rates via CryptoCompare for balances and transaction costs
- **Gas price estimation** — check current gas costs before committing to a transaction
- **Password management** — change your account password at any time

## Supported Networks

| Network | Chain ID | Coin  |
| ------- | -------- | ----- |
| Mainnet | 1        | ETH   |
| Goerli  | 5        | ETH   |
| Polygon | 137      | MATIC |
| Mumbai  | 80001    | MATIC |

Additional networks can be added by editing `config/chains.json`.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)

### Build & Run

```bash
git clone https://github.com/devleed/rust-cli-wallet
cd rust-cli-wallet
cargo run
```

## Usage

On launch the wallet presents a list of saved accounts. From there you can:

1. **Select an existing account** — enter your password to unlock
2. **Create new** — generates a fresh BIP-39 mnemonic and encrypts the keystore
3. **Import wallet** — enter an existing 12-word seed phrase or private key

Once authenticated, the dashboard exposes all wallet actions:

```
> Send native token
> Change network
> Display balance
> Add token
> Select token
> Beneficiary management
> Change password
> Check gas prices
> Delete account (danger)
> Logout
```

### Sending Tokens

When sending native tokens or ERC-20 tokens, the wallet will:

- Show your current balance
- Let you pick a saved beneficiary or type an address manually
- Estimate gas cost in the native coin and USD
- Ask for confirmation before broadcasting the transaction

## Project Structure

```
src/
├── main.rs          # Entry point
├── account.rs       # Account creation, login, and authenticated dashboard
├── wallet.rs        # Wallet building and ETH send flow
├── tokens.rs        # ERC-20 token management
├── networks.rs      # Network configuration and switching
├── beneficiaries.rs # Saved recipient address management
├── fiat.rs          # Live USD exchange rate fetching
├── provider.rs      # Ethers.rs provider and gas utilities
├── keystore.rs      # Keystore serialization/deserialization
├── utils.rs         # Input helpers, logging, and validation
└── ierc20.rs        # Generated ERC-20 contract bindings

config/
└── chains.json      # Network RPC and explorer configuration

abis/
└── erc20.json       # ERC-20 ABI
```

## Security

- Private keys and seed phrases are **never stored in plaintext** — they are encrypted using the Web3 keystore standard before being written to disk
- All account data is stored locally in an `accounts/` directory; nothing is transmitted to a server
- Passwords are read with `rpassword` to prevent terminal echo

## License

MIT
