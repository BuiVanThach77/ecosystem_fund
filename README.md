# ecosystem_fund

## Project Title
ecosystem_fund

## Project Description
Ecosystem funding in Web3 often suffers from a lack of transparency and accountability: backers cannot easily see how their contributions are spent, and grantees have no auditable trail for the milestones they complete. **ecosystem_fund** solves this by providing a fully on-chain, milestone-based grant management system built on Stellar's Soroban platform. Backers pledge capital into a shared pool, a council of admins awards grants to projects, grantees report milestone completion, the council verifies each milestone, and any unspent funds can be clawed back if a grantee misses their commitments. The contract is intentionally simple and tracks balances internally, so it can be audited end-to-end in a single read.

## Project Vision
Our vision is to become the trust layer for community-driven ecosystem development on Stellar. We aim to empower DAOs, developer collectives, and open-source foundations to allocate capital transparently, track progress immutably, and hold grantees accountable — all without intermediaries. In the long term, ecosystem_fund will serve as public-good infrastructure for any community that wants to fund public goods with auditable, on-chain governance and a verifiable chain of custody from pledge to milestone.

## Key Features
- **Pledge Pool** — Any address can pledge into the shared grant pool, building a transparent treasury that is queryable on-chain.
- **Council-Governed Awards** — Only the configured admin (council) can award grants, ensuring capital is deployed through a trusted decision-making process.
- **Milestone-Based Reporting** — Grants are split into an ordered list of milestones; grantees report progress one milestone at a time, in order.
- **Council Verification** — Each reported milestone must be verified by the council before it is considered complete, providing a check-and-balance between grantees and the council.
- **Clawback for Unspent Funds** — If a grantee fails to deliver, the council can claw back the unspent portion of a grant and return it to the pool for re-allocation.
- **Public Status Tracking** — Every grant exposes a public status code (`active`, `completed`, or `clawed back`) plus a human-readable reason, making the full lifecycle of a grant auditable from any Stellar block explorer.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** community dApp — see `contracts/ecosystem_fund/src/lib.rs` for the full ecosystem_fund business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CBTALXMPNT5HH5EXCTXORLBUBHGQEREP7W4LWJ2IA3M534NUJBDAXJYZ`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/d6431e7aecc4176a0cc26c8ecd2a95a2fc17ba9c7640d868e9feb2b2dd65977a`


## Future Scope
- **Multi-Admin Council** — Replace the single admin with a weighted council and on-chain voting for grant approvals and clawbacks.
- **Token-Based Payouts** — Integrate a Soroban token (such as a stablecoin) so that verified milestones release capital directly to the grantee's wallet on-chain.
- **Grant Amendments** — Allow the council to extend deadlines, adjust milestone counts, or re-allocate funds between active grants.
- **Time-Locked Milestones** — Add a deadline per milestone that auto-expires the grant and triggers a clawback if the grantee misses it.
- **Frontend Dashboard** — Build a web dashboard so backers, grantees, and council members can monitor pool balances, grant progress, and the reason for any clawback in real time.
- **Public Analytics** — Expose aggregate metrics (total pledged, total awarded, total clawed back, success rate per grant) for ecosystem transparency.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `ecosystem_fund` (community)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
