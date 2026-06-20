#![no_std]

//! # visa_letter — Sponsorship Letter Registry
//!
//! A Soroban smart contract that anchors sponsorship / invitation
//! letters for visa applicants on the Stellar blockchain. A sponsor
//! (employer, family member, university, etc.) issues a letter for an
//! applicant; consulates and border officers can then verify the
//! letter's authenticity, validity and revocation status without
//! trusting a centralised authority.
//!
//! The contract stores a 32-byte hash of the off-chain letter document
//! (PDF, image, etc.) along with metadata such as destination country
//! and an expiry timestamp. The document itself is **not** stored
//! on-chain — only its cryptographic anchor.

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, Symbol};

/// Status of a sponsorship letter for a given applicant.
///
/// Returned by `verify` so a consulate can render a clear decision
/// without re-implementing the rules.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LetterStatus {
    /// No letter has ever been issued for this applicant.
    None = 0,
    /// A letter exists, was issued by the recorded sponsor, has not
    /// been revoked and is not past its `valid_until` timestamp.
    Valid = 1,
    /// A letter exists but `valid_until` is in the past.
    Expired = 2,
    /// A letter exists but has been explicitly revoked by the sponsor.
    Revoked = 3,
}

/// On-chain record for a single sponsorship letter. Keyed by the
/// applicant's address.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Letter {
    /// Address of the sponsor that issued the letter.
    pub sponsor: Address,
    /// SHA-256 hash of the off-chain letter document.
    pub letter_hash: BytesN<32>,
    /// ISO-style country code (e.g. `US`, `DE`, `JP`) the applicant
    /// intends to travel to.
    pub country: Symbol,
    /// Unix timestamp (seconds) at which the letter stops being valid.
    pub valid_until: u64,
    /// `true` once the sponsor has revoked the letter.
    pub revoked: bool,
    /// Short reason string when `revoked == true`, otherwise empty.
    pub revoke_reason: Symbol,
}

/// Storage key holding the `Map<Address, Letter>` registry.
const REGISTRY: Symbol = Symbol::short("registry");

#[contract]
pub struct VisaLetter;

#[contractimpl]
impl VisaLetter {
    // ------------------------------------------------------------------
    // Write paths
    // ------------------------------------------------------------------

    /// Issue a sponsorship letter for `applicant`.
    ///
    /// `sponsor` must authorise the call. If a letter already exists
    /// for the applicant the contract panics — call `revoke` first
    /// (or `renew` to extend) instead of issuing a duplicate.
    ///
    /// * `letter_hash` — SHA-256 hash of the off-chain PDF/document.
    /// * `country`     — destination country code as a short symbol.
    /// * `valid_until` — unix timestamp after which the letter expires.
    pub fn issue_letter(
        env: Env,
        sponsor: Address,
        applicant: Address,
        letter_hash: BytesN<32>,
        country: Symbol,
        valid_until: u64,
    ) {
        // The sponsor is the entity committing to the letter.
        sponsor.require_auth();

        let mut registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .unwrap_or_else(|| Map::new(&env));

        if registry.get(applicant.clone()).is_some() {
            panic!("letter already issued for applicant");
        }

        let letter = Letter {
            sponsor: sponsor.clone(),
            letter_hash,
            country,
            valid_until,
            revoked: false,
            revoke_reason: Symbol::new(&env, ""),
        };

        registry.set(applicant, letter);
        env.storage().instance().set(&REGISTRY, &registry);
    }

    /// Revoke an existing sponsorship letter.
    ///
    /// Only the original sponsor (matched against the stored record)
    /// can revoke. The `reason` symbol is recorded on-chain for
    /// auditors. Panics if no letter exists.
    pub fn revoke(env: Env, sponsor: Address, applicant: Address, reason: Symbol) {
        sponsor.require_auth();

        let mut registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .expect("registry not initialised");

        let mut letter = registry
            .get(applicant.clone())
            .expect("no letter for applicant");

        if letter.sponsor != sponsor {
            panic!("only the original sponsor may revoke");
        }

        if letter.revoked {
            panic!("letter already revoked");
        }

        letter.revoked = true;
        letter.revoke_reason = reason;

        registry.set(applicant, letter);
        env.storage().instance().set(&REGISTRY, &registry);
    }

    /// Renew an existing letter by pushing out its `valid_until`
    /// timestamp. Optionally resets the `revoked` flag if the sponsor
    /// re-confirms sponsorship (`re_confirm == true`).
    pub fn renew(
        env: Env,
        sponsor: Address,
        applicant: Address,
        new_valid_until: u64,
    ) {
        sponsor.require_auth();

        let mut registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .expect("registry not initialised");

        let mut letter = registry
            .get(applicant.clone())
            .expect("no letter for applicant");

        if letter.sponsor != sponsor {
            panic!("only the original sponsor may renew");
        }

        letter.valid_until = new_valid_until;
        registry.set(applicant, letter);
        env.storage().instance().set(&REGISTRY, &registry);
    }

    // ------------------------------------------------------------------
    // Read paths
    // ------------------------------------------------------------------

    /// Compute the current status of an applicant's letter.
    ///
    /// Returns a `u32` matching the `LetterStatus` enum so that
    /// off-chain clients written in JSON-RPC bindings can interpret
    /// the result without a custom decoder:
    ///
    /// * `0` — no letter on file
    /// * `1` — valid (active and un-revoked)
    /// * `2` — expired (`valid_until` in the past)
    /// * `3` — revoked by sponsor
    pub fn verify(env: Env, applicant: Address) -> u32 {
        let registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .unwrap_or_else(|| Map::new(&env));

        let letter = match registry.get(applicant) {
            Some(l) => l,
            None => return LetterStatus::None as u32,
        };

        if letter.revoked {
            return LetterStatus::Revoked as u32;
        }

        if env.ledger().timestamp() > letter.valid_until {
            return LetterStatus::Expired as u32;
        }

        LetterStatus::Valid as u32
    }

    /// Convenience boolean wrapper around `verify`. Useful for
    /// consulates and border kiosks that just need a yes/no answer.
    pub fn is_valid(env: Env, applicant: Address) -> bool {
        Self::verify(env, applicant) == LetterStatus::Valid as u32
    }

    /// Return the sponsor address recorded for an applicant.
    /// Panics if no letter exists.
    pub fn get_sponsor(env: Env, applicant: Address) -> Address {
        let registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .expect("registry not initialised");

        registry
            .get(applicant)
            .expect("no letter for applicant")
            .sponsor
    }

    /// Return the full on-chain `Letter` record for an applicant.
    /// Useful for auditors and off-chain indexers.
    pub fn get_letter(env: Env, applicant: Address) -> Letter {
        let registry: Map<Address, Letter> = env
            .storage()
            .instance()
            .get(&REGISTRY)
            .expect("registry not initialised");

        registry
            .get(applicant)
            .expect("no letter for applicant")
    }
}
