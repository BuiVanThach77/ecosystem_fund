#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

/// Status code for a grant. `Active` means the grant is in progress,
/// `Completed` means every milestone has been verified, and `ClawedBack`
/// means the council recalled unspent funds.
const STATUS_ACTIVE: u32 = 0;
const STATUS_COMPLETED: u32 = 1;
const STATUS_CLAWED_BACK: u32 = 2;

/// Storage keys used by the contract. A contract type is the recommended
/// way to namespace persistent state on Soroban.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Address of the council (admin) that controls the fund.
    Admin,
    /// Whether `init` has been called. Prevents re-initialization.
    Initialized,
    /// Aggregate amount currently sitting in the pledge pool and any
    /// unspent, clawed-back capital.
    TotalPool,
    /// Total amount pledged by a single backer.
    BackerPledge(Address),
    /// Per-grant record, keyed by the grant id.
    Grant(u32),
}

/// On-chain record describing a single grant awarded by the council.
#[derive(Clone)]
#[contracttype]
pub struct Grant {
    /// Project team that will execute the work.
    pub grantee: Address,
    /// Total capital committed to this grant.
    pub total_amount: i128,
    /// Number of ordered milestones the grantee must complete.
    pub milestones: u32,
    /// Highest milestone id the grantee has reported as done.
    pub completed_milestones: u32,
    /// Highest milestone id the council has verified as done.
    pub verified_milestones: u32,
    /// `STATUS_ACTIVE`, `STATUS_COMPLETED`, or `STATUS_CLAWED_BACK`.
    pub status: u32,
    /// Short human-readable reason set by the council (e.g. on clawback).
    pub reason: Symbol,
}

#[contract]
pub struct EcosystemFund;

