# audio_license

## Project Title
audio_license

## Project Description
Producers of audio samples and beats (drum kits, loops, vocal chops, full instrumentals) struggle to prove authorship and to monetise re-use of their work in a transparent way. `audio_license` is a Soroban smart contract that lets a producer register a sample on-chain with a content hash and a base price, and lets buyers obtain a usage license — `sync`, `lease`, or `exclusive` — that is publicly verifiable and time-bounded. The goal is a lightweight, trustless proof-of-licensing layer that any DAW plugin, beat marketplace, or content platform can integrate without rebuilding a full rights-management backend.

## Project Vision
Become the on-chain clearing house for short-form audio rights on Stellar. In the long run, `audio_license` should let independent producers monetise their sound libraries globally, give downstream creators (YouTubers, streamers, indie game studios) a one-click way to license cleared samples, and give rights organisations a tamper-proof audit trail. Eventually the same primitives can extend to full song stems, voice performances, and AI-generated audio assets.

## Key Features
- **Sample Registration** — `register_sample` stores the producer's address, a content hash, and a base price on-chain. Each `sample_id` can only be registered once.
- **Three License Types** — `license_sample` issues `sync` (one-off use), `lease` (time-bound, transferable), or `exclusive` (single buyer, non-transferable) licenses, each with its own on-chain rules.
- **Lease Transferability** — `transfer_license` lets a lease holder reassign the license to a new address while preserving the original expiration date.
- **Public Verification** — `verify_license` returns a numeric status code (`0` inactive, `1` sync, `2` lease, `3` exclusive) so any third party can check a buyer-sample pair in a single read.
- **Discoverability Helpers** — `get_producer`, `list_licenses`, `get_sample`, and `get_buyer_samples` make it easy for off-chain indexers and front-ends to render catalogues and dashboards.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** content dApp — see `contracts/audio_license/src/lib.rs` for the full audio_license business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<CAQHHUGQJCDV4F4JDMUW3CNG5Y7TTIIRF7WP7R5NI24J6UHBXH34U4IY>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/e329cb062ebb3b9f9f72b1fc9791e505b08b77a9a66455f2fd416c6f38c52ce7`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(https://prnt.sc/FHVQWU0LoPC4)_`


## Future Scope
- **Token-gated licensing** — accept payment in a custom Soroban asset (e.g. a stablecoin) and forward it to the producer inside `license_sample`, with royalties splits via a payment-stream contract.
- **Royalty distribution** — extend the contract (or add a sibling `audio_royalty` contract) to pay producers on every downstream sync usage, using a pull-payment pattern.
- **Off-chain metadata anchoring** — store an IPFS / Arweave CID alongside the on-chain hash so the audio file itself can be retrieved and re-hashed for tamper detection.
- **Dispute and revocation** — add an arbiter role that can deactivate a license in case of copyright disputes, with a quorum of producer + arbiter signatures.
- **Frontend dApp** — a React + Freighter UI that lets producers drag-and-drop a sample to register it, and lets buyers browse a catalogue, choose a license, and download a signed license receipt.
- **Cross-chain bridge** — mirror license receipts to a public chain for indexers (e.g. via a Wormhole-style attestation) so traditional rights organisations can audit on-chain activity.
- **AI-generated audio** — adapt the same `content_hash + producer` model to register AI-generated samples with the model owner as the producer, enabling transparent provenance for synthetic audio.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `audio_license` (content)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
