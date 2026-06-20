# visa_letter

## Project Title

visa_letter — Sponsorship Letter Registry on Stellar / Soroban

## Project Description

Every year, millions of visa applicants must present a physical or
emailed "sponsorship letter" — issued by an employer, university or
family member — to a consulate before a travel visa is granted. These
letters are trivially forged (a single PDF attached to an email), and
consulates have no reliable way to cross-check whether a sponsor
really wrote the letter they received, or whether the letter has since
been withdrawn.

`visa_letter` solves this by anchoring a SHA-256 hash of the
sponsorship letter on the Stellar blockchain via a Soroban smart
contract. The sponsor issues a letter on-chain for a specific
applicant, recording the destination country and an expiry timestamp.
The applicant then shows the original PDF to the consulate; the
consulate queries the contract to confirm that the document's hash
matches what was anchored, who issued it, and whether the letter is
still active. If the sponsor withdraws support, they revoke the
letter on-chain and consulates see the change immediately.

No XLM is transferred by the contract — only public, verifiable
attestations are written, which keeps the cost per letter well under
one US cent.

## Project Vision

Our long-term goal is to make sponsorship letters as trustworthy as
diplomatic notes: tamper-evident, instantly verifiable, and globally
portable. By moving the *attestation* layer — not the personal data —
on-chain, `visa_letter` gives consulates a single source of truth for
"did this sponsor really stand behind this applicant?" while
preserving privacy. The same primitive can later be reused for
immigration bonds, university acceptance letters, and travel
insurance certificates.

## Key Features

- **`issue_letter`** — A sponsor anchors a 32-byte hash of the
  sponsorship document, the destination country code, and a
  `valid_until` timestamp. The applicant address is the registry key.
- **`revoke`** — The original sponsor can withdraw a letter at any
  time, recording a short on-chain reason. Once revoked, consulates
  see status `Revoked` immediately.
- **`renew`** — A sponsor can push out the expiry of an existing
  letter without re-uploading a new document.
- **`verify`** — Returns a numeric status (`0` none, `1` valid,
  `2` expired, `3` revoked) computed against the current ledger
  timestamp, suitable for kiosk-style "scan and check" flows.
- **`is_valid`** / **`get_sponsor`** / **`get_letter`** — Thin
  read-only helpers so front-ends, consulates and auditors can query
  the registry without re-implementing the state machine.
- **Authorisation** — Every state-changing function uses
  `require_auth()`, so only the recorded sponsor can revoke or renew
  their own letters.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** travel dApp — see `contracts/visa_letter/src/lib.rs` for the full visa_letter business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CCCF3UUMGVYUBNSNZ3T2ZAXTQJ2PECCVASUHBW2D4LBK6H5GFR5YCRGQ`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/7f3ea087b8e56bbb7d6584b9605a6c4722bd245f84e5c1a207e072985c562ec8`



## Future Scope

- **Multi-sponsor co-signing** — require N-of-M sponsor signatures
  (e.g. employer + host family) for high-risk destinations.
- **Per-country validators** — a separate contract that recognises
  consulates as authorised verifiers and emits verifiable
  "checked-by" receipts.
- **Off-chain document storage integration** — store the full PDF on
  IPFS/Filecoin and pin the CID alongside the hash on-chain.
- **Frontend dApp** — a React/Next.js UI with Freighter wallet
  integration so sponsors can issue/revoke letters and consulates can
  scan a QR code to verify them in seconds.
- **Privacy-preserving variant** — replace the public applicant
  address with a ZK-friendly commitment so the registry no longer
  leaks the applicant's Stellar identity while remaining verifiable.
- **Mainnet launch checklist** — security audit, key-management
  guide for consulates, and a SaaS wrapper for travel agencies.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `visa_letter` (travel)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