#[contractimpl]
impl EcosystemFund {
    /// Initialize the contract and set the council (admin) address.
    ///
    /// Must be called exactly once after deployment. The caller must
    /// authorize this call so the deploying wallet becomes the admin.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Contract already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::TotalPool, &0_i128);
    }

    /// Pledge `amount` into the grant pool.
    ///
    /// Any address may become a backer. The pledge is added to both the
    /// caller's running total and the global pool balance. The contract
    /// does not move real XLM; it tracks balances internally.
    pub fn pledge(env: Env, backer: Address, amount: i128) {
        if amount <= 0 {
            panic!("Pledge amount must be positive");
        }
        backer.require_auth();

        let key = DataKey::BackerPledge(backer.clone());
        let current: i128 = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(0_i128);
        env.storage()
            .instance()
            .set(&key, &(current + amount));

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPool)
            .unwrap_or(0_i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPool, &(total + amount));
    }

    /// Award a new grant to `grantee` for `total_amount` split across
    /// `milestones` ordered checkpoints.
    ///
    /// Only the council (admin) can call this. The grant amount is
    /// reserved out of the pool immediately so the pool balance always
    /// reflects free capital.
    pub fn award_grant(
        env: Env,
        admin: Address,
        grant_id: u32,
        grantee: Address,
        total_amount: i128,
        milestones: u32,
    ) {
        Self::require_admin(&env, &admin);

        if total_amount <= 0 {
            panic!("Grant amount must be positive");
        }
        if milestones == 0 {
            panic!("Milestones must be greater than zero");
        }
        if env.storage().instance().has(&DataKey::Grant(grant_id)) {
            panic!("Grant id already used");
        }

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPool)
            .unwrap_or(0_i128);
        if total_amount > total {
            panic!("Insufficient pool funds for this grant");
        }

        let grant = Grant {
            grantee,
            total_amount,
            milestones,
            completed_milestones: 0,
            verified_milestones: 0,
            status: STATUS_ACTIVE,
            reason: Symbol::new(&env, "awarded"),
        };
        env.storage()
            .instance()
            .set(&DataKey::Grant(grant_id), &grant);
        env.storage()
            .instance()
            .set(&DataKey::TotalPool, &(total - total_amount));
    }

    /// Grantee reports that `milestone_id` is done.
    ///
    /// Milestones must be reported strictly in order. A grantee can
    /// always re-report a later milestone as long as the previous one
    /// has already been completed. The council still has to verify
    /// the milestone before funds are considered released.
    pub fn mark_milestone_done(
        env: Env,
        grantee: Address,
        grant_id: u32,
        milestone_id: u32,
    ) {
        grantee.require_auth();

        let key = DataKey::Grant(grant_id);
        let mut grant: Grant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Grant not found");

        if grant.grantee != grantee {
            panic!("Only the grantee can mark milestones");
        }
        if grant.status != STATUS_ACTIVE {
            panic!("Grant is no longer active");
        }
        if milestone_id == 0 || milestone_id > grant.milestones {
            panic!("Invalid milestone id");
        }
        if milestone_id > grant.completed_milestones + 1 {
            panic!("Milestones must be reported in order");
        }
        if milestone_id <= grant.completed_milestones {
            panic!("Milestone already reported");
        }

        grant.completed_milestones = milestone_id;
        env.storage().instance().set(&key, &grant);
    }

    /// Council verifies a milestone that the grantee reported as done.
    ///
    /// Verification must also happen in order. Once every milestone of a
    /// grant is verified, the grant is automatically marked as completed.
    pub fn verify_milestone(
        env: Env,
        admin: Address,
        grant_id: u32,
        milestone_id: u32,
    ) {
        Self::require_admin(&env, &admin);

        let key = DataKey::Grant(grant_id);
        let mut grant: Grant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Grant not found");

        if grant.status != STATUS_ACTIVE {
            panic!("Grant is no longer active");
        }
        if milestone_id == 0 || milestone_id > grant.milestones {
            panic!("Invalid milestone id");
        }
        if milestone_id > grant.completed_milestones {
            panic!("Milestone has not been reported yet");
        }
        if milestone_id <= grant.verified_milestones {
            panic!("Milestone already verified");
        }

        grant.verified_milestones = milestone_id;
        if grant.verified_milestones == grant.milestones {
            grant.status = STATUS_COMPLETED;
            grant.reason = Symbol::new(&env, "completed");
        }
        env.storage().instance().set(&key, &grant);
    }

    /// Council claws back the unspent portion of a grant.
    ///
    /// Useful when the grantee misses deadlines or stops reporting
    /// progress. The unspent capital — based on the number of verified
    /// milestones — is returned to the pool so it can be re-awarded.
    pub fn clawback(env: Env, admin: Address, grant_id: u32, reason: Symbol) {
        Self::require_admin(&env, &admin);

        let key = DataKey::Grant(grant_id);
        let mut grant: Grant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Grant not found");

        if grant.status == STATUS_COMPLETED {
            panic!("Grant already completed");
        }
        if grant.status == STATUS_CLAWED_BACK {
            panic!("Grant already clawed back");
        }

        let verified_value: i128 = grant.total_amount
            * (grant.verified_milestones as i128)
            / (grant.milestones as i128);
        let unspent: i128 = grant.total_amount - verified_value;

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPool)
            .unwrap_or(0_i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPool, &(total + unspent));

        grant.status = STATUS_CLAWED_BACK;
        grant.reason = reason;
        env.storage().instance().set(&key, &grant);
    }

    /// Read the current status of a grant.
    ///
    /// Returns `0` for active, `1` for completed, and `2` for clawed back.
    /// Panics if the grant id has never been awarded.
    pub fn get_grant_status(env: Env, grant_id: u32) -> u32 {
        let grant: Grant = env
            .storage()
            .instance()
            .get(&DataKey::Grant(grant_id))
            .expect("Grant not found");
        grant.status
    }

    /// Read the full grant record. Convenience accessor for off-chain
    /// dashboards and auditors.
    pub fn get_grant(env: Env, grant_id: u32) -> Grant {
        env.storage()
            .instance()
            .get(&DataKey::Grant(grant_id))
            .expect("Grant not found")
    }

    /// Read the current free balance of the pledge pool (includes
    /// capital that has been clawed back from incomplete grants).
    pub fn get_pool_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalPool)
            .unwrap_or(0_i128)
    }

    // --- internal helpers ------------------------------------------------

    /// Verify that `admin` is the stored council address and that they
    /// authorized this transaction.
    fn require_admin(env: &Env, admin: &Address) {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");
        if stored_admin != *admin {
            panic!("Caller is not the council admin");
        }
        admin.require_auth();
    }
}
