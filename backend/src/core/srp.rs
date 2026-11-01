//! Who has paid SRP, worked out from the SRP corp's wallet journal and contracts.
//!
//! A character is covered for a waitlist's `coverage_hours` after sending its `min_amount` or more
//! in its `division` to the waitlist's SRP corp (a player donation), or after accepting a ship loan
//! contract the corp issued for that much. Only the character who paid is covered, and payments
//! don't stack: the latest one counts. The rule belongs to the waitlist (table srp_waitlist); the
//! defaults are only what a new waitlist starts with.

use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use serde::{Deserialize, Serialize};

use crate::core::auth::AuthenticatedAccount;
use crate::core::esi::{ESIScope, SEAT_LOGIN_SCOPES};

/// What the wallet character needs
pub const WALLET_SCOPES: &[ESIScope] = &[
    ESIScope::Wallet_ReadCorporationWallets_v1,
    ESIScope::Contracts_ReadCorporationContracts_v1,
];

/// The SeAT login that connects the wallet character: the scopes every SeAT login needs, plus the
/// wallet's
pub fn connect_scopes() -> Vec<ESIScope> {
    SEAT_LOGIN_SCOPES
        .iter()
        .chain(WALLET_SCOPES)
        .copied()
        .collect()
}

/// Results older than this are not shown: better nothing than a stale "not paid"
const STALE_AFTER_SECS: i64 = 2 * 3600;

/// How long the admin page shows why a connect failed
const CONNECT_ERROR_SHOWN_SECS: i64 = 15 * 60;

pub const CONNECT_STATE_PREFIX: &[u8] = b"srpconnect:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Via {
    Donation,
    Contract,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payment {
    pub paid_at: i64,
    pub amount: f64,
    pub via: Via,
}

/// What a new waitlist starts with, and what the migration gave the existing ones. Each waitlist
/// stores its own copy on the SRP wallet page; nothing falls back to these when the check reads.
pub const DEFAULT_DIVISION: i32 = 1;
pub const DEFAULT_MIN_AMOUNT: i64 = 50_000_000;
pub const DEFAULT_COVERAGE_HOURS: i64 = 24;

/// EVE corporations have seven wallet divisions
pub const MAX_DIVISION: i32 = 7;
/// Contracts are kept for 30 days, so a longer window pages for nothing
pub const MAX_COVERAGE_HOURS: i64 = 24 * 30;

/// The rule a waitlist's checks apply: payments of `min_amount` or more, in `division`, cover the
/// payer for `coverage_secs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    pub division: i32,
    pub min_amount: f64,
    pub coverage_secs: i64,
}

impl Default for Rule {
    fn default() -> Rule {
        Rule::new(DEFAULT_DIVISION, DEFAULT_MIN_AMOUNT, DEFAULT_COVERAGE_HOURS)
    }
}

impl Rule {
    pub fn new(division: i32, min_amount: i64, coverage_hours: i64) -> Rule {
        Rule {
            division,
            min_amount: min_amount as f64,
            coverage_secs: coverage_hours * 3600,
        }
    }

    /// As stored, without the loss the f64 amount would introduce
    pub fn min_amount_isk(&self) -> i64 {
        self.min_amount as i64
    }

    pub fn coverage_hours(&self) -> i64 {
        self.coverage_secs / 3600
    }

    /// The rule that counts everything either one does. Several waitlists can share one corp's
    /// division: its result is read once with this, and each waitlist then applies its own.
    pub fn loosest(self, other: Rule) -> Rule {
        Rule {
            division: self.division,
            min_amount: self.min_amount.min(other.min_amount),
            coverage_secs: self.coverage_secs.max(other.coverage_secs),
        }
    }

    pub fn expires_at(&self, paid_at: i64) -> i64 {
        paid_at + self.coverage_secs
    }

    pub fn covers(&self, paid_at: i64, now: i64) -> bool {
        self.expires_at(paid_at) > now
    }
}

/// Why a division, minimum amount or coverage is out of range, in words an admin can act on
pub fn validate_rule(division: i32, min_amount: i64, coverage_hours: i64) -> Result<(), String> {
    if !(1..=MAX_DIVISION).contains(&division) {
        return Err(format!(
            "The wallet division must be between 1 and {}.",
            MAX_DIVISION
        ));
    }
    if min_amount < 0 {
        return Err("The minimum amount can't be negative.".to_string());
    }
    if !(1..=MAX_COVERAGE_HOURS).contains(&coverage_hours) {
        return Err(format!(
            "Coverage must be between 1 and {} hours.",
            MAX_COVERAGE_HOURS
        ));
    }
    Ok(())
}

/// The fields we use from ESI's corporation wallet journal
#[derive(Debug, Deserialize)]
pub struct JournalEntry {
    pub date: String,
    pub ref_type: String,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default)]
    pub first_party_id: Option<i64>,
}

/// The fields we use from ESI's corporation contracts
#[derive(Debug, Deserialize)]
pub struct Contract {
    #[serde(rename = "type")]
    pub kind: String,
    pub status: String,
    pub issuer_corporation_id: i64,
    pub for_corporation: bool,
    pub acceptor_id: i64,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub date_accepted: Option<String>,
}

fn timestamp(date: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(date)
        .ok()
        .map(|d| d.timestamp())
}

fn insert_latest(paid: &mut HashMap<i64, Payment>, character_id: i64, payment: Payment) {
    match paid.get(&character_id) {
        Some(existing) if existing.paid_at >= payment.paid_at => {}
        _ => {
            paid.insert(character_id, payment);
        }
    }
}

/// Donations to the corp. `first_party_id` is the sender for player_donation entries.
pub fn paid_from_journal(entries: &[JournalEntry], now: i64, rule: &Rule) -> HashMap<i64, Payment> {
    let mut paid = HashMap::new();
    for entry in entries {
        if entry.ref_type != "player_donation" {
            continue;
        }
        let (amount, character_id, paid_at) =
            match (entry.amount, entry.first_party_id, timestamp(&entry.date)) {
                (Some(a), Some(c), Some(t)) => (a, c, t),
                _ => continue,
            };
        if amount >= rule.min_amount && rule.covers(paid_at, now) {
            insert_latest(
                &mut paid,
                character_id,
                Payment {
                    paid_at,
                    amount,
                    via: Via::Donation,
                },
            );
        }
    }
    paid
}

