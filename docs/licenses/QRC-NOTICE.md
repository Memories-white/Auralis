# QRC decoder adaptation

`core/src/qrc.rs` and `core/src/qrc_tables.rs` adapt the custom DES transform,
permutation tables and QRC decryption order from
[jixunmoe-go/qrc](https://github.com/jixunmoe-go/qrc), commit
`866e996416b0cec7bef648400633f6483c4200d5`.

Copyright (c) 2023 Jixun Wu. The upstream MIT notice is retained in [QRC-LICENSE](QRC-LICENSE).

Adaptation: Go code rewritten in Rust; cloud hex payloads only; bounded zlib
output; malformed input handling; QRC word timeline extraction; conversion to
plain LRC for existing media/Provider consumers. Local QMC-encrypted files are
not supported by this adaptation. The cipher test vector is retained from the
upstream DES tests. UI word progress and text gradients are original Rust/CSS code.
