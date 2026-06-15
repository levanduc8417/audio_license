#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

/// License types that a buyer can obtain for an audio sample.
const LICENSE_SYNC: &str = "sync";
const LICENSE_LEASE: &str = "lease";
const LICENSE_EXCLUSIVE: &str = "exclusive";

/// Return codes emitted by `verify_license`.
const STATUS_INACTIVE: u32 = 0;
const STATUS_SYNC: u32 = 1;
const STATUS_LEASE: u32 = 2;
const STATUS_EXCLUSIVE: u32 = 3;

#[contracttype]
#[derive(Clone)]
pub struct Sample {
    pub producer: Address,
    pub content_hash: Symbol,
    pub base_price: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct License {
    pub holder: Address,
    pub sample_id: Symbol,
    pub license_type: Symbol,
    pub issued_at: u64,
    pub expires_at: u64,
    pub active: bool,
}

#[contract]
pub struct AudioLicense;

#[contractimpl]
impl AudioLicense {
    /// Register a new audio sample on-chain. The producer stores the sample
    /// identifier, a content hash (e.g. SHA-256 of the audio file) and a base
    /// price. Each `sample_id` may only be registered once.
    pub fn register_sample(
        env: Env,
        producer: Address,
        sample_id: Symbol,
        content_hash: Symbol,
        base_price: u64,
    ) {
        producer.require_auth();

        let key = (symbol_short!("sample"), sample_id.clone());
        if env
            .storage()
            .instance()
            .get::<_, Sample>(&key)
            .is_some()
        {
            panic!("sample already registered");
        }

        let sample = Sample {
            producer: producer.clone(),
            content_hash,
            base_price,
            created_at: env.ledger().timestamp(),
        };
        env.storage().instance().set(&key, &sample);

        let producer_count_key = (symbol_short!("p_count"), producer.clone());
        let count: u32 = env
            .storage()
            .instance()
            .get(&producer_count_key)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&producer_count_key, &(count + 1));
    }

    /// Buy a license for an existing sample. The buyer chooses a license type
    /// (`sync`, `lease`, or `exclusive`) and an expiration timestamp. Only one
    /// `exclusive` license may be issued per sample. The function does not
    /// move any tokens — payment is handled off-chain for this MVP.
    pub fn license_sample(
        env: Env,
        buyer: Address,
        sample_id: Symbol,
        license_type: Symbol,
        expires_at: u64,
    ) {
        buyer.require_auth();

        let sample_key = (symbol_short!("sample"), sample_id.clone());
        let _sample: Sample = env
            .storage()
            .instance()
            .get(&sample_key)
            .expect("sample not found");

        let lt = license_type.clone();
        if lt != Symbol::new(&env, LICENSE_SYNC)
            && lt != Symbol::new(&env, LICENSE_LEASE)
            && lt != Symbol::new(&env, LICENSE_EXCLUSIVE)
        {
            panic!("invalid license type");
        }

        if expires_at <= env.ledger().timestamp() {
            panic!("expires_at must be in the future");
        }

        if lt == Symbol::new(&env, LICENSE_EXCLUSIVE) {
            let exclusive_key = (symbol_short!("excl"), sample_id.clone());
            if env.storage().instance().has(&exclusive_key) {
                panic!("exclusive license already granted");
            }
            env.storage().instance().set(&exclusive_key, &buyer);
        }

        let license = License {
            holder: buyer.clone(),
            sample_id: sample_id.clone(),
            license_type: lt,
            issued_at: env.ledger().timestamp(),
            expires_at,
            active: true,
        };
        let license_key = (symbol_short!("lic"), sample_id.clone(), buyer.clone());
        env.storage().instance().set(&license_key, &license);

        let count_key = (symbol_short!("l_count"), sample_id.clone());
        let count: u32 = env
            .storage()
            .instance()
            .get(&count_key)
            .unwrap_or(0);
        env.storage().instance().set(&count_key, &(count + 1));
    }

    /// Transfer an active `lease` license from the current holder to a new
    /// holder. Sync and exclusive licenses are non-transferable; expired
    /// leases are rejected. The expiration timestamp is preserved.
    pub fn transfer_license(
        env: Env,
        holder: Address,
        sample_id: Symbol,
        new_holder: Address,
    ) {
        holder.require_auth();

        let license_key = (symbol_short!("lic"), sample_id.clone(), holder.clone());
        let mut license: License = env
            .storage()
            .instance()
            .get(&license_key)
            .expect("license not found");

        if !license.active {
            panic!("license is not active");
        }
        if license.license_type != Symbol::new(&env, LICENSE_LEASE) {
            panic!("only lease licenses can be transferred");
        }
        if license.expires_at <= env.ledger().timestamp() {
            panic!("license has expired");
        }

        license.active = false;
        env.storage().instance().set(&license_key, &license);

        let transferred = License {
            holder: new_holder.clone(),
            sample_id: sample_id.clone(),
            license_type: license.license_type.clone(),
            issued_at: env.ledger().timestamp(),
            expires_at: license.expires_at,
            active: true,
        };
        let new_key = (symbol_short!("lic"), sample_id, new_holder);
        env.storage().instance().set(&new_key, &transferred);
    }

    /// Return the status of a holder's license for a given sample.
    /// `0` = inactive / not found / expired, `1` = sync, `2` = lease,
    /// `3` = exclusive. Useful for downstream verifiers and dispute checks.
    pub fn verify_license(env: Env, holder: Address, sample_id: Symbol) -> u32 {
        let license_key = (symbol_short!("lic"), sample_id, holder);
        let license: License = match env.storage().instance().get(&license_key) {
            Some(l) => l,
            None => return STATUS_INACTIVE,
        };

        if !license.active {
            return STATUS_INACTIVE;
        }
        if license.expires_at <= env.ledger().timestamp() {
            return STATUS_INACTIVE;
        }

        let sync = Symbol::new(&env, LICENSE_SYNC);
        let lease = Symbol::new(&env, LICENSE_LEASE);
        let exclusive = Symbol::new(&env, LICENSE_EXCLUSIVE);

        if license.license_type == sync {
            STATUS_SYNC
        } else if license.license_type == lease {
            STATUS_LEASE
        } else if license.license_type == exclusive {
            STATUS_EXCLUSIVE
        } else {
            STATUS_INACTIVE
        }
    }

    /// Return the original producer address for a registered sample.
    pub fn get_producer(env: Env, sample_id: Symbol) -> Address {
        let key = (symbol_short!("sample"), sample_id);
        let sample: Sample = env
            .storage()
            .instance()
            .get(&key)
            .expect("sample not found");
        sample.producer
    }

    /// Return the total number of licenses that have been issued for a sample
    /// (active + transferred-away + expired). Useful for sales dashboards.
    pub fn list_licenses(env: Env, sample_id: Symbol) -> u32 {
        let count_key = (symbol_short!("l_count"), sample_id);
        env.storage().instance().get(&count_key).unwrap_or(0)
    }

    /// Return the full `Sample` record for off-chain indexers / UI rendering.
    pub fn get_sample(env: Env, sample_id: Symbol) -> Sample {
        let key = (symbol_short!("sample"), sample_id);
        env.storage()
            .instance()
            .get(&key)
            .expect("sample not found")
    }

    /// Return the list of sample IDs a buyer has ever licensed.
    pub fn get_buyer_samples(env: Env, buyer: Address) -> Vec<Symbol> {
        let key = (symbol_short!("buyer"), buyer);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&env))
    }
}
