use std::collections::{HashMap, HashSet};

use rocket::response::Redirect;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{
    app::Application,
    core::{
        auth::{authorize_character, AuthenticatedAccount},
        esi::ESIClient,
        esi::ESIError,
        srp::{self, connection, database_error, Payment, Rule, SrpState, Via},
    },
    data::character,
    util::madness::Madness,
};

fn parse_ids(character_ids: &str) -> Result<Vec<i64>, Madness> {
    let ids = character_ids
        .split(',')
        .map(|id| id.trim().parse::<i64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Madness::BadRequest("Invalid character_ids".to_string()))?;
    let mut seen = HashSet::new();
    Ok(ids.into_iter().filter(|id| seen.insert(*id)).collect())
}

#[derive(Debug, Serialize)]
struct CharacterStatus {
    id: i64,
    paid: bool,
    expires_at: Option<i64>,
    via: Option<Via>,
}

fn character_status(
    paid: &HashMap<i64, Payment>,
    id: i64,
    now: i64,
    rule: &Rule,
) -> CharacterStatus {
    let payment = srp::current_payment(paid, id, now, rule);
    CharacterStatus {
        id,
        paid: payment.is_some(),
        expires_at: payment.map(|p| rule.expires_at(p.paid_at)),
        via: payment.map(|p| p.via),
    }
}

/// What pilots on a waitlist see, given the waitlist's SRP corp
#[derive(Debug)]
enum WaitlistStatus {
    /// The waitlist has no SRP corp
    NotApplicable,
    /// Its corp has no recent successful check: show nothing rather than guess
    Unavailable,
    /// Each character's cover from payments to that corp
    Available {
        checked_at: Option<i64>,
        /// When ESI produced the wallet data; a payment made after it can't show yet
        data_as_of: Option<i64>,
        characters: Vec<CharacterStatus>,
    },
}