/// Ship loan contracts: item exchanges issued on behalf of the SRP corp, covering the acceptor.
/// Read from the contract list rather than the journal's contract_price entries, whose party ids
/// ESI documents as inconsistent between ref types.
pub fn paid_from_contracts(
    contracts: &[Contract],
    corporation_id: i64,
    now: i64,
    rule: &Rule,
) -> HashMap<i64, Payment> {
    let mut paid = HashMap::new();
    for contract in contracts {
        if contract.kind != "item_exchange"
            || contract.status != "finished"
            || contract.issuer_corporation_id != corporation_id
            || !contract.for_corporation
            || contract.acceptor_id == 0
        {
            continue;
        }
        let (amount, paid_at) = match (
            contract.price,
            contract.date_accepted.as_deref().and_then(timestamp),
        ) {
            (Some(a), Some(t)) => (a, t),
            _ => continue,
        };
        if amount >= rule.min_amount && rule.covers(paid_at, now) {
            insert_latest(
                &mut paid,
                contract.acceptor_id,
                Payment {
                    paid_at,
                    amount,
                    via: Via::Contract,
                },
            );
        }
    }
    paid
}

/// The payment covering `character_id` now, if any
pub fn current_payment(
    paid: &HashMap<i64, Payment>,
    character_id: i64,
    now: i64,
    rule: &Rule,
) -> Option<Payment> {
    // The stored entries were filtered by the loosest rule of every waitlist sharing the division;
    // this waitlist's own amount and window decide here
    paid.get(&character_id)
        .filter(|p| p.amount >= rule.min_amount && rule.covers(p.paid_at, now))
        .copied()
}

/// Latest payment per character wins
pub fn merge(mut a: HashMap<i64, Payment>, b: HashMap<i64, Payment>) -> HashMap<i64, Payment> {
    for (character_id, payment) in b {
        insert_latest(&mut a, character_id, payment);
    }
    a
}

/// Journal pages are newest first: once a page ends before the coverage window, later pages
/// can't matter
pub fn journal_page_reaches_past_window(entries: &[JournalEntry], now: i64, rule: &Rule) -> bool {
    match entries.last().and_then(|e| timestamp(&e.date)) {
        Some(t) => !rule.covers(t, now),
        None => true,
    }
}

/// One successful read of a corp's wallet
pub struct WalletRead {
    pub paid: HashMap<i64, Payment>,
    /// When ESI produced the data, if it said: the older of the journal's and the contracts'.
    /// A payment made after this can't show yet.
    pub data_as_of: Option<i64>,
}

/// The older of two times. Unknown if either is: the known one could be the fresher source, and
/// claiming fresher data than we have would show "No SRP" where it should show "Checking".
pub fn older(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        _ => None,
    }
}

/// One SRP corp's wallet division and its check results
#[derive(Debug, Default)]
pub struct CorpState {
    pub paid: HashMap<i64, Payment>,
    /// When the wallet was last read successfully
    pub checked_at: Option<i64>,
    /// When ESI produced that data; see `WalletRead`
    pub data_as_of: Option<i64>,
    /// Why the last check failed, if it did, in words an admin can act on
    pub last_error: Option<String>,
}

/// What `CorpState::apply` did with a check's result, for the log
#[derive(Debug, PartialEq)]
pub enum Applied {
    Updated,
    /// Updated after the previous check had failed
    Recovered,
    Failed,
    /// The wallet changed while the check ran
    Discarded,
}

impl CorpState {
    pub fn is_fresh(&self, now: i64) -> bool {
        match self.checked_at {
            Some(t) => now - t < STALE_AFTER_SECS,
            None => false,
        }
    }

    /// Records a check's result. A failure keeps the last good result, which stops being shown
    /// once it goes stale. Whether the wallet changed meanwhile is `SrpState::apply`'s call: the
    /// generation lives per corp, not per division.
    pub fn apply(&mut self, result: Result<WalletRead, String>, now: i64) -> Applied {
        match result {
            Ok(read) => {
                let recovered = self.last_error.is_some();
                self.paid = read.paid;
                self.data_as_of = read.data_as_of;
                self.checked_at = Some(now);
                self.last_error = None;
                match recovered {
                    true => Applied::Recovered,
                    false => Applied::Updated,
                }
            }
            Err(message) => {
                self.last_error = Some(message);
                Applied::Failed
            }
        }
    }

    /// Payments still covering someone, newest first
    pub fn recent(&self, now: i64, rule: &Rule) -> Vec<(i64, Payment)> {
        let mut recent: Vec<(i64, Payment)> = self
            .paid
            .iter()
            .filter(|(_, p)| p.amount >= rule.min_amount && rule.covers(p.paid_at, now))
            .map(|(&id, &p)| (id, p))
            .collect();
        recent.sort_by_key(|(_, p)| std::cmp::Reverse(p.paid_at));
        recent
    }
}

#[derive(Debug, Default)]
pub struct SrpState {
    /// By SRP corp id and wallet division: each division has its own journal, and waitlists of
    /// one corp can use different ones
    corps: HashMap<(i64, i32), CorpState>,
    /// Bumped per corp whenever a wallet of it is connected, reconnected or disconnected, so a
    /// check that started before then throws its result away instead of publishing the old
    /// wallet's
    generations: HashMap<i64, u64>,
    /// When and why the last connect failed. Kept here rather than in the redirect URL, so a
    /// link can't put words on the admin page.
    connect_error: Option<ConnectError>,
}

#[derive(Debug)]
struct ConnectError {
    at: i64,
    message: String,
}

impl SrpState {
    pub fn corp(&self, corporation_id: i64, division: i32) -> Option<&CorpState> {
        self.corps.get(&(corporation_id, division))
    }

    pub fn corp_mut(&mut self, corporation_id: i64, division: i32) -> &mut CorpState {
        self.corps.entry((corporation_id, division)).or_default()
    }

    /// To pass to `apply` when the check finishes
    pub fn generation(&self, corporation_id: i64) -> u64 {
        self.generations.get(&corporation_id).copied().unwrap_or(0)
    }

    fn bump(&mut self, corporation_id: i64) -> u64 {
        let generation = self.generation(corporation_id) + 1;
        self.generations.insert(corporation_id, generation);
        generation
    }

