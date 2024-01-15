# [Mitre](https://attack.mitre.org/) enum generator

This python script generates `kraken-proto/src/mitre.rs`.

It takes no arguments and writes the rust code to `stdout`.

The json is takes its data from, is downloaded from [github](https://github.com/mitre/cti/blob/230f6c26b1554d90993e59cb60e0aeefef530147/enterprise-attack/enterprise-attack.json) and cached in `/tmp/`.