fn waitlist_status(
    state: &SrpState,
    corporation_id: Option<i64>,
    ids: &[i64],
    now: i64,
    rule: &Rule,
) -> WaitlistStatus {
    let corporation_id = match corporation_id {
        Some(c) => c,
        None => return WaitlistStatus::NotApplicable,
    };
    match state.corp(corporation_id, rule.division) {
        Some(corp) if corp.is_fresh(now) => WaitlistStatus::Available {
            checked_at: corp.checked_at,
            data_as_of: corp.data_as_of,
            characters: ids
                .iter()
                .map(|&id| character_status(&corp.paid, id, now, rule))
                .collect(),
        },
        _ => WaitlistStatus::Unavailable,
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    /// False when nothing can feed this waitlist: no assignment, or its assigned account's wallet
    /// is gone. The page can stop asking.
    applies: bool,
    /// False also when there's no recent successful check of the waitlist's SRP corp (not
    /// connected, or failing for over two hours): show nothing rather than guess
    available: bool,
    checked_at: Option<i64>,
    division: i32,
    min_amount: i64,
    coverage_hours: i64,
    characters: Vec<CharacterStatus>,
}

/// The characters' SRP cover on `waitlist_id`, from payments to that waitlist's SRP corp
#[get("/api/srp/status?<character_ids>&<waitlist_id>")]
async fn status(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    character_ids: &str,
    waitlist_id: i64,
) -> Result<Json<StatusResponse>, Madness> {
    let ids = parse_ids(character_ids)?;

    let now = chrono::Utc::now().timestamp();
    // The waitlist's own rule, also when SRP doesn't feed it: the page can then still say what its
    // payments would count
    let target = srp::waitlists::target(app.get_db(), waitlist_id).await?;
    let rule = target.as_ref().map(|t| t.rule).unwrap_or_default();
    let nothing = |applies| StatusResponse {
        applies,
        available: false,
        checked_at: None,
        division: rule.division,
        min_amount: rule.min_amount_isk(),
        coverage_hours: rule.coverage_hours(),
        characters: Vec::new(),
    };

    // Before checking the characters: most requests come from waitlists SRP doesn't apply to
    let corporation_id = match target.and_then(|t| t.corporation_id) {
        Some(c) => c,
        None => return Ok(Json(nothing(false))),
    };

    for &id in &ids {
        authorize_character(app.get_db(), &account, id, None).await?;
    }

    let status = waitlist_status(
        &app.srp_state.read(),
        Some(corporation_id),
        &ids,
        now,
        &rule,
    );
    let (checked_at, characters) = match status {
        WaitlistStatus::Available {
            checked_at,
            characters,
            ..
        } => (checked_at, characters),
        WaitlistStatus::Unavailable => return Ok(Json(nothing(true))),
        WaitlistStatus::NotApplicable => return Ok(Json(nothing(false))),
    };

    Ok(Json(StatusResponse {
        applies: true,
        available: true,
        checked_at,
        division: rule.division,
        min_amount: rule.min_amount_isk(),
        coverage_hours: rule.coverage_hours(),
        characters,
    }))
}

#[derive(Debug, Serialize)]
struct FleetStatusResponse {
    /// As for pilots: false when the waitlist has no SRP wallet
    applies: bool,
    available: bool,
    checked_at: Option<i64>,
    /// When ESI produced the wallet data. A member who joined the fleet after it may have paid
    /// already without it showing.
    data_as_of: Option<i64>,
    characters: Vec<CharacterStatus>,
}

/// For FCs: the SRP cover of fleet members on `waitlist_id`. Any characters, unlike the pilot
/// route, so it needs fleet access.
#[get("/api/srp/fleet?<character_ids>&<waitlist_id>")]
async fn fleet_status(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    character_ids: &str,
    waitlist_id: i64,
) -> Result<Json<FleetStatusResponse>, Madness> {
    account.require_access("fleet-view")?;
    let ids = parse_ids(character_ids)?;

    let target = srp::waitlists::target(app.get_db(), waitlist_id).await?;
    let now = chrono::Utc::now().timestamp();
    let rule = target.as_ref().map(|t| t.rule).unwrap_or_default();
    let status = waitlist_status(
        &app.srp_state.read(),
        target.and_then(|t| t.corporation_id),
        &ids,
        now,
        &rule,
    );

    let response = match status {
        WaitlistStatus::Available {
            checked_at,
            data_as_of,
            characters,
        } => FleetStatusResponse {
            applies: true,
            available: true,
            checked_at,
            data_as_of,
            characters,
        },
        WaitlistStatus::Unavailable => FleetStatusResponse {
            applies: true,
            available: false,
            checked_at: None,
            data_as_of: None,
            characters: Vec::new(),
        },
        WaitlistStatus::NotApplicable => FleetStatusResponse {
            applies: false,
            available: false,
            checked_at: None,
            data_as_of: None,
            characters: Vec::new(),
        },
    };
    Ok(Json(response))
}

#[derive(Debug, Serialize)]
struct NamedCharacter {
    id: i64,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct RecentPayment {
    character: NamedCharacter,
    amount: f64,
    paid_at: i64,
    expires_at: i64,
    via: Via,
}

/// One division a waitlist of the corp reads, with its check
#[derive(Debug, Serialize)]
struct WalletCheck {
    division: i32,
    /// The loosest rule of the waitlists reading this division: what `recent` counts
    min_amount: i64,
    coverage_hours: i64,
    checked_at: Option<i64>,
    last_error: Option<String>,
    /// Payments still inside the coverage window, newest first. How an admin checks the result
    /// against the in-game wallet.
    recent: Vec<RecentPayment>,
}

/// One connected SRP wallet account and its checks
#[derive(Debug, Serialize)]
struct Wallet {
    character: NamedCharacter,
    corporation_id: i64,
    corporation_name: String,
    connected_by: NamedCharacter,
    connected_at: i64,
    /// One per division a waitlist of this corp uses; empty while none does
    checks: Vec<WalletCheck>,
}

#[derive(Debug, Serialize)]
struct AdminWaitlist {
    id: i64,
    name: String,
    /// The wallet account whose payments cover its pilots; None for no SRP. May be a character
    /// whose wallet has since been disconnected.
    character: Option<NamedCharacter>,
    /// A corp it used before the account was picked (or while its wallet is disconnected), from
    /// the migration; the character wins once set.
    corporation_id: Option<i64>,
    /// The rule the waitlist applies
    division: i32,
    min_amount: i64,
    coverage_hours: i64,
    set_by: Option<NamedCharacter>,
    set_at: Option<i64>,
}

/// What a waitlist with no rule yet starts with
#[derive(Debug, Serialize)]
struct RuleDefaults {
    division: i32,
    min_amount: i64,
    coverage_hours: i64,
}

#[derive(Debug, Serialize)]
struct AdminResponse {
    defaults: RuleDefaults,
    /// Why the last connect failed, for a few minutes after it did
    connect_error: Option<String>,
    /// By corp name, then character
    wallets: Vec<Wallet>,
    /// Every waitlist that isn't archived, by name
    waitlists: Vec<AdminWaitlist>,
}

fn require_manager(account: &AuthenticatedAccount) -> Result<(), Madness> {
    account.require_access("srp-manage")?;
    Ok(())
}

#[get("/api/srp/admin")]
async fn admin_status(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
) -> Result<Json<AdminResponse>, Madness> {
    require_manager(&account)?;

    let now = chrono::Utc::now().timestamp();

    let connections = connection::all(app.get_db()).await?;
    let waitlists = srp::waitlists::list(app.get_db()).await?;

    // A migrated waitlist may only have kept a corp: if an account of that corp is connected, it
    // is what feeds the waitlist now, so show it as the row's wallet
    let mut account_for_corp: HashMap<i64, i64> = HashMap::new();
    let mut corp_for_character: HashMap<i64, i64> = HashMap::new();
    for c in &connections {
        account_for_corp
            .entry(c.corporation_id)
            .or_insert(c.character_id);
        corp_for_character.insert(c.character_id, c.corporation_id);
    }

    // Per corp and division: the loosest rule of the waitlists that read it. The checks below and
    // their recent payments use it, so an admin sees everything any of those waitlists counts.
    let mut rules: HashMap<(i64, i32), srp::Rule> = HashMap::new();
    for w in &waitlists {
        let corporation = match (w.character_id, w.pending_corporation_id) {
            (Some(character_id), _) => corp_for_character.get(&character_id).copied(),
            (None, corporation_id) => corporation_id,
        };
        let rule = w.rule();
        if let Some(corporation) = corporation {
            rules
                .entry((corporation, rule.division))
                .and_modify(|existing| *existing = existing.loosest(rule))
                .or_insert(rule);
        }
    }

    // Copied out so the lock isn't held across the name lookup below
    struct Check {
        division: i32,
        rule: srp::Rule,
        checked_at: Option<i64>,
        last_error: Option<String>,
        recent: Vec<(i64, Payment)>,
    }
    let (connect_error, checks) = {
        let state = app.srp_state.read();
        let checks: Vec<Vec<Check>> = connections
            .iter()
            .map(|c| {
                let mut divisions: Vec<(i32, srp::Rule)> = rules
                    .iter()
                    .filter(|((corporation, _), _)| *corporation == c.corporation_id)
                    .map(|((_, division), rule)| (*division, *rule))
                    .collect();
                divisions.sort_by_key(|(division, _)| *division);
                divisions
                    .into_iter()
                    .map(
                        |(division, rule)| match state.corp(c.corporation_id, division) {
                            Some(corp) => Check {
                                division,
                                rule,
                                checked_at: corp.checked_at,
                                last_error: corp.last_error.clone(),
                                recent: corp.recent(now, &rule),
                            },
                            None => Check {
                                division,
                                rule,
                                checked_at: None,
                                last_error: None,
                                recent: Vec::new(),
                            },
                        },
                    )
                    .collect()
            })
            .collect();
        (state.recent_connect_error(now), checks)
    };

    let mut ids: Vec<i64> = Vec::new();
    for (c, checks) in connections.iter().zip(&checks) {
        ids.extend([c.character_id, c.connected_by]);
        for check in checks {
            ids.extend(check.recent.iter().map(|(id, _)| *id));
        }
    }
    ids.extend(waitlists.iter().filter_map(|w| w.set_by));
    ids.extend(waitlists.iter().filter_map(|w| w.character_id));
    let names = character::lookup(app.get_db(), &ids).await?;
    let named = |id: i64| NamedCharacter {
        id,
        name: names.get(&id).map(|c| c.name.clone()),
    };

    let wallets = connections
        .into_iter()
        .zip(checks)
        .map(|(c, checks)| Wallet {
            character: named(c.character_id),
            corporation_id: c.corporation_id,
            corporation_name: c.corporation_name,
            connected_by: named(c.connected_by),
            connected_at: c.connected_at,
            checks: checks
                .into_iter()
                .map(|check| {
                    let rule = check.rule;
                    WalletCheck {
                        division: check.division,
                        min_amount: rule.min_amount_isk(),
                        coverage_hours: rule.coverage_hours(),
                        checked_at: check.checked_at,
                        last_error: check.last_error,
                        recent: check
                            .recent
                            .into_iter()
                            .map(|(id, p)| RecentPayment {
                                character: named(id),
                                amount: p.amount,
                                paid_at: p.paid_at,
                                expires_at: rule.expires_at(p.paid_at),
                                via: p.via,
                            })
                            .collect(),
                    }
                })
                .collect(),
        })
        .collect();

    Ok(Json(AdminResponse {
        defaults: RuleDefaults {
            division: srp::DEFAULT_DIVISION,
            min_amount: srp::DEFAULT_MIN_AMOUNT,
            coverage_hours: srp::DEFAULT_COVERAGE_HOURS,
        },
        connect_error,
        wallets,
        waitlists: waitlists
            .into_iter()
            .map(|w| {
                let rule = w.rule();
                AdminWaitlist {
                    id: w.id,
                    character: w
                        .character_id
                        .or_else(|| {
                            w.pending_corporation_id
                                .and_then(|c| account_for_corp.get(&c).copied())
                        })
                        .map(|id| NamedCharacter {
                            id,
                            name: names.get(&id).map(|c| c.name.clone()),
                        }),
                    corporation_id: w.pending_corporation_id,
                    division: rule.division,
                    min_amount: rule.min_amount_isk(),
                    coverage_hours: rule.coverage_hours(),
                    name: w.name,
                    set_by: w.set_by.map(named),
                    set_at: w.set_at,
                }
            })
            .collect(),
    }))
}

/// Starts connecting `character_id` as an SRP wallet: returns the SeAT login URL. With
/// `waitlist_id` the wallet is also assigned to that waitlist when the login comes back, which
/// hands it to `finish_connect`.
#[get("/api/srp/admin/connect?<character_id>&<waitlist_id>")]
async fn connect(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    character_id: i64,
    waitlist_id: Option<i64>,
) -> Result<String, Madness> {
    require_manager(&account)?;
    if let Some(waitlist_id) = waitlist_id {
        if srp::waitlists::active_name(app.get_db(), waitlist_id)
            .await?
            .is_none()
        {
            return Err(Madness::BadRequest("No such waitlist".to_string()));
        }
    }
    authorize_character(app.get_db(), &account, character_id, None).await?;

    Ok(super::auth::authorize_url(
        app,
        &srp::connect_scopes(),
        &srp::connect_state(account.id, character_id, waitlist_id),
    ))
}

#[derive(Deserialize)]
struct Disconnect {
    character_id: i64,
}

/// Removes one SRP wallet account. Waitlists using it keep it, and show nothing until it's
/// reconnected.
#[post("/api/srp/admin/disconnect", data = "<input>")]
async fn disconnect(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    input: Json<Disconnect>,
) -> Result<&'static str, Madness> {
    require_manager(&account)?;
    let connection = connection::get(app.get_db(), input.character_id)
        .await?
        .ok_or_else(|| Madness::BadRequest("That wallet isn't connected".to_string()))?;
    // Only the connection that was read: a reconnect since then must survive
    if !connection::delete(app.get_db(), input.character_id, connection.connected_at).await? {
        return Err(Madness::BadRequest(
            "The wallet changed while disconnecting. Try again.".to_string(),
        ));
    }
    // Another account of the same corp can still feed its checks
    if !connection::corporation_connected(app.get_db(), connection.corporation_id).await? {
        app.srp_state
            .write()
            .wallet_removed(connection.corporation_id);
    }
    info!(
        "SRP wallet character {} disconnected by {}",
        input.character_id, account.id
    );
    Ok("OK")
}

#[derive(Deserialize)]
struct RuleInput {
    division: i32,
    min_amount: i64,
    coverage_hours: i64,
}

#[derive(Deserialize)]
struct SetWaitlist {
    waitlist_id: i64,
    /// None for no SRP, or for a corporation_id to keep as the fallback
    character_id: Option<i64>,
    /// The corp kept for a waitlist whose wallet account is gone
    corporation_id: Option<i64>,
    /// The rule to apply; None keeps the current one
    rule: Option<RuleInput>,
}

/// Sets which wallet account pays SRP for a waitlist, or none
#[post("/api/srp/admin/waitlist", data = "<input>")]
async fn set_waitlist(
    app: &rocket::State<Application>,
    account: AuthenticatedAccount,
    input: Json<SetWaitlist>,
) -> Result<&'static str, Madness> {
    require_manager(&account)?;
    let name = srp::waitlists::active_name(app.get_db(), input.waitlist_id)
        .await?
        .ok_or_else(|| Madness::BadRequest("No such waitlist".to_string()))?;
    let current = srp::waitlists::assignment(app.get_db(), input.waitlist_id).await?;
    if let Some(character_id) = input.character_id {
        // A disconnected wallet can stay (it's the orphan entry in the picker); anything else
        // must at least be a character the waitlist knows
        let known = connection::exists(app.get_db(), character_id).await?
            || character::lookup(app.get_db(), &[character_id])
                .await?
                .contains_key(&character_id);
        if !known {
            return Err(Madness::BadRequest("No such character".to_string()));
        }
    }
    if let Some(corporation_id) = input.corporation_id {
        // A waitlist can keep a corp while its wallet is gone, but a new corp must be one SRP
        // actually watches: otherwise a typo pins the waitlist to payments nobody collects
        if !connection::corporation_connected(app.get_db(), corporation_id).await?
            && current.1 != Some(corporation_id)
        {
            return Err(Madness::BadRequest(
                "That corp's wallet isn't connected".to_string(),
            ));
        }
    }

    let rule = match &input.rule {
        Some(rule) => {
            srp::validate_rule(rule.division, rule.min_amount, rule.coverage_hours)
                .map_err(Madness::BadRequest)?;
            Some(srp::Rule::new(
                rule.division,
                rule.min_amount,
                rule.coverage_hours,
            ))
        }
        None => None,
    };

    let now = chrono::Utc::now().timestamp();
    if srp::waitlists::set(
        app.get_db(),
        input.waitlist_id,
        input.character_id,
        input.corporation_id,
        rule,
        account.id,
        now,
    )
    .await?
    {
        match (input.character_id, input.corporation_id) {
            (Some(c), _) => info!(
                "SRP on waitlist {} ({}) now from wallet character {}, by {}",
                name, input.waitlist_id, c, account.id
            ),
            (None, Some(c)) => info!(
                "SRP on waitlist {} ({}) kept for corp {}, waiting for a wallet, by {}",
                name, input.waitlist_id, c, account.id
            ),
            (None, None) => info!(
                "SRP removed from waitlist {} ({}) by {}",
                name, input.waitlist_id, account.id
            ),
        }
        if let Some(rule) = rule {
            info!(
                "SRP rule on waitlist {} ({}) now division {}, {} ISK, {}h cover, by {}",
                name,
                input.waitlist_id,
                rule.division,
                rule.min_amount_isk(),
                rule.coverage_hours(),
                account.id
            );
        }
        // A new assignment or division shouldn't wait for the periodic pass
        app.srp_state.check_now();
    }
    Ok("OK")
}