    fn divisions_of(&mut self, corporation_id: i64) -> impl Iterator<Item = &mut CorpState> {
        self.corps
            .iter_mut()
            .filter(move |((corp, _), _)| *corp == corporation_id)
            .map(|(_, corp)| corp)
    }

    /// Records a check's result against the generation it started with. The result is discarded
    /// when the corp's wallet changed meanwhile: it is about the old one.
    pub fn apply(
        &mut self,
        corporation_id: i64,
        division: i32,
        generation: u64,
        result: Result<WalletRead, String>,
        now: i64,
    ) -> Applied {
        if generation != self.generation(corporation_id) {
            return Applied::Discarded;
        }
        self.corp_mut(corporation_id, division).apply(result, now)
    }

    /// The corp's wallet was connected or reconnected. Its last good results stay until the new
    /// wallet's first check replaces them, so a reconnect that turns out wrong doesn't blank
    /// pilots' status.
    pub fn wallet_replaced(&mut self, corporation_id: i64) {
        self.bump(corporation_id);
        for corp in self.divisions_of(corporation_id) {
            corp.last_error = None;
        }
        self.connect_error = None;
    }

    /// The corp's wallet was disconnected: what was read through it goes
    pub fn wallet_removed(&mut self, corporation_id: i64) {
        self.bump(corporation_id);
        for corp in self.divisions_of(corporation_id) {
            *corp = CorpState::default();
        }
    }

    pub fn connect_failed(&mut self, now: i64, message: String) {
        self.connect_error = Some(ConnectError { at: now, message });
    }

    pub fn recent_connect_error(&self, now: i64) -> Option<String> {
        match &self.connect_error {
            Some(e) if now - e.at < CONNECT_ERROR_SHOWN_SECS => Some(e.message.clone()),
            _ => None,
        }
    }
}

/// The SRP check's state, shared between the updater and the routes, and a way to ask the updater
/// for a check now
#[derive(Debug, Default)]
pub struct Shared {
    state: RwLock<SrpState>,
    wake: tokio::sync::Notify,
}

impl Shared {
    // Every write is a few assignments, so a panic elsewhere while the lock was held can't have
    // left the state half-written: carry on rather than fail every later request
    pub fn read(&self) -> RwLockReadGuard<'_, SrpState> {
        self.state.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, SrpState> {
        self.state.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Wakes the updater, or makes its next wait return at once if it's mid-check
    pub fn check_now(&self) {
        self.wake.notify_one();
    }

    /// Until `check_now` or `max` passes
    pub async fn wait(&self, max: std::time::Duration) {
        let _ = tokio::time::timeout(max, self.wake.notified()).await;
    }
}

pub type SharedSrpState = Arc<Shared>;

pub fn new_shared_state() -> SharedSrpState {
    Arc::new(Shared::default())
}

/// For an admin: the detail goes to the log, not the page
pub fn database_error(error: sqlx::Error) -> String {
    error!("SRP database error: {}", error);
    "Database error; the server log has the details.".to_string()
}

/// Whether the account may run the SRP wallet page. The `srp-manage` key comes with the fc role
/// and everything above it.
pub fn is_manager(account: &AuthenticatedAccount) -> bool {
    account.access.contains("srp-manage")
}

/// How old a connect login may be when it comes back. The OAuth round trip takes seconds; this
/// keeps a leaked or replayed URL short-lived.
pub const CONNECT_STATE_MAX_AGE_SECS: u32 = 10 * 60;
/// A minute of slack for clocks that are slightly apart
const CONNECT_STATE_FUTURE_SLACK_SECS: i64 = 60;

/// What a connect login carries. It is only ever seen inside the Branca-encrypted OAuth state
/// (`routes::auth::authorize_url`), so the ids don't appear in the URL, and it is bound to the
/// account that started it.
#[derive(Debug, PartialEq)]
pub struct ConnectState {
    /// 0 for a login started before the state carried it; such a login isn't account-bound
    pub account_id: i64,
    pub character_id: i64,
    pub waitlist_id: Option<i64>,
    /// None for a login started before the state carried it
    pub issued_at: Option<i64>,
}

/// OAuth state marking a login as "connect this character as the SRP wallet for this account",
/// optionally for one waitlist
pub fn connect_state(account_id: i64, character_id: i64, waitlist_id: Option<i64>) -> Vec<u8> {
    let waitlist = waitlist_id.unwrap_or(0);
    let issued_at = chrono::Utc::now().timestamp();
    format!(
        "srpconnect:{}:{}:{}:{}",
        account_id, character_id, waitlist, issued_at
    )
    .into_bytes()
}

pub fn parse_connect_state(state: &[u8]) -> Option<ConnectState> {
    let rest = state.strip_prefix(CONNECT_STATE_PREFIX)?;
    let parts: Vec<&str> = std::str::from_utf8(rest).ok()?.split(':').collect();
    let (account_id, character_id, waitlist_id, issued_at) = match parts.as_slice() {
        // Started before the account, waitlist and time were carried
        [character_id] => (0, character_id.parse().ok()?, None, None),
        [account_id, character_id, waitlist_id, issued_at] => {
            let waitlist_id = match *waitlist_id {
                "0" => None,
                id => Some(id.parse().ok()?),
            };
            let issued_at: i64 = issued_at.parse().ok()?;
            let now = chrono::Utc::now().timestamp();
            if now - issued_at > CONNECT_STATE_MAX_AGE_SECS as i64
                || issued_at - now > CONNECT_STATE_FUTURE_SLACK_SECS
            {
                return None;
            }
            (
                account_id.parse().ok()?,
                character_id.parse().ok()?,
                waitlist_id,
                Some(issued_at),
            )
        }
        _ => return None,
    };
    Some(ConnectState {
        account_id,
        character_id,
        waitlist_id,
        issued_at,
    })
}

/// The SRP wallets (table srp_connection), one per wallet character: the SeAT login of a character
/// in an SRP corp. Keyed by character rather than corp, so two waitlists can each have their own
/// account even when both accounts are in the same corp. Runtime-checked queries, like
/// routes/statistics.rs, so sqlx-data.json needs no regeneration.
pub mod connection {
    // Not Debug: it holds the refresh token, which must never reach a log
    #[derive(sqlx::FromRow)]
    pub struct Connection {
        pub character_id: i64,
        pub corporation_id: i64,
        pub corporation_name: String,
        pub refresh_token: String,
        pub connected_by: i64,
        pub connected_at: i64,
    }

