# SecretVoteDemo

Allow Ethereum wallets to register DAO governance token holders for a proposal and hold voting in secret on scrt. Query the scrt contract for winner without revealing individual votes.

Configure secretcli to use a key and connect to a scrt blockchain.

Use this specific localsecret release in case of contract instantiation errors:
`docker run -it -p 9091:9091 -p 26657:26657 -p 1317:1317 -p 5000:5000 --name localsecret ghcr.io/scrtlabs/localsecret:v1.4.0-cw-v1-beta.2`

Compile contract.wasm:
`make build`

Deploy and save contract address:
`make address`

Submit a governance proposal:
`make proposal`

Register a voter:
`make regvoter`

Cast a vote and immediately count the winner:
`make castvote`

### Thanks to MillionaireProblemTutorial
