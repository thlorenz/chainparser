# chainparser [![chainparser Build+Test](https://github.com/thlorenz/chainparser/actions/workflows/build-test-lint.yml/badge.svg)](https://github.com/thlorenz/chainparser/actions/workflows/build-test-lint.yml)

Deserializing Solana accounts using their progam IDL

```rs
use chainparser::{ChainparserDeserializer, IdlProvider, SerializationOpts};

let opts = SerializationOpts {
    pubkey_as_base58: true,
    n64_as_string: false,
    n128_as_string: true,
};

let mut chainparser = ChainparserDeserializer::new(&opts);

// 1. Add IDLS

// Candy Machine IDL
let cndy_program_id = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ";
{
    let idl_json = read_idl_json(&cndy_program_id);
    chainparser
        .add_idl_json(cndy_program_id.to_string(), &idl_json, IdlProvider::Anchor)
        .expect("failed adding IDL JSON");
}

//  Staking Program IDL
let stake_program_id = "StakeSSzfxn391k3LvdKbZP5WVwWd6AsY1DNiXHjQfK";
{
    let idl_json = read_idl_json(&stake_program_id);
    chainparser
        .add_idl_json(stake_program_id.to_string(), &idl_json, IdlProvider::Anchor)
        .expect("failed adding IDL JSON");
}

// 2. Read Accounts Data

// Stake Account
let stake_acc_data = read_account(
    &stake_program_id,
    "EscrowHistory",
    "5AEHnKRonYWeXWQTCqbfaEY6jHy38ifutWsriVsxsgbL",
);

// Candy Machine Account
let cndy_acc_data = read_account(
    &cndy_program_id,
    "CollectionPDA",
    "4gt6YPtgZp2MYJUP7cAH8E3UiL6mUruYaPprEiyJytQ4",
);

// 3. Deserialize Accounts

// Stake Account
{
    let mut acc_json = String::new();
    chainparser
        .deserialize_account(
            &stake_program_id,
            &mut stake_acc_data.as_slice(),
            &mut acc_json,
        )
        .expect("failed to deserialize account");
    assert!(acc_json.contains("{\"escrow\":\"4uj6fRJzqoNRPktmYqGX1nBkjAJBsimJ4ug77S3Tzj7y\""));
}

// Candy Machine Account
{
    let mut acc_json = String::new();
    chainparser
        .deserialize_account(
            &cndy_program_id,
            &mut cndy_acc_data.as_slice(),
            &mut acc_json,
        )
        .expect("failed to deserialize account");
    assert_eq!(acc_json, "{\"mint\":\"BrqNo3sQFTaq9JevoWYhgagJEjE3MmTgYonfaHV5Mf3E\",\"candyMachine\":\"DpBwktkJsEPTtsRpD8kCFGwEUjwTkXARSGSTQ7MJr4kE\"}");
}
```

## LICENSE

MIT