    /// By corp name, then character id
    pub async fn all(db: &crate::DB) -> Result<Vec<Connection>, sqlx::Error> {
        sqlx::query_as::<_, Connection>(
            "SELECT character_id, corporation_id, corporation_name, refresh_token, connected_by,
                connected_at FROM srp_connection ORDER BY corporation_name, character_id",
        )
        .fetch_all(db)
        .await
    }

    pub async fn get(db: &crate::DB, character_id: i64) -> Result<Option<Connection>, sqlx::Error> {
        sqlx::query_as::<_, Connection>(
            "SELECT character_id, corporation_id, corporation_name, refresh_token, connected_by,
                connected_at FROM srp_connection WHERE character_id = $1",
        )
        .bind(character_id)
        .fetch_optional(db)
        .await
    }

    /// The corp's wallets, the most recently connected first: a check tries them in this order
    /// until one works
    pub async fn for_corporation(
        db: &crate::DB,
        corporation_id: i64,
    ) -> Result<Vec<Connection>, sqlx::Error> {
        sqlx::query_as::<_, Connection>(
            "SELECT character_id, corporation_id, corporation_name, refresh_token, connected_by,
                connected_at FROM srp_connection WHERE corporation_id = $1
                ORDER BY connected_at DESC, character_id",
        )
        .bind(corporation_id)
        .fetch_all(db)
        .await
    }

    pub async fn exists(db: &crate::DB, character_id: i64) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM srp_connection WHERE character_id = $1)")
            .bind(character_id)
            .fetch_one(db)
            .await
    }

    /// Whether any account is still connected for the corp
    pub async fn corporation_connected(
        db: &crate::DB,
        corporation_id: i64,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM srp_connection WHERE corporation_id = $1)")
            .bind(corporation_id)
            .fetch_one(db)
            .await
    }

    /// Replaces the character's previous login, if any
    pub async fn save(db: &crate::DB, connection: &Connection) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO srp_connection (character_id, corporation_id, corporation_name,
                refresh_token, connected_by, connected_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (character_id) DO UPDATE SET corporation_id = $2, corporation_name = $3,
                refresh_token = $4, connected_by = $5, connected_at = $6",
        )
        .bind(connection.character_id)
        .bind(connection.corporation_id)
        .bind(&connection.corporation_name)
        .bind(&connection.refresh_token)
        .bind(connection.connected_by)
        .bind(connection.connected_at)
        .execute(db)
        .await?;
        Ok(())
    }

    /// Replaces the refresh token only if it is still `old`: if the wallet was reconnected while
    /// a check was using the old token, the new connection's token must not be overwritten.
    /// Returns whether it was replaced.
    pub async fn update_refresh_token(
        db: &crate::DB,
        character_id: i64,
        old: &str,
        new: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE srp_connection SET refresh_token = $3
                WHERE character_id = $1 AND refresh_token = $2",
        )
        .bind(character_id)
        .bind(old)
        .bind(new)
        .execute(db)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Deletes the connection only while it is still the one read (`connected_at`): a connect or
    /// reconnect since then must not be deleted unseen. Returns whether it was deleted.
    pub async fn delete(
        db: &crate::DB,
        character_id: i64,
        connected_at: i64,
    ) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM srp_connection WHERE character_id = $1 AND connected_at = $2")
                .bind(character_id)
                .bind(connected_at)
                .execute(db)
                .await?;
        Ok(result.rows_affected() == 1)
    }
}

/// Which wallet account pays SRP for each waitlist (table srp_waitlist) and the rule that waitlist
/// applies, both set on the SRP wallet page. A waitlist has a row once either is set; no SRP means
/// no wallet account and no corp. Both the wallet and the rule are kept when the wallet is
/// disconnected (the waitlist then just shows nothing), so reconnecting doesn't lose them.
pub mod waitlists {
    use super::{Rule, DEFAULT_COVERAGE_HOURS, DEFAULT_DIVISION, DEFAULT_MIN_AMOUNT};

    #[derive(sqlx::FromRow)]
    pub struct Waitlist {
        pub id: i64,
        pub name: String,
        pub character_id: Option<i64>,
        /// The corp a migrated waitlist pointed at while it has no wallet account yet
        pub pending_corporation_id: Option<i64>,
        /// The rule it applies; defaults filled in when it has no row yet
        pub division: i32,
        pub min_amount: i64,
        pub coverage_hours: i64,
        /// Who set the wallet and when
        pub set_by: Option<i64>,
        pub set_at: Option<i64>,
    }

    impl Waitlist {
        pub fn rule(&self) -> Rule {
            Rule::new(self.division, self.min_amount, self.coverage_hours)
        }
    }

    /// What a waitlist's checks use: the corp that pays for its pilots and the rule it applies
    pub struct Target {
        /// None while the waitlist has no wallet account and no corp kept from the migration
        pub corporation_id: Option<i64>,
        pub rule: Rule,
    }

    /// One (corp, division) pair the updater must read, with the loosest rule of the waitlists
    /// that read it
    #[derive(Debug)]
    pub struct Check {
        pub corporation_id: i64,
        pub division: i32,
        pub rule: Rule,
    }

    #[derive(sqlx::FromRow)]
    struct CheckRow {
        corporation_id: i64,
        division: i32,
        min_amount: i64,
        coverage_hours: i64,
    }

    /// Every waitlist that isn't archived, with its SRP wallet and rule
    pub async fn list(db: &crate::DB) -> Result<Vec<Waitlist>, sqlx::Error> {
        sqlx::query_as::<_, Waitlist>(
            "SELECT w.id, w.name, s.character_id, s.corporation_id AS pending_corporation_id,
                COALESCE(s.division, $1) AS division,
                COALESCE(s.min_amount, $2) AS min_amount,
                COALESCE(s.coverage_hours, $3) AS coverage_hours,
                s.set_by, s.set_at
                FROM waitlist w
                LEFT JOIN srp_waitlist s ON s.waitlist_id = w.id
                WHERE NOT w.is_archived ORDER BY w.name",
        )
        .bind(DEFAULT_DIVISION)
        .bind(DEFAULT_MIN_AMOUNT)
        .bind(DEFAULT_COVERAGE_HOURS)
        .fetch_all(db)
        .await
    }