/// Completes a connect started by `connect`, then sends the admin back to the SRP wallet page,
/// which shows the reason if it failed. The check itself runs in the updater, not in this login
/// redirect: it can take a while, and the request may be dropped.
pub async fn finish_connect(
    app: &Application,
    account: Option<AuthenticatedAccount>,
    code: &str,
    state: srp::ConnectState,
) -> Redirect {
    match save_connection(app, account, code, &state).await {
        Ok((corporation_id, assignment_error)) => {
            app.srp_state.write().wallet_replaced(corporation_id);
            app.srp_state.check_now();
            // The wallet is connected: a failed waitlist assignment must not undo it, but the
            // admin has to hear about it
            if let Some(message) = assignment_error {
                warn!(
                    "SRP wallet connected but its waitlist assignment failed: {}",
                    message
                );
                let now = chrono::Utc::now().timestamp();
                app.srp_state.write().connect_failed(now, message);
            }
        }
        Err(message) => {
            warn!("SRP wallet connect failed: {}", message);
            let now = chrono::Utc::now().timestamp();
            app.srp_state.write().connect_failed(now, message);
        }
    }
    Redirect::to("/fc/srp")
}

/// The corp `character_id` is in, as id and name
async fn corporation_of(esi: &ESIClient, character_id: i64) -> Result<(i64, String), String> {
    #[derive(Deserialize)]
    struct CharacterInfo {
        corporation_id: i64,
    }
    #[derive(Deserialize)]
    struct CorporationInfo {
        name: String,
    }
    let info: CharacterInfo = esi
        .get(
            &format!("/v5/characters/{}/", character_id),
            character_id,
            None,
        )
        .await
        .map_err(|e| format!("Couldn't look up the character's corp: {}", e))?;
    let corporation: CorporationInfo = esi
        .get(
            &format!("/v5/corporations/{}/", info.corporation_id),
            character_id,
            None,
        )
        .await
        .map_err(|e| format!("Couldn't look up the corp: {}", e))?;
    Ok((info.corporation_id, corporation.name))
}

