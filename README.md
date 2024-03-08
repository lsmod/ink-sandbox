### build

```
cargo contract build
```

### build release

```
cargo contract build --release
```

### End-to-End tests

Export the path to substrate-contracts-node
export CONTRACTS_NODE="/YOUR_PATH/substrate-contracts-node"

```
cargo test --features e2e-tests
```

### test

```
cargo test
```

### local node deployment

1. install substrate-contracts-node https://github.com/paritytech/substrate-contracts-node
2. run substrate-contracts-node locally
3. launch ethier https://contracts-ui.substrate.io/ or https://polkadot.js.org/apps
4. upload hold.contract
5. interact with it throught the ui