    /// The pairs to check: every waitlist's assigned account's corp, or the corp a migrated
    /// waitlist kept, at the division it uses. Shared pairs carry the loosest rule so one read
    /// covers all of them.
    pub async fn checks(db: &crate::DB) -> Result<Vec<Check>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CheckRow>(
            "SELECT corporation_id, division, MIN(min_amount) AS min_amount,
                MAX(coverage_hours)::int8 AS coverage_hours
                FROM (
                    SELECT c.corporation_id, s.division, s.min_amount, s.coverage_hours
                        FROM srp_waitlist s
                        JOIN srp_connection c ON c.character_id = s.character_id
                    UNION ALL
                    SELECT s.corporation_id, s.division, s.min_amount, s.coverage_hours
                        FROM srp_waitlist s
                        WHERE s.character_id IS NULL AND s.corporation_id IS NOT NULL
                ) pairs
                GROUP BY corporation_id, division
                ORDER BY corporation_id, division",
        )
        .fetch_all(db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| Check {
                corporation_id: row.corporation_id,
                division: row.division,
                rule: Rule::new(row.division, row.min_amount, row.coverage_hours),
            })
            .collect())
    }

    /// None when there's no such waitlist, or it's archived
    pub async fn active_name(
        db: &crate::DB,
        waitlist_id: i64,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT name FROM waitlist WHERE id = $1 AND NOT is_archived")
            .bind(waitlist_id)
            .fetch_optional(db)
            .await
    }

    /// What feeds the waitlist. An assigned account is the source: its corp, or nothing while its
    /// wallet is disconnected. Only a corp-only assignment (a migration leftover) resolves to the
    /// corp on its own.
    pub async fn target(db: &crate::DB, waitlist_id: i64) -> Result<Option<Target>, sqlx::Error> {
        // Option<Option>: no row, or a row that resolves to no corp. The scalar must be Option
        // or a NULL decode fails before the flatten.
        let row: Option<(i32, i64, i32, Option<Option<i64>>)> = sqlx::query_as(
            "SELECT s.division, s.min_amount, s.coverage_hours,
                    COALESCE(c.corporation_id,
                        CASE WHEN s.character_id IS NULL THEN s.corporation_id END)
                FROM srp_waitlist s
                LEFT JOIN srp_connection c ON c.character_id = s.character_id
                WHERE s.waitlist_id = $1",
        )
        .bind(waitlist_id)
        .fetch_optional(db)
        .await?;
        Ok(row.map(
            |(division, min_amount, coverage_hours, corporation)| Target {
                corporation_id: corporation.flatten(),
                rule: Rule::new(division, min_amount, coverage_hours as i64),
            },
        ))
    }

    /// The waitlist's current assignment: wallet character, and the corp it falls back to
    pub async fn assignment(
        db: &crate::DB,
        waitlist_id: i64,
    ) -> Result<(Option<i64>, Option<i64>), sqlx::Error> {
        Ok(sqlx::query_as(
            "SELECT character_id, corporation_id FROM srp_waitlist WHERE waitlist_id = $1",
        )
        .bind(waitlist_id)
        .fetch_optional(db)
        .await?
        .unwrap_or((None, None)))
    }

    /// Saves the waitlist's SRP wallet character, the corp to fall back to while it has none, and
    /// its rule when one is given; NULL character and corp mean no SRP. Creating the row this way
    /// fills the rule with the defaults. Returns whether anything changed.
    pub async fn set(
        db: &crate::DB,
        waitlist_id: i64,
        character_id: Option<i64>,
        corporation_id: Option<i64>,
        rule: Option<Rule>,
        by: i64,
        now: i64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO srp_waitlist (waitlist_id, character_id, corporation_id,
                    division, min_amount, coverage_hours, set_by, set_at)
                VALUES ($1, $2, $3, COALESCE($4, $9), COALESCE($5, $10),
                    COALESCE($6, $11), $7, $8)
                ON CONFLICT (waitlist_id) DO UPDATE
                SET character_id = $2, corporation_id = $3,
                    division = COALESCE($4, srp_waitlist.division),
                    min_amount = COALESCE($5, srp_waitlist.min_amount),
                    coverage_hours = COALESCE($6, srp_waitlist.coverage_hours),
                    set_by = $7, set_at = $8
                WHERE srp_waitlist.character_id IS DISTINCT FROM $2
                    OR srp_waitlist.corporation_id IS DISTINCT FROM $3
                    OR ($4 IS NOT NULL AND srp_waitlist.division IS DISTINCT FROM $4)
                    OR ($5 IS NOT NULL AND srp_waitlist.min_amount IS DISTINCT FROM $5)
                    OR ($6 IS NOT NULL AND srp_waitlist.coverage_hours IS DISTINCT FROM $6)",
        )
        .bind(waitlist_id)
        .bind(character_id)
        .bind(corporation_id)
        .bind(rule.map(|r| r.division))
        .bind(rule.map(|r| r.min_amount_isk()))
        .bind(rule.map(|r| r.coverage_hours()))
        .bind(by)
        .bind(now)
        .bind(DEFAULT_DIVISION)
        .bind(DEFAULT_MIN_AMOUNT)
        .bind(DEFAULT_COVERAGE_HOURS)
        .execute(db)
        .await?;
        Ok(result.rows_affected() == 1)
    }
}

