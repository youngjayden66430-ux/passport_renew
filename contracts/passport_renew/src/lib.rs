#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

// ---------------------------------------------------------------------------
// Status codes for a renewal application lifecycle.
// ---------------------------------------------------------------------------
const STATUS_PENDING: u32 = 0;
const STATUS_APPROVED: u32 = 1;
const STATUS_REJECTED: u32 = 2;
const STATUS_CANCELLED: u32 = 3;

/// A full renewal application record stored on the ledger.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Application {
    pub id: u64,
    pub citizen: Address,
    pub old_passport_hash: BytesN<32>,
    pub new_passport_hash: BytesN<32>,
    pub status: u32,
    pub valid_until: u64,
    pub reason: Symbol,
    pub officer: Address,
    pub created_at: u64,
    pub reviewed_at: u64,
}

/// Storage keys used by the contract.
#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Officer,
    AppCounter,
    Application(u64),
    Expired(BytesN<32>),
    Issued(BytesN<32>),
}

#[contract]
pub struct PassportRenew;

#[contractimpl]
impl PassportRenew {
    // -----------------------------------------------------------------------
    /// Initialize the contract with the address of the passport office
    /// officer that is allowed to approve / reject applications. Must be
    /// called exactly once before any other write call.
    pub fn init(env: Env, officer: Address) {
        if env.storage().instance().has(&DataKey::Officer) {
            panic!("Contract already initialized");
        }
        officer.require_auth();
        env.storage().instance().set(&DataKey::Officer, &officer);
        env.storage().instance().set(&DataKey::AppCounter, &0u64);
    }

    // -----------------------------------------------------------------------
    /// Citizen files a renewal application that references the hash of
    /// their currently valid (not yet expired) passport. The application is
    /// stored with status `Pending` and is the only state mutation needed
    /// for a fresh request. Returns the application id.
    pub fn apply_renewal(
        env: Env,
        citizen: Address,
        application_id: u64,
        old_passport_hash: BytesN<32>,
    ) -> u64 {
        citizen.require_auth();

        if env
            .storage()
            .instance()
            .has(&DataKey::Expired(old_passport_hash.clone()))
        {
            panic!("Old passport is already expired");
        }
        if env
            .storage()
            .instance()
            .has(&DataKey::Application(application_id))
        {
            panic!("Application ID already exists");
        }

        let now = env.ledger().timestamp();
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        let app = Application {
            id: application_id,
            citizen: citizen.clone(),
            old_passport_hash: old_passport_hash.clone(),
            new_passport_hash: zero_hash,
            status: STATUS_PENDING,
            valid_until: 0,
            reason: Symbol::new(&env, "none"),
            officer: citizen,
            created_at: now,
            reviewed_at: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::Application(application_id), &app);

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AppCounter)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::AppCounter, &(counter + 1));

        application_id
    }

    // -----------------------------------------------------------------------
    /// Officer approves a pending application, records the hash of the new
    /// passport and its validity end timestamp. The old passport hash is
    /// permanently marked as expired so it cannot be used for further
    /// renewals.
    pub fn approve(
        env: Env,
        officer: Address,
        application_id: u64,
        new_passport_hash: BytesN<32>,
        valid_until: u64,
    ) {
        officer.require_auth();
        Self::verify_officer(&env, &officer);

        let mut app: Application = env
            .storage()
            .instance()
            .get(&DataKey::Application(application_id))
            .expect("Application not found");

        if app.status != STATUS_PENDING {
            panic!("Application is not pending");
        }
        if valid_until <= env.ledger().timestamp() {
            panic!("valid_until must be in the future");
        }
        if env
            .storage()
            .instance()
            .has(&DataKey::Issued(new_passport_hash.clone()))
        {
            panic!("New passport hash already issued");
        }

        let now = env.ledger().timestamp();
        app.new_passport_hash = new_passport_hash.clone();
        app.status = STATUS_APPROVED;
        app.valid_until = valid_until;
        app.officer = officer;
        app.reviewed_at = now;

        env.storage()
            .instance()
            .set(&DataKey::Application(application_id), &app);
        env.storage()
            .instance()
            .set(&DataKey::Expired(app.old_passport_hash.clone()), &true);
        env.storage()
            .instance()
            .set(&DataKey::Issued(new_passport_hash), &application_id);
    }

    // -----------------------------------------------------------------------
    /// Officer rejects a pending application with a short human-readable
    /// reason. The application is closed and cannot be re-opened; the
    /// citizen may file a new one if they remain eligible.
    pub fn reject(env: Env, officer: Address, application_id: u64, reason: Symbol) {
        officer.require_auth();
        Self::verify_officer(&env, &officer);

        let mut app: Application = env
            .storage()
            .instance()
            .get(&DataKey::Application(application_id))
            .expect("Application not found");

        if app.status != STATUS_PENDING {
            panic!("Application is not pending");
        }

        let now = env.ledger().timestamp();
        app.status = STATUS_REJECTED;
        app.reason = reason;
        app.officer = officer;
        app.reviewed_at = now;

        env.storage()
            .instance()
            .set(&DataKey::Application(application_id), &app);
    }

    // -----------------------------------------------------------------------
    /// Citizen cancels their own pending application with a reason. Only
    /// the original applicant can cancel, and only while the application
    /// is still in the `Pending` state.
    pub fn cancel(env: Env, citizen: Address, application_id: u64, reason: Symbol) {
        citizen.require_auth();

        let mut app: Application = env
            .storage()
            .instance()
            .get(&DataKey::Application(application_id))
            .expect("Application not found");

        if app.citizen != citizen {
            panic!("Only the applicant can cancel this application");
        }
        if app.status != STATUS_PENDING {
            panic!("Only pending applications can be cancelled");
        }

        let now = env.ledger().timestamp();
        app.status = STATUS_CANCELLED;
        app.reason = reason;
        app.reviewed_at = now;

        env.storage()
            .instance()
            .set(&DataKey::Application(application_id), &app);
    }

    // -----------------------------------------------------------------------
    /// Return the current status code of an application:
    /// 0 = Pending, 1 = Approved, 2 = Rejected, 3 = Cancelled.
    pub fn get_status(env: Env, application_id: u64) -> u32 {
        let app: Application = env
            .storage()
            .instance()
            .get(&DataKey::Application(application_id))
            .expect("Application not found");
        app.status
    }

    // -----------------------------------------------------------------------
    /// Returns `true` if the given old passport hash has been marked
    /// expired as part of a successful renewal approval.
    pub fn is_expired(env: Env, old_passport_hash: BytesN<32>) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Expired(old_passport_hash))
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    /// Return the full application record for a given id.
    pub fn get_application(env: Env, application_id: u64) -> Application {
        env.storage()
            .instance()
            .get(&DataKey::Application(application_id))
            .expect("Application not found")
    }

    // -----------------------------------------------------------------------
    /// Return the number of applications filed so far.
    pub fn get_application_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AppCounter)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helper: enforce that the caller is the registered officer.
    // -----------------------------------------------------------------------
    fn verify_officer(env: &Env, officer: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Officer)
            .expect("Contract not initialized");
        if &stored != officer {
            panic!("Caller is not the registered officer");
        }
    }
}
