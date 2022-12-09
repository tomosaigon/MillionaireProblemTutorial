.PHONY: check
check:
	cargo check

.PHONY: clippy
clippy:
	cargo clippy

PHONY: test
test: unit-test

.PHONY: unit-test
unit-test:
	cargo unit-test

# This is a local build with debug-prints activated. Debug prints only show up
# in the local development chain (see the `start-server` command below)
# and mainnet won't accept contracts built with the feature enabled.
.PHONY: build _build
build: _build compress-wasm
_build:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

# This is a build suitable for uploading to mainnet.
# Calls to `debug_print` get removed by the compiler.
.PHONY: build-mainnet _build-mainnet
build-mainnet: _build-mainnet compress-wasm
_build-mainnet:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

# like build-mainnet, but slower and more deterministic
.PHONY: build-mainnet-reproducible
build-mainnet-reproducible:
	docker run --rm -v "$$(pwd)":/contract \
		--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/contract/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		enigmampc/secret-contract-optimizer:1.0.8

.PHONY: compress-wasm
compress-wasm:
	cp ./target/wasm32-unknown-unknown/release/*.wasm ./contract.wasm
	@## The following line is not necessary, may work only on linux (extra size optimization)
	@# wasm-opt -Os ./contract.wasm -o ./contract.wasm
	cat ./contract.wasm | gzip -9 > ./contract.wasm.gz

.PHONY: schema
schema:
	cargo run --example schema

# Run local development chain with four funded accounts (named a, b, c, and d)
.PHONY: start-server
start-server: # CTRL+C to stop
	docker run -it --rm \
		-p 26657:26657 -p 26656:26656 -p 1317:1317 -p 5000:5000 -p 9090:9090 -p 9091:9091 \
		-v $$(pwd):/root/code \
		--name secretdev ghcr.io/scrtlabs/localsecret:latest

.PHONY: clean
clean:
	cargo clean
	-rm -f ./contract.wasm ./contract.wasm.gz

SCRTFROM=tomoscrt1
ORIGLEN := $(shell secretcli query compute list-code | jq length)
FINALID := $(shell secretcli query compute list-code|jq '1+.[-1].id') # HACK
.PHONY: fun
fun: address

address: build
	@echo codes before store: $(ORIGLEN)
	secretcli tx compute store ./contract.wasm --gas 5000000 --from $(SCRTFROM) --chain-id secretdev-1 -y
	sleep 5
	@echo codes after store and wait 5s \(+1?\)
	secretcli query compute list-code | jq length
	SGX_MODE=SW secretcli tx compute instantiate $(FINALID) '{}' --from $(SCRTFROM) --label work$(FINALID) -y
	sleep 5
	secretcli query compute list-contract-by-code $(FINALID) | jq -r .[0].address > target/address
	cat target/address

CADDR := $(shell cat target/address)
.PHONY: proposal
proposal:
	SGX_MODE=SW secretcli tx compute execute $(CADDR) '{"submit_proposal": {"id": "propcli1", "choice_count": 4, "start_time": "1101", "end_time": "1201"}}' --from $(SCRTFROM) -y

.PHONY: proposal2
proposal2:
	SGX_MODE=SW secretcli tx compute execute $(CADDR) '{"submit_proposal": {"id": "propcli2", "choice_count": 2, "start_time": "1101", "end_time": "1201"}}' --from $(SCRTFROM) -y

.PHONY: countproposals
countproposals:
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"proposal_count": {}}'

.PHONY: showpropcli1
showpropcli1:
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"proposal_by_id": {"proposal_id": "propcli1"}}'

.PHONY: showpropcli2
showpropcli2:
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"proposal_by_id": {"proposal_id": "propcli2"}}'

.PHONY: showproposal
showproposal:
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"current_proposal": {}}'

.PHONY: regvoter
regvoter:
	SGX_MODE=SW secretcli tx compute execute $(CADDR) '{"register_voter": {"proposal_id": "propcli1", "eth_addr": "0xDEAD", "scrt_addr": "secretvoter1", "power": "100"}}' --from $(SCRTFROM) -y

.PHONY: countvoters
countvoters:
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"voter_count": {}}'

.PHONY: castvote
castvote: #proposal regvoter
	SGX_MODE=SW secretcli tx compute execute $(CADDR) '{"cast_vote": {"proposal_id": "propcli1", "eth_addr": "0xDEAD", "scrt_addr": "secretvoter1", "choice": 1}}' --from $(SCRTFROM) -y
	sleep 10
	SGX_MODE=SW secretcli q compute query $(CADDR) '{"who_won": {"proposal_id": "propcli1"}}'