#[cfg(test)]
impl From<HashMap<i64, Payment>> for WalletRead {
    fn from(paid: HashMap<i64, Payment>) -> WalletRead {
        WalletRead {
            paid,
            data_as_of: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_as_old_as_its_older_source() {
        assert_eq!(older(Some(100), Some(200)), Some(100));
        assert_eq!(older(None, Some(200)), None);
        assert_eq!(older(Some(100), None), None);
        assert_eq!(older(None, None), None);
    }

    const CORP: i64 = 98000001;
    // 2026-10-09T15:00:00Z
    const NOW: i64 = 1791558000;

    fn rule() -> Rule {
        Rule::default()
    }

    fn donation(date: &str, from: i64, amount: f64) -> JournalEntry {
        JournalEntry {
            date: date.to_string(),
            ref_type: "player_donation".to_string(),
            amount: Some(amount),
            first_party_id: Some(from),
        }
    }

    fn contract(accepted: &str, acceptor: i64, price: f64) -> Contract {
        Contract {
            kind: "item_exchange".to_string(),
            status: "finished".to_string(),
            issuer_corporation_id: CORP,
            for_corporation: true,
            acceptor_id: acceptor,
            price: Some(price),
            date_accepted: Some(accepted.to_string()),
        }
    }

    #[test]
    fn donation_of_min_amount_or_more_covers_the_sender() {
        let entries = vec![
            donation("2026-10-09T14:41:00Z", 1, 50_000_000.0),
            donation("2026-10-09T09:12:00Z", 2, 55_000_000.0),
            donation("2026-10-09T10:00:00Z", 3, 49_999_999.0),
        ];
        let paid = paid_from_journal(&entries, NOW, &rule());
        assert_eq!(paid.len(), 2);
        assert_eq!(paid[&1].via, Via::Donation);
        assert_eq!(paid[&2].amount, 55_000_000.0);
        assert!(!paid.contains_key(&3));
    }

    #[test]
    fn donation_older_than_coverage_is_ignored() {
        let entries = vec![donation("2026-10-08T14:59:00Z", 1, 50_000_000.0)];
        assert!(paid_from_journal(&entries, NOW, &rule()).is_empty());
    }

    #[test]
    fn cover_ends_exactly_after_coverage_hours() {
        // NOW is 15:00; a payment exactly 24 hours earlier no longer covers
        let at_the_limit = vec![donation("2026-10-08T15:00:00Z", 1, 50_000_000.0)];
        assert!(paid_from_journal(&at_the_limit, NOW, &rule()).is_empty());
        let just_inside = vec![donation("2026-10-08T15:00:01Z", 1, 50_000_000.0)];
        assert!(paid_from_journal(&just_inside, NOW, &rule()).contains_key(&1));
    }

    #[test]
    fn other_journal_entries_are_ignored() {
        let mut entry = donation("2026-10-09T14:00:00Z", 1, 50_000_000.0);
        entry.ref_type = "corporation_account_withdrawal".to_string();
        let mut no_sender = donation("2026-10-09T14:00:00Z", 2, 50_000_000.0);
        no_sender.first_party_id = None;
        assert!(paid_from_journal(&[entry, no_sender], NOW, &rule()).is_empty());
    }

    #[test]
    fn small_payments_do_not_add_up() {
        let entries = vec![
            donation("2026-10-09T14:00:00Z", 1, 25_000_000.0),
            donation("2026-10-09T14:05:00Z", 1, 25_000_000.0),
        ];
        assert!(paid_from_journal(&entries, NOW, &rule()).is_empty());
    }

    #[test]
    fn latest_payment_sets_the_window() {
        let entries = vec![
            donation("2026-10-09T08:00:00Z", 1, 50_000_000.0),
            donation("2026-10-09T14:00:00Z", 1, 60_000_000.0),
        ];
        let paid = paid_from_journal(&entries, NOW, &rule());
        assert_eq!(paid[&1].amount, 60_000_000.0);
        assert_eq!(paid[&1].paid_at, timestamp("2026-10-09T14:00:00Z").unwrap());
    }

    #[test]
    fn loan_contract_covers_the_acceptor() {
        let paid = paid_from_contracts(
            &[contract("2026-10-09T13:40:00Z", 7, 50_000_000.0)],
            CORP,
            NOW,
            &rule(),
        );
        assert_eq!(paid[&7].via, Via::Contract);
    }

    #[test]
    fn contracts_that_are_not_corp_loans_are_ignored() {
        let mut personal = contract("2026-10-09T13:40:00Z", 1, 50_000_000.0);
        personal.for_corporation = false;
        let mut other_corp = contract("2026-10-09T13:40:00Z", 2, 50_000_000.0);
        other_corp.issuer_corporation_id = 98000002;
        let mut outstanding = contract("2026-10-09T13:40:00Z", 3, 50_000_000.0);
        outstanding.status = "outstanding".to_string();
        let mut courier = contract("2026-10-09T13:40:00Z", 4, 50_000_000.0);
        courier.kind = "courier".to_string();
        let cheap = contract("2026-10-09T13:40:00Z", 5, 10_000_000.0);
        let old = contract("2026-10-08T13:40:00Z", 6, 50_000_000.0);
        let mut unaccepted = contract("2026-10-09T13:40:00Z", 0, 50_000_000.0);
        unaccepted.date_accepted = None;

        let contracts = [
            personal,
            other_corp,
            outstanding,
            courier,
            cheap,
            old,
            unaccepted,
        ];
        assert!(paid_from_contracts(&contracts, CORP, NOW, &rule()).is_empty());
    }

    #[test]
    fn outgoing_donations_are_ignored() {
        // The corp paying a pilot is a player_donation too, with a negative amount
        let entries = vec![donation("2026-10-09T14:00:00Z", CORP, -50_000_000.0)];
        assert!(paid_from_journal(&entries, NOW, &rule()).is_empty());
    }

    #[test]
    fn same_time_donation_and_contract_keep_the_donation() {
        let journal = paid_from_journal(
            &[donation("2026-10-09T13:00:00Z", 1, 50_000_000.0)],
            NOW,
            &rule(),
        );
        let contracts = paid_from_contracts(
            &[contract("2026-10-09T13:00:00Z", 1, 60_000_000.0)],
            CORP,
            NOW,
            &rule(),
        );
        assert_eq!(merge(journal, contracts)[&1].via, Via::Donation);
    }

    #[test]
    fn current_payment_only_while_covered() {
        let paid = paid_from_journal(
            &[donation("2026-10-09T13:00:00Z", 1, 50_000_000.0)],
            NOW,
            &rule(),
        );
        assert!(current_payment(&paid, 1, NOW, &rule()).is_some());
        assert!(current_payment(&paid, 2, NOW, &rule()).is_none());
        assert!(current_payment(&paid, 1, NOW + 24 * 3600, &rule()).is_none());
    }

    // The stored payments are filtered by the loosest rule of every waitlist on the division; each
    // waitlist then applies its own amount and window
    #[test]
    fn current_payment_applies_the_waitlists_own_amount() {
        let paid = paid_from_journal(
            &[donation("2026-10-09T13:00:00Z", 1, 60_000_000.0)],
            NOW,
            &rule(),
        );
        assert!(current_payment(&paid, 1, NOW, &Rule::new(1, 70_000_000, 24)).is_none());
        assert!(current_payment(&paid, 1, NOW, &Rule::new(1, 60_000_000, 24)).is_some());
    }

    #[test]
    fn recent_applies_the_rule_it_is_given() {
        let mut corp = CorpState::default();
        corp.paid = paid_from_journal(
            &[donation("2026-10-09T13:00:00Z", 1, 60_000_000.0)],
            NOW,
            &rule(),
        );
        assert_eq!(corp.recent(NOW, &Rule::new(1, 70_000_000, 24)).len(), 0);
        assert_eq!(corp.recent(NOW, &Rule::new(1, 60_000_000, 24)).len(), 1);
        assert_eq!(corp.recent(NOW, &Rule::new(1, 60_000_000, 1)).len(), 0);
    }

    #[test]
    fn rules_are_validated() {
        assert!(validate_rule(1, 0, 1).is_ok());
        assert!(validate_rule(MAX_DIVISION, i64::MAX, MAX_COVERAGE_HOURS).is_ok());
        assert!(validate_rule(0, 50_000_000, 24).is_err());
        assert!(validate_rule(MAX_DIVISION + 1, 50_000_000, 24).is_err());
        assert!(validate_rule(1, -1, 24).is_err());
        assert!(validate_rule(1, 50_000_000, 0).is_err());
        assert!(validate_rule(1, 50_000_000, MAX_COVERAGE_HOURS + 1).is_err());
    }

    #[test]
    fn the_loosest_rule_counts_everything_either_does() {
        let a = Rule::new(2, 100_000_000, 48);
        let b = Rule::new(2, 50_000_000, 24);
        let loosest = a.loosest(b);
        assert_eq!(loosest.division, 2);
        assert_eq!(loosest.min_amount_isk(), 50_000_000);
        assert_eq!(loosest.coverage_hours(), 48);
        // And the other way round
        assert_eq!(b.loosest(a), loosest);
    }

    #[test]
    fn merge_keeps_the_latest_payment_per_character() {
        let journal = paid_from_journal(
            &[donation("2026-10-09T09:00:00Z", 1, 50_000_000.0)],
            NOW,
            &rule(),
        );
        let contracts = paid_from_contracts(
            &[contract("2026-10-09T13:00:00Z", 1, 50_000_000.0)],
            CORP,
            NOW,
            &rule(),
        );
        let merged = merge(journal, contracts);
        assert_eq!(merged[&1].via, Via::Contract);
    }

    #[test]
    fn journal_paging_stops_past_the_window() {
        let recent = vec![donation("2026-10-09T10:00:00Z", 1, 1.0)];
        let reaching_back = vec![
            donation("2026-10-09T10:00:00Z", 1, 1.0),
            donation("2026-10-07T10:00:00Z", 1, 1.0),
        ];
        assert!(!journal_page_reaches_past_window(&recent, NOW, &rule()));
        assert!(journal_page_reaches_past_window(
            &reaching_back,
            NOW,
            &rule()
        ));
        assert!(journal_page_reaches_past_window(&[], NOW, &rule()));
    }

    // Shapes from ESI's published OpenAPI spec, with fields we don't use, and an entry missing the
    // optional ones
    #[test]
    fn parses_esi_responses() {
        let journal: Vec<JournalEntry> = serde_json::from_str(
            r#"[
                {"id": 1, "date": "2026-10-09T14:41:00Z", "ref_type": "player_donation",
                 "description": "Aria Kestrel deposited cash into Test Corp's account",
                 "amount": 50000000.0, "balance": 1250000000.0, "first_party_id": 2112000001,
                 "first_party_type": "character", "second_party_id": 98000001},
                {"id": 2, "date": "2026-10-09T14:00:00Z", "ref_type": "brokers_fee",
                 "description": "Market fee"}
            ]"#,
        )
        .unwrap();
        let contracts: Vec<Contract> = serde_json::from_str(
            r#"[
                {"contract_id": 5, "type": "item_exchange", "status": "finished",
                 "issuer_id": 2112000009, "issuer_corporation_id": 98000001, "assignee_id": 2112000002,
                 "acceptor_id": 2112000002, "for_corporation": true, "availability": "personal",
                 "date_issued": "2026-10-09T13:00:00Z", "date_expired": "2026-10-23T13:00:00Z",
                 "date_accepted": "2026-10-09T13:40:00Z", "price": 50000000.0, "volume": 1.0},
                {"contract_id": 6, "type": "courier", "status": "outstanding",
                 "issuer_id": 2112000009, "issuer_corporation_id": 98000001, "assignee_id": 0,
                 "acceptor_id": 0, "for_corporation": false, "availability": "public",
                 "date_issued": "2026-10-09T13:00:00Z", "date_expired": "2026-10-23T13:00:00Z"}
            ]"#,
        )
        .unwrap();

        let paid = merge(
            paid_from_journal(&journal, NOW, &rule()),
            paid_from_contracts(&contracts, CORP, NOW, &rule()),
        );
        assert_eq!(paid.len(), 2);
        assert_eq!(paid[&2112000001].via, Via::Donation);
        assert_eq!(paid[&2112000002].via, Via::Contract);
    }

