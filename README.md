# SecretVoteDemo

Allow Ethereum wallets to register DAO governance token holders for a proposal and hold voting in secret on scrt. Query the scrt contract for winner without revealing individual votes.

Configure secretcli to use a key and connect to a scrt blockchain.

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