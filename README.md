# passport_renew

## Project Title
passport_renew

## Project Description
passport_renew is a Soroban smart contract that brings passport renewal workflows on-chain in a transparent, auditable, and tamper-proof way. A citizen submits a renewal application that references the hash of their current passport; a passport-office officer reviews it and either approves (recording the new passport's hash and validity end-date, and marking the old one expired) or rejects it with a reason. The citizen can also cancel a pending application at any time. Because every state transition is signed and persisted on the Stellar ledger, applicants and officers get a single shared, immutable history of every renewal — eliminating lost paperwork, ambiguous status checks, and disputes about whether a passport is still valid.

## Project Vision
The long-term vision for passport_renew is to become a foundational identity-credential primitive for the Stellar ecosystem — a building block that any government, employer, airline, or border authority can plug into to verify passport status in real time without trusting a single central database. The MVP captures the full happy-path lifecycle (file -> approve / reject / cancel) and the "old passport is now expired" state transition; future iterations aim to add multi-officer quorum approvals, off-chain document anchoring (IPFS / Arweave), cross-border mutual recognition, and integration with Stellar-native stablecoins for renewal fee escrow.

## Key Features
- **Citizen self-service renewal application** — `apply_renewal` lets any signed-in citizen file a new application tied to their old passport hash in a single transaction.
- **Officer approval with old-passport expiration** — `approve` records the new passport's hash and validity end-date, then atomically marks the old passport hash as expired so it can never be reused.
- **Officer rejection with reason** — `reject` closes a pending application with a short, on-chain reason code for full auditability.
- **Citizen cancellation** — `cancel` allows the original applicant to withdraw a pending application and provide a reason, preventing stale "ghost" requests.
- **Transparent status lookups** — `get_status` returns the current lifecycle state (Pending / Approved / Rejected / Cancelled) of any application id, and `is_expired` lets anyone verify that an old passport hash is no longer valid.
- **Officer access control** — a one-time `init` registers the passport-office officer; only that address can approve or reject applications.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** travel dApp — see `contracts/passport_renew/src/lib.rs` for the full passport_renew business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CD5VH3AKFZBNOLKXSI2W7I7U4P765Q2VGCXMJSWQXBJZMMRIM26XETDA`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/e89fcb115027134d5b26c3dc73ac4d2d135dc5f2c72cfaab67165c4d2b9f6fee`

## Future Scope
- **Multi-officer quorum approvals** — require N-of-M officer signatures for high-risk renewals (lost-passport cases, diplomatic passports, etc.).
- **Document anchoring** — store the encrypted passport scan's IPFS / Arweave CID on-chain and verify it against the on-chain hash at issuance time.
- **Renewal-fee escrow** — integrate a Stellar stablecoin (USDC) payment that is held in contract escrow and released to the treasury only on approval.
- **Off-chain notification layer** — emit Soroban events on every state transition so that a backend can email / push-notify the citizen.
- **Dispute and appeal flow** — add `appeal_rejected` so a rejected citizen can request a second-officer review with a bond.
- **Time-bound auto-expiry** — automatically transition a `Pending` application to `Cancelled` after a configurable review window (using `env.ledger().timestamp()`).
- **Frontend dApp** — React + Freighter UI with a status-check page and an officer dashboard for review queues.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `passport_renew` (travel)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