    #[test]
    fn connect_state_round_trips() {
        let state = parse_connect_state(&connect_state(7, 2112345678, None)).unwrap();
        assert_eq!(state.account_id, 7);
        assert_eq!(state.character_id, 2112345678);
        assert_eq!(state.waitlist_id, None);
        // With a waitlist, so the login assigns it when it comes back
        let state = parse_connect_state(&connect_state(7, 2112345678, Some(5))).unwrap();
        assert_eq!(state.waitlist_id, Some(5));
        // A login started before the account, waitlist and time were carried
        let state = parse_connect_state(b"srpconnect:2112345678").unwrap();
        assert_eq!(state.account_id, 0);
        assert_eq!(state.character_id, 2112345678);
        assert_eq!(state.waitlist_id, None);

        assert_eq!(parse_connect_state(b"oauthhellostate"), None);
        assert_eq!(parse_connect_state(b"srpconnect:abc"), None);
        assert_eq!(parse_connect_state(b"srpconnect:"), None);
        assert_eq!(parse_connect_state(b"srpconnect:1:"), None);
        assert_eq!(parse_connect_state(b"srpconnect:7:1:0:1:2"), None);
    }

    // The login URL carries the state Branca-encrypted: the ids must not be readable in it
    #[test]
    fn connect_state_is_encrypted() {
        let mut branca = branca::Branca::new(&[7u8; 32]).unwrap();
        let encoded = branca
            .encode(&connect_state(7, 2112345678, Some(5)))
            .unwrap();
        assert!(!encoded.contains("srpconnect"));
        assert!(!encoded.contains("2112345678"));
        let decoded = branca.decode(&encoded, 60).unwrap();
        assert_eq!(
            parse_connect_state(&decoded).unwrap().character_id,
            2112345678
        );
    }