/// Stores the login as an SRP wallet, replacing the character's previous one, and assigns it to
/// `waitlist_id` when given. Returns the corp.
async fn save_connection(
    app: &Application,
    account: Option<AuthenticatedAccount>,
    code: &str,
    state: &srp::ConnectState,
) -> Result<(i64, Option<String>), String> {
    let account = account.ok_or(
        "Log in to the waitlist first, then connect the SRP wallet from the FC dashboard.",
    )?;
    if !srp::is_manager(&account) {
        return Err("You don't have permission to connect the SRP wallet.".to_string());
    }
    // The state is encrypted, but binding it to the account stops a connect link started by
    // someone else from being completed in this session (0: a login started before the binding)
    if state.account_id != 0 && account.id != state.account_id {
        return Err(
            "This connect link was started by another account. Start it again from the SRP wallet \
            page."
                .to_string(),
        );
    }
    let character_id = state.character_id;
    let waitlist_id = state.waitlist_id;

    // Not save_auth: this login must not replace the admin's normal one
    let auth = app
        .esi_client
        .exchange_code(code, &srp::connect_scopes())
        .await
        .map_err(|e| match e {
            ESIError::MissingScope => "SeAT didn't grant the wallet and contracts permissions. \
                Check the character has them in SeAT, then try again."
                .to_string(),
            e => format!("The SeAT login failed: {}", e),
        })?;
    let character_name = auth
        .alt_ids
        .iter()
        .flatten()
        .find(|a| a.id == character_id)
        .map(|a| a.name.clone())
        .or_else(|| (auth.character_id == character_id).then(|| auth.character_name.clone()))
        .ok_or("The character you picked isn't on the SeAT account you logged in with.")?;
    let refresh_token = auth
        .refresh_token
        .ok_or("SeAT didn't return a refresh token.")?;

    let (corporation_id, corporation_name) = corporation_of(&app.esi_client, character_id).await?;

    character::upsert(app.get_db(), character_id, &character_name)
        .await
        .map_err(database_error)?;
    connection::save(
        app.get_db(),
        &connection::Connection {
            character_id,
            corporation_id,
            corporation_name,
            refresh_token,
            connected_by: account.id,
            connected_at: chrono::Utc::now().timestamp(),
        },
    )
    .await
    .map_err(database_error)?;

    // Started from a waitlist row: use the new wallet there straight away. A waitlist archived
    // while the login was open is skipped. The wallet itself is already saved, so an assignment
    // error is reported without undoing it.
    let mut assignment_error = None;
    if let Some(waitlist_id) = waitlist_id {
        // Nothing here may turn the saved wallet into an error: only `assignment_error`
        match srp::waitlists::active_name(app.get_db(), waitlist_id).await {
            Err(e) => assignment_error = Some(database_error(e)),
            Ok(Some(_)) => {
                if let Err(e) = srp::waitlists::set(
                    app.get_db(),
                    waitlist_id,
                    Some(character_id),
                    Some(corporation_id),
                    None,
                    account.id,
                    chrono::Utc::now().timestamp(),
                )
                .await
                {
                    assignment_error = Some(database_error(e));
                }
            }
            Ok(None) => {}
        }
    }

    info!(
        "SRP wallet of corp {} connected: character {}, by {}",
        corporation_id, character_id, account.id
    );
    Ok((corporation_id, assignment_error))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        status,
        fleet_status,
        admin_status,
        set_waitlist,
        connect,
        disconnect
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const TDF_CORP: i64 = 98000001;
    const CN_CORP: i64 = 98000002;
    const NOW: i64 = 1_000_000;

    fn paid(character_id: i64) -> HashMap<i64, Payment> {
        [(
            character_id,
            Payment {
                paid_at: NOW - 3600,
                amount: 50_000_000.0,
                via: Via::Donation,
            },
        )]
        .into()
    }

    fn rule() -> Rule {
        Rule::default()
    }

    fn paid_ids(status: WaitlistStatus) -> Vec<i64> {
        match status {
            WaitlistStatus::Available { characters, .. } => characters
                .into_iter()
                .filter(|c| c.paid)
                .map(|c| c.id)
                .collect(),
            other => panic!("expected a status, got {:?}", other),
        }
    }

    #[test]
    fn character_ids_are_parsed_and_deduplicated() {
        assert_eq!(parse_ids("1, 2,1,3").unwrap(), vec![1, 2, 3]);
        assert!(parse_ids("1,x").is_err());
    }

    // The fleet page tells "may have paid, can't show yet" from "not paid" by the data's age

    #[test]
    fn status_carries_when_the_wallet_data_was_produced() {
        let rule = rule();
        let mut state = SrpState::default();
        let read = srp::WalletRead {
            paid: paid(1),
            data_as_of: Some(NOW - 1200),
        };
        state.corp_mut(TDF_CORP, rule.division).apply(Ok(read), NOW);
        match waitlist_status(&state, Some(TDF_CORP), &[1, 2], NOW, &rule) {
            WaitlistStatus::Available {
                checked_at,
                data_as_of,
                ..
            } => {
                assert_eq!(checked_at, Some(NOW));
                assert_eq!(data_as_of, Some(NOW - 1200));
            }
            other => panic!("expected a status, got {:?}", other),
        }
    }

    // Two waitlists on different SRP corps: a pilot is covered only on the waitlist whose corp
    // they paid
    #[test]
    fn each_waitlist_counts_only_its_own_corps_payments() {
        let rule = rule();
        let mut state = SrpState::default();
        // Character 1 paid the TDF corp, character 2 the CN corp
        state
            .corp_mut(TDF_CORP, rule.division)
            .apply(Ok(paid(1).into()), NOW);
        state
            .corp_mut(CN_CORP, rule.division)
            .apply(Ok(paid(2).into()), NOW);
        let pilots = [1, 2, 3];

        let tdf_waitlist = waitlist_status(&state, Some(TDF_CORP), &pilots, NOW, &rule);
        assert_eq!(paid_ids(tdf_waitlist), vec![1]);
        let cn_waitlist = waitlist_status(&state, Some(CN_CORP), &pilots, NOW, &rule);
        assert_eq!(paid_ids(cn_waitlist), vec![2]);

        // A waitlist with no SRP corp shows nothing
        assert!(matches!(
            waitlist_status(&state, None, &pilots, NOW, &rule),
            WaitlistStatus::NotApplicable
        ));
        // Nor does one whose corp has no wallet (disconnected) or no recent check
        assert!(matches!(
            waitlist_status(&state, Some(98000003), &pilots, NOW, &rule),
            WaitlistStatus::Unavailable
        ));
        state.wallet_removed(CN_CORP);
        assert!(matches!(
            waitlist_status(&state, Some(CN_CORP), &pilots, NOW, &rule),
            WaitlistStatus::Unavailable
        ));
        // The other corp's waitlist is unaffected
        let tdf_waitlist = waitlist_status(&state, Some(TDF_CORP), &pilots, NOW, &rule);
        assert_eq!(paid_ids(tdf_waitlist), vec![1]);
    }

    // Two waitlists on one corp's division: the check runs with the loosest rule, each waitlist
    // then applies its own
    #[test]
    fn each_waitlist_applies_its_own_rule() {
        let mut state = SrpState::default();
        let mut payments = paid(1);
        payments.insert(
            2,
            Payment {
                paid_at: NOW - 30 * 3600,
                amount: 150_000_000.0,
                via: Via::Donation,
            },
        );
        state.corp_mut(TDF_CORP, 1).apply(Ok(payments.into()), NOW);

        let strict = Rule::new(1, 100_000_000, 24);
        let loose = Rule::new(1, 50_000_000, 48);
        let pilots = [1, 2, 3];
        assert!(paid_ids(waitlist_status(
            &state,
            Some(TDF_CORP),
            &pilots,
            NOW,
            &strict
        ))
        .is_empty());
        assert_eq!(
            paid_ids(waitlist_status(
                &state,
                Some(TDF_CORP),
                &pilots,
                NOW,
                &loose
            )),
            vec![1, 2]
        );
    }

    // The same corp can be watched in different divisions, which have separate journals
    #[test]
    fn a_waitlists_division_selects_the_journal() {
        let mut state = SrpState::default();
        state.corp_mut(TDF_CORP, 2).apply(Ok(paid(1).into()), NOW);
        let pilots = [1];
        let division_one = Rule::new(1, 50_000_000, 24);
        let division_two = Rule::new(2, 50_000_000, 24);
        // Division 1 was never read: nothing there to show
        assert!(matches!(
            waitlist_status(&state, Some(TDF_CORP), &pilots, NOW, &division_one),
            WaitlistStatus::Unavailable
        ));
        assert_eq!(
            paid_ids(waitlist_status(
                &state,
                Some(TDF_CORP),
                &pilots,
                NOW,
                &division_two
            )),
            vec![1]
        );
        // Disconnecting clears every division of the corp
        state.wallet_removed(TDF_CORP);
        assert!(matches!(
            waitlist_status(&state, Some(TDF_CORP), &pilots, NOW, &division_two),
            WaitlistStatus::Unavailable
        ));
    }

    #[test]
    fn character_status_shows_only_current_cover() {
        let rule = rule();
        let paid: HashMap<i64, Payment> = [(
            1,
            Payment {
                paid_at: 1000,
                amount: 50_000_000.0,
                via: Via::Contract,
            },
        )]
        .into();

        let covered = character_status(&paid, 1, 2000, &rule);
        assert!(covered.paid);
        assert_eq!(covered.expires_at, Some(1000 + 24 * 3600));
        assert_eq!(covered.via, Some(Via::Contract));

        let expired = character_status(&paid, 1, 1000 + 24 * 3600, &rule);
        assert!(!expired.paid && expired.expires_at.is_none() && expired.via.is_none());
        assert!(!character_status(&paid, 2, 2000, &rule).paid);
    }

    // Runs the real handler against a throwaway Postgres to check what corporation_id values and
    // rules set_waitlist accepts. Setup: docker run --rm -d -e POSTGRES_PASSWORD=pass
    // -p 55432:5432 postgres:16, then create database refute1 with `character` and `waitlist`
    // tables and run sql/migration_postgres_0005.sql and 0007 on it.
    // Run with: cargo test --bin tdf_wl -- --ignored
    #[ignore = "needs a throwaway Postgres on 127.0.0.1:55432; see the comment"]
    #[rocket::async_test]
    async fn set_waitlist_corp_validation() {
        use crate::config::{
            AppConfig, Config, DatabaseConfig, ESIConfig, FleetUpdaterConfig, SSEConfig,
            SkillUpdaterConfig,
        };
        use std::sync::Arc;

        let url = "postgres://postgres:pass@127.0.0.1:55432/refute1";
        let pool = sqlx::Pool::<sqlx::Postgres>::connect(url).await.unwrap();
        sqlx::query(
            "ALTER TABLE waitlist ADD COLUMN IF NOT EXISTS name varchar NOT NULL DEFAULT ''",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "ALTER TABLE waitlist ADD COLUMN IF NOT EXISTS is_archived boolean NOT NULL DEFAULT false",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "ALTER TABLE waitlist ADD COLUMN IF NOT EXISTS is_open boolean NOT NULL DEFAULT false",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS alt_character (account_id int8, alt_id int8)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM srp_waitlist")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM srp_connection")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM waitlist")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM character")
            .execute(&pool)
            .await
            .unwrap();
        // A previous run may have left rows behind
        sqlx::query(
            "INSERT INTO character (id, name) VALUES (1, 'a'), (2, 'b'), (3, 'c')
                ON CONFLICT (id) DO NOTHING",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO waitlist (id, name, is_open, is_archived)
                VALUES (42, 'wl', false, false), (43, 'wl2', false, false)
                ON CONFLICT (id) DO NOTHING",
        )
        .execute(&pool)
        .await
        .unwrap();

        let config = Config {
            database: DatabaseConfig {
                url: url.to_string(),
                min_connections: 1,
                max_connections: 2,
                idle_timeout: 60,
                connect_timeout: 60,
            },
            app: AppConfig {
                token_secret: "00".repeat(32),
            },
            esi: ESIConfig {
                client_id: "x".to_string(),
                client_secret: "y".to_string(),
                url: "http://127.0.0.1".to_string(),
            },
            sse: SSEConfig {
                url: "http://127.0.0.1".to_string(),
                secret: "00".repeat(32),
            },
            fleet_updater: FleetUpdaterConfig {
                enable: false,
                min_in_fleet: 1,
            },
            skill_updater: SkillUpdaterConfig {
                enable: false,
                runtime: 1.0,
            },
        };
        let db: Arc<crate::DB> = Arc::new(pool.clone());
        let app = crate::app::new(db, config, srp::new_shared_state());
        let state = rocket::State::from(&app);

        let account = || AuthenticatedAccount {
            id: 1,
            access: crate::core::auth::get_access_keys("fc").unwrap(),
        };
        let input = |waitlist_id, corporation_id| {
            Json(SetWaitlist {
                waitlist_id,
                character_id: None,
                corporation_id,
                rule: None,
            })
        };

        // A corp with no connected wallet on a fresh waitlist is rejected
        match set_waitlist(state, account(), input(42, Some(98000002))).await {
            Err(Madness::BadRequest(m)) => {
                assert!(m.contains("isn't connected"), "{}", m);
            }
            other => panic!("expected rejection, got {:?}", other),
        }
        // A bogus corp id is rejected too
        assert!(matches!(
            set_waitlist(state, account(), input(43, Some(1))).await,
            Err(Madness::BadRequest(_))
        ));

        // Once any wallet of that corp is connected, the bare corp is accepted
        sqlx::query(
            "INSERT INTO srp_connection (character_id, corporation_id, corporation_name,
                refresh_token, connected_by, connected_at)
                VALUES (2, 98000002, 'CN', 't', 1, 0)
                ON CONFLICT (character_id) DO UPDATE SET corporation_id = 98000002",
        )
        .execute(&pool)
        .await
        .unwrap();
        set_waitlist(state, account(), input(42, Some(98000002)))
            .await
            .unwrap();
        let row: (Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT character_id, corporation_id FROM srp_waitlist WHERE waitlist_id = 42",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row, (None, Some(98000002)));
        assert_eq!(
            srp::waitlists::target(&pool, 42)
                .await
                .unwrap()
                .unwrap()
                .corporation_id,
            Some(98000002)
        );

        // With a character given, a corp nobody watches is still rejected
        let with_character = |waitlist_id, character_id, corporation_id| {
            Json(SetWaitlist {
                waitlist_id,
                character_id: Some(character_id),
                corporation_id: Some(corporation_id),
                rule: None,
            })
        };
        assert!(matches!(
            set_waitlist(state, account(), with_character(42, 2, 7)).await,
            Err(Madness::BadRequest(_))
        ));
        // Another connected wallet's corp is accepted, but the character's own corp still wins
        sqlx::query(
            "INSERT INTO srp_connection (character_id, corporation_id, corporation_name,
                refresh_token, connected_by, connected_at)
                VALUES (3, 98000003, 'OTHER', 't', 1, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        set_waitlist(state, account(), with_character(42, 2, 98000003))
            .await
            .unwrap();
        let character_id: Option<i64> =
            sqlx::query_scalar("SELECT character_id FROM srp_waitlist WHERE waitlist_id = 42")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(character_id, Some(2));
        assert_eq!(
            srp::waitlists::target(&pool, 42)
                .await
                .unwrap()
                .unwrap()
                .corporation_id,
            Some(98000002)
        );
        // Back to the bare corp for the disconnect case
        set_waitlist(state, account(), input(42, Some(98000002)))
            .await
            .unwrap();

        // The rule is saved with the assignment, and each bound is checked before anything is
        // written
        let ruled = |division, min_amount, coverage_hours| {
            Json(SetWaitlist {
                waitlist_id: 42,
                character_id: None,
                corporation_id: Some(98000002),
                rule: Some(RuleInput {
                    division,
                    min_amount,
                    coverage_hours,
                }),
            })
        };
        set_waitlist(state, account(), ruled(2, 75_000_000, 48))
            .await
            .unwrap();
        let saved: (i32, i64, i32) = sqlx::query_as(
            "SELECT division, min_amount, coverage_hours FROM srp_waitlist WHERE waitlist_id = 42",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(saved, (2, 75_000_000, 48));
        match set_waitlist(state, account(), ruled(8, 75_000_000, 48)).await {
            Err(Madness::BadRequest(m)) => assert!(m.contains("division"), "{}", m),
            other => panic!("expected rejection, got {:?}", other),
        }
        assert!(matches!(
            set_waitlist(state, account(), ruled(1, -1, 48)).await,
            Err(Madness::BadRequest(_))
        ));
        assert!(matches!(
            set_waitlist(state, account(), ruled(1, 75_000_000, 0)).await,
            Err(Madness::BadRequest(_))
        ));
        let after: (i32, i64, i32) = sqlx::query_as(
            "SELECT division, min_amount, coverage_hours FROM srp_waitlist WHERE waitlist_id = 42",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(after, saved);
        // Picking a wallet without a rule keeps the saved one
        set_waitlist(state, account(), input(42, Some(98000002)))
            .await
            .unwrap();
        let kept: (i32, i64, i32) = sqlx::query_as(
            "SELECT division, min_amount, coverage_hours FROM srp_waitlist WHERE waitlist_id = 42",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(kept, saved);

        // The current corp survives a disconnect (migration case) and can be re-saved
        sqlx::query("DELETE FROM srp_connection")
            .execute(&pool)
            .await
            .unwrap();
        set_waitlist(state, account(), input(42, Some(98000002)))
            .await
            .unwrap();
        // A different waitlist still rejects the now-disconnected corp
        assert!(matches!(
            set_waitlist(state, account(), input(43, Some(98000002))).await,
            Err(Madness::BadRequest(_))
        ));

        // A character with no wallet and no kept corp has no corp, rather than a decode error
        set_waitlist(
            state,
            account(),
            Json(SetWaitlist {
                waitlist_id: 42,
                character_id: Some(2),
                corporation_id: None,
                rule: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            srp::waitlists::target(&pool, 42)
                .await
                .unwrap()
                .unwrap()
                .corporation_id,
            None
        );

        for table in ["srp_waitlist", "waitlist", "character"] {
            sqlx::query(&format!("DELETE FROM {}", table))
                .execute(&pool)
                .await
                .unwrap();
        }
    }
    // The routes must tell an assigned account (the source, hidden when its wallet is gone) from
    // a corp-only migration row. Needs a second throwaway database on the same server, set up
    // like refute1 in the test above, named refute2.
    #[ignore = "needs a throwaway Postgres on 127.0.0.1:55432; see the comment"]
    #[rocket::async_test]
    async fn status_and_disconnect_with_shared_corp() {
        use crate::config::{
            AppConfig, Config, DatabaseConfig, ESIConfig, FleetUpdaterConfig, SSEConfig,
            SkillUpdaterConfig,
        };
        use std::sync::Arc;

        let url = "postgres://postgres:pass@127.0.0.1:55432/refute2";
        let pool = sqlx::Pool::<sqlx::Postgres>::connect(url).await.unwrap();
        // The fixture only needs these columns; a real schema has more
        for column in [
            "name varchar NOT NULL DEFAULT ''",
            "is_open boolean NOT NULL DEFAULT false",
            "is_archived boolean NOT NULL DEFAULT false",
        ] {
            sqlx::query(&format!(
                "ALTER TABLE waitlist ADD COLUMN IF NOT EXISTS {}",
                column
            ))
            .execute(&pool)
            .await
            .unwrap();
        }
        sqlx::query("CREATE TABLE IF NOT EXISTS alt_character (account_id int8, alt_id int8)")
            .execute(&pool)
            .await
            .unwrap();
        for table in ["srp_waitlist", "srp_connection", "waitlist", "character"] {
            sqlx::query(&format!("DELETE FROM {}", table))
                .execute(&pool)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO character (id, name) VALUES (1, 'a'), (2, 'b')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO waitlist (id, name, is_open, is_archived)
                VALUES (10, 'dead account', false, false), (11, 'no wallet', false, false),
                    (12, 'shared corp', false, false)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO srp_connection (character_id, corporation_id, corporation_name,
                refresh_token, connected_by, connected_at)
                VALUES (2, 101, 'Corp', 't', 1, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 10: assigned account 1, wallet gone, corp kept. 12: corp only. 11: corp only, no wallet.
        // 12 carries its own rule, which its status must report
        sqlx::query(
            "INSERT INTO srp_waitlist (waitlist_id, character_id, corporation_id, set_by, set_at,
                division, min_amount, coverage_hours)
                VALUES (10, 1, 101, 1, 1, 1, 50000000, 24), (12, NULL, 101, 1, 1, 2, 60000000, 36),
                    (11, NULL, 999, 1, 1, 1, 50000000, 24)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let config = Config {
            database: DatabaseConfig {
                url: url.to_string(),
                min_connections: 1,
                max_connections: 2,
                idle_timeout: 60,
                connect_timeout: 60,
            },
            app: AppConfig {
                token_secret: "00".repeat(32),
            },
            esi: ESIConfig {
                client_id: "x".to_string(),
                client_secret: "y".to_string(),
                url: "http://127.0.0.1".to_string(),
            },
            sse: SSEConfig {
                url: "http://127.0.0.1".to_string(),
                secret: "00".repeat(32),
            },
            fleet_updater: FleetUpdaterConfig {
                enable: false,
                min_in_fleet: 1,
            },
            skill_updater: SkillUpdaterConfig {
                enable: false,
                runtime: 1.0,
            },
        };
        let db: Arc<crate::DB> = Arc::new(pool.clone());
        let app = crate::app::new(db, config, srp::new_shared_state());
        let state = rocket::State::from(&app);
        let account = || AuthenticatedAccount {
            id: 1,
            access: crate::core::auth::get_access_keys("fc").unwrap(),
        };

        // The assigned account is the source: its disconnection hides the waitlist, even though
        // another account of the same corp is connected
        let answer = status(state, account(), "1", 10).await.unwrap().0;
        assert!(!answer.applies && !answer.available);
        let fleet = fleet_status(state, account(), "1", 10).await.unwrap().0;
        assert!(!fleet.applies);

        // A corp-only row is fed by any account of that corp
        let answer = status(state, account(), "1", 12).await.unwrap().0;
        assert!(answer.applies && !answer.available);
        // Its own rule, also while nothing can be shown
        assert_eq!((answer.min_amount, answer.coverage_hours), (60_000_000, 36));
        let fleet = fleet_status(state, account(), "1", 12).await.unwrap().0;
        assert!(fleet.applies && !fleet.available);

        // Admin resolves the corp-only row to the account that feeds it, and keeps the dead row
        let admin = admin_status(state, account()).await.unwrap().0;
        let row = |id: i64| admin.waitlists.iter().find(|w| w.id == id).unwrap();
        assert_eq!(row(10).character.as_ref().map(|c| c.id), Some(1));
        assert_eq!(row(12).character.as_ref().map(|c| c.id), Some(2));
        assert!(row(11).character.is_none());
        assert_eq!(
            (row(12).division, row(12).min_amount, row(12).coverage_hours),
            (2, 60_000_000, 36)
        );
        // The wallet shows a check for the division its waitlist reads, under that rule
        let wallet = admin.wallets.iter().find(|w| w.character.id == 2).unwrap();
        assert_eq!(wallet.checks.len(), 1);
        assert_eq!(wallet.checks[0].division, 2);
        assert!(wallet.checks[0].checked_at.is_none());
        assert_eq!(
            (wallet.checks[0].min_amount, wallet.checks[0].coverage_hours),
            (60_000_000, 36)
        );

        disconnect(state, account(), Json(Disconnect { character_id: 2 }))
            .await
            .unwrap();
        let admin = admin_status(state, account()).await.unwrap().0;
        assert!(admin
            .waitlists
            .iter()
            .find(|w| w.id == 12)
            .unwrap()
            .character
            .is_none());
        assert!(matches!(
            disconnect(state, account(), Json(Disconnect { character_id: 2 })).await,
            Err(Madness::BadRequest(_))
        ));

        for table in ["srp_waitlist", "srp_connection", "waitlist", "character"] {
            sqlx::query(&format!("DELETE FROM {}", table))
                .execute(&pool)
                .await
                .unwrap();
        }
    }
}