    #[test]
    fn connect_state_goes_stale() {
        let old = chrono::Utc::now().timestamp() - CONNECT_STATE_MAX_AGE_SECS as i64 - 1;
        assert_eq!(
            parse_connect_state(format!("srpconnect:7:1:0:{}", old).as_bytes()),
            None
        );
        let fresh = chrono::Utc::now().timestamp() - CONNECT_STATE_MAX_AGE_SECS as i64 + 60;
        assert!(parse_connect_state(format!("srpconnect:7:1:0:{}", fresh).as_bytes()).is_some());
    }

    #[test]
    fn state_goes_stale() {
        let mut corp = CorpState::default();
        assert!(!corp.is_fresh(NOW));
        corp.checked_at = Some(NOW - 60);
        assert!(corp.is_fresh(NOW));
        corp.checked_at = Some(NOW - STALE_AFTER_SECS);
        assert!(!corp.is_fresh(NOW));
    }

    fn paid_one() -> HashMap<i64, Payment> {
        paid_from_journal(
            &[donation("2026-10-09T13:00:00Z", 1, 50_000_000.0)],
            NOW,
            &rule(),
        )
    }

    #[test]
    fn connection_change_clears_that_corp_only() {
        let mut state = SrpState::default();
        assert_eq!(
            state.apply(CORP, 1, 0, Ok(paid_one().into()), NOW),
            Applied::Updated
        );
        assert_eq!(
            state.apply(CORP + 1, 1, 0, Ok(paid_one().into()), NOW),
            Applied::Updated
        );
        state.connect_failed(NOW, "failed".to_string());
        state.wallet_removed(CORP);
        assert!(!state.corp(CORP, 1).unwrap().is_fresh(NOW));
        assert!(state.corp(CORP, 1).unwrap().paid.is_empty());
        assert_eq!(state.generation(CORP), 1);
        assert!(state.corp(CORP + 1, 1).unwrap().is_fresh(NOW));
        assert_eq!(state.generation(CORP + 1), 0);
    }

    #[test]
    fn a_reconnect_keeps_the_last_good_result() {
        let mut state = SrpState::default();
        state.apply(CORP, 1, 0, Ok(paid_one().into()), NOW);
        state.connect_failed(NOW, "failed".to_string());
        state.wallet_replaced(CORP);
        assert!(state.recent_connect_error(NOW).is_none());
        assert!(state.corp(CORP, 1).unwrap().is_fresh(NOW));
        assert_eq!(state.corp(CORP, 1).unwrap().paid.len(), 1);
        assert_eq!(state.generation(CORP), 1);
    }

    #[test]
    fn changing_a_wallet_keeps_its_divisions_apart() {
        let mut state = SrpState::default();
        state.apply(CORP, 1, 0, Ok(paid_one().into()), NOW);
        // The same corp read at another division is untouched by a reconnect on the first
        state.apply(CORP, 2, 0, Ok(paid_one().into()), NOW);
        state.wallet_replaced(CORP);
        assert!(state.corp(CORP, 1).unwrap().is_fresh(NOW));
        assert_eq!(state.generation(CORP), 1);
        // A result started before the bump is thrown away for every division
        assert_eq!(
            state.apply(CORP, 2, 0, Ok(paid_one().into()), NOW + 60),
            Applied::Discarded
        );
        assert_eq!(state.corp(CORP, 2).unwrap().checked_at, Some(NOW));
    }

    #[test]
    fn connect_error_is_shown_for_a_while() {
        let mut state = SrpState::default();
        state.connect_failed(NOW, "failed".to_string());
        assert_eq!(
            state.recent_connect_error(NOW + 60).as_deref(),
            Some("failed")
        );
        assert_eq!(
            state.recent_connect_error(NOW + CONNECT_ERROR_SHOWN_SECS),
            None
        );
    }

    #[test]
    fn a_failed_check_keeps_the_last_good_result() {
        let mut corp = CorpState::default();
        assert_eq!(corp.apply(Ok(paid_one().into()), NOW), Applied::Updated);
        assert_eq!(
            corp.apply(Err("ESI down".to_string()), NOW + 300),
            Applied::Failed
        );
        assert_eq!(corp.paid.len(), 1);
        assert_eq!(corp.checked_at, Some(NOW));
        assert_eq!(corp.last_error.as_deref(), Some("ESI down"));
        assert_eq!(
            corp.apply(Ok(paid_one().into()), NOW + 600),
            Applied::Recovered
        );
        assert!(corp.last_error.is_none());
    }

    #[test]
    fn a_check_for_an_old_wallet_is_thrown_away() {
        let mut state = SrpState::default();
        let started_with = state.generation(CORP);
        state.wallet_replaced(CORP);
        assert_eq!(
            state.apply(CORP, 1, started_with, Ok(paid_one().into()), NOW),
            Applied::Discarded
        );
        // Discarded before it could create the division's entry
        assert!(state.corp(CORP, 1).is_none());
        let generation = state.generation(CORP);
        assert_eq!(
            state.apply(CORP, 1, generation, Ok(paid_one().into()), NOW),
            Applied::Updated
        );
    }

    #[test]
    fn recent_is_newest_first_and_only_covered() {
        let mut corp = CorpState::default();
        corp.paid = paid_from_journal(
            &[
                donation("2026-10-09T09:00:00Z", 1, 50_000_000.0),
                donation("2026-10-09T13:00:00Z", 2, 50_000_000.0),
            ],
            NOW,
            &rule(),
        );
        let ids: Vec<i64> = corp
            .recent(NOW, &rule())
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(ids, vec![2, 1]);
        // 1 paid at 09:00, so its cover ends at 09:00 the next day
        let ids: Vec<i64> = corp
            .recent(NOW + 20 * 3600, &rule())
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(ids, vec![2]);
    }
}
