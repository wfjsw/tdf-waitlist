import React from "react";
import styled from "styled-components";
import { AuthContext, ToastContext } from "../../contexts";
import { apiCall, errorToaster, toaster } from "../../api";
import { Button, Buttons, Input, Select } from "../../Components/Form";
import { Confirm } from "../../Components/Modal";
import { InfoNote } from "../../Components/NoteBox";
import { Content, PageTitle } from "../../Components/Page";
import { Cell, CellHead, Row, Table, TableBody, TableHead } from "../../Components/Table";
import { addToast } from "../../Components/Toast";
import { formatDatetime } from "../../Util/time";
import { formatAgo, formatClockUtc, formatIsk, formatSpan } from "../../Util/srp";

const POLL_MS = 60000;
// Right after connecting, the first check is due within a minute, so look more often until it lands
const CHECKING_POLL_MS = 5000;
const CHECKING_WINDOW_SECONDS = 10 * 60;
// The server checks the wallet every few minutes, so this long without a success means it is
// probably not running the check at all
const STALE_SECONDS = 15 * 60;

const Muted = styled.p`
  color: ${(props) => props.theme.colors.accent4};
`;

const Hint = styled.span`
  color: ${(props) => props.theme.colors.accent4};
  font-size: 0.9em;
`;

const TableScroll = styled.div`
  overflow-x: auto;
  max-width: 900px;
`;

const Details = styled.dl`
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 0.2em 1.2em;
  margin: 0.7em 0 1em;
  line-height: 1.5em;

  dt {
    color: ${(props) => props.theme.colors.accent4};
  }
  dd {
    margin: 0;
    word-break: break-word;
  }
`;

const Step = styled.section`
  margin: 1.2em 0;
`;

const StepHeading = styled.h3`
  margin: 0 0 0.3em;
`;

const StepNumber = styled.span`
  display: inline-block;
  min-width: 1.6em;
  margin-right: 0.5em;
  border-radius: 1em;
  background-color: ${(props) => props.theme.colors.secondary.color};
  color: ${(props) => props.theme.colors.secondary.text};
  font-size: 0.85em;
  line-height: 1.6em;
  text-align: center;
`;

// Groups the selects for assistive tech without a visible frame or caption: the step heading above
// is already the caption
const Fieldset = styled.fieldset`
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
`;

const HiddenLegend = styled.legend`
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
`;

const WalletBlock = styled.div`
  margin: 0.8em 0;
  padding: 0.7em 0.9em;
  border: 1px solid ${(props) => props.theme.colors.accent2};
  border-radius: 4px;
`;

const WalletHead = styled.div`
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 0 1em;
  margin-bottom: 0.5em;
  word-break: break-word;
`;

const Toggle = styled.summary`
  cursor: pointer;
  padding: 0.3em 0;
`;

const WaitlistCard = styled.div`
  margin: 0.8em 0;
  padding: 0.7em 0.9em;
  border: 1px solid ${(props) => props.theme.colors.accent2};
  border-radius: 4px;
  word-break: break-word;
`;

const WaitlistHead = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.3em 1em;

  > .name {
    flex: 1 1 12em;
    min-width: 0;
  }
  > .pick {
    flex: 0 1 18em;
    min-width: 0;
  }
  > .pick select {
    width: 100%;
  }
`;

// The two settings side by side on wide screens, stacked when there's no room
const WaitlistColumns = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 0.5em 1.5em;
  margin-top: 0.6em;
`;

const Group = styled.section`
  min-width: 0;
`;

const GroupTitle = styled.h4`
  margin: 0 0 0.3em;
  font-size: 0.85em;
  font-weight: 700;
  color: ${(props) => props.theme.colors.accent4};
`;

const RuleRow = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 0.5em 0.7em;

  input,
  select {
    width: 6.5em;
  }
`;

const Field = styled.label`
  display: flex;
  flex-direction: column;
  gap: 0.2em;

  > .label {
    font-size: 0.85em;
    color: ${(props) => props.theme.colors.accent4};
  }
  &.division select {
    width: 4.5em;
  }
`;

// Not InfoNote: its fixed "!" icon is wrong on the healthy state, and its white text fails contrast
// on the Light theme's green. The colour is only the edge; the glyph and words carry the state.
const Status = styled.div`
  width: fit-content;
  max-width: 100%;
  margin-bottom: 0.5em;
  padding: 0.4em 0.8em;
  border: 1px solid ${(props) => props.theme.colors.accent2};
  border-left: 4px solid ${(props) => props.theme.colors[props.variant].color};
  border-radius: 4px;
  background-color: ${(props) => props.theme.colors.background};
  color: ${(props) => props.theme.colors.text};
`;

// The warning sign in its plain text form: without U+FE0E some systems draw a coloured emoji
const WARN = "⚠︎";

const datetime = (unixSeconds) => formatDatetime(new Date(unixSeconds * 1000));

async function startConnect(characterId, waitlistId) {
  const url = await apiCall(
    `/api/srp/admin/connect?character_id=${characterId}&waitlist_id=${waitlistId}`,
    {}
  );
  window.location.href = url;
}

async function disconnect(wallet) {
  await apiCall("/api/srp/admin/disconnect", { json: { character_id: wallet.character.id } });
  return `${wallet.character.name || wallet.character.id} wallet disconnected`;
}

// Just connected: no check or error reported yet, and the connection is new enough that the first
// check is still expected
function isChecking(wallet, check, now) {
  return (
    !check.checked_at &&
    !check.last_error &&
    now / 1000 - wallet.connected_at < CHECKING_WINDOW_SECONDS
  );
}

// Loads the admin status and keeps it fresh while the tab is visible, so the status lines don't go
// stale while the page stays open. A failed refresh keeps what is on screen (only a failed first
// load leaves the page empty); `now` still moves on, so the "last check" age keeps growing and the
// stale warning can appear. A response is dropped if a newer request has started since, or if the
// page is gone.
function useSrpAdmin() {
  const toastContext = React.useContext(ToastContext);
  const [admin, setAdmin] = React.useState(null);
  const [loadError, setLoadError] = React.useState(null);
  const [now, setNow] = React.useState(() => Date.now());
  const mounted = React.useRef(false);
  const latest = React.useRef(0);

  const load = React.useCallback(
    (quiet) => {
      const id = ++latest.current;
      const request = apiCall("/api/srp/admin", {}).then(
        (data) => {
          if (mounted.current && id === latest.current) {
            setAdmin(data);
            setLoadError(null);
          }
        },
        (err) => {
          if (mounted.current && id === latest.current) {
            setLoadError(String(err));
            throw err;
          }
        }
      );
      const done = quiet ? request.catch(() => {}) : errorToaster(toastContext, request);
      return done.finally(() => {
        if (mounted.current) setNow(Date.now());
      });
    },
    [toastContext]
  );

  React.useEffect(() => {
    mounted.current = true;
    load(false);
    return () => {
      mounted.current = false;
    };
  }, [load]);

  const checking =
    admin &&
    admin.wallets.some((wallet) => wallet.checks.some((check) => isChecking(wallet, check, now)));
  const pollMs = checking ? CHECKING_POLL_MS : POLL_MS;
  React.useEffect(() => {
    const loadIfVisible = () => {
      if (document.visibilityState === "visible") load(true);
    };

    const timer = setInterval(loadIfVisible, pollMs);
    document.addEventListener("visibilitychange", loadIfVisible);
    return () => {
      clearInterval(timer);
      document.removeEventListener("visibilitychange", loadIfVisible);
    };
  }, [load, pollMs]);

  const refresh = React.useCallback(() => load(false), [load]);
  return [admin, refresh, now, loadError];
}

// One line per wallet division that says whether the check works. Every state carries a glyph or
// a word as well as a colour.
function StatusLine({ wallet, check, now }) {
  const checkedAt = check.checked_at;
  const age = checkedAt ? now / 1000 - checkedAt : null;

  let variant = "success";
  let text;
  if (check.last_error) {
    variant = "danger";
    text = `✕ Not working: ${check.last_error}`;
  } else if (isChecking(wallet, check, now)) {
    variant = "secondary";
    text = "◷ Checking the wallet now. The first check takes up to a minute.";
  } else if (age === null) {
    variant = "warning";
    text = `${WARN} No successful check yet. The server may not be running the SRP check.`;
  } else if (age > STALE_SECONDS) {
    variant = "warning";
    const span = formatSpan(age);
    text = `${WARN} No successful check for ${span}. The server may not be running the SRP check.`;
  } else {
    text = `✓ Working · last check ${formatClockUtc(new Date(checkedAt * 1000))} (${formatAgo(
      checkedAt,
      now
    )})`;
  }

  return <Status variant={variant}>{text}</Status>;
}

// Warning about SRP not being shown anywhere
function ConfigNotes({ admin }) {
  // With no wallets or no active waitlists at all, the steps below say so themselves
  const none =
    admin.wallets.length > 0 &&
    admin.waitlists.length > 0 &&
    !admin.waitlists.some((w) => w.character !== null || w.corporation_id !== null);
  return (
    <>
      {none && (
        <InfoNote variant="warning">
          <span>
            {WARN} SRP isn't shown on any waitlist yet. Pick a wallet for the waitlists below.
          </span>
        </InfoNote>
      )}
    </>
  );
}

function RecentPayments({ check }) {
  const recent = check.recent;
  return (
    <details>
      <Toggle>Recent payments ({recent.length})</Toggle>
      {recent.length === 0 ? (
        <p style={{ margin: "0.5em 0" }}>
          <em>
            No payments of {formatIsk(check.min_amount)}+ in the last{" "}
            {formatSpan(check.coverage_hours * 3600)}.
          </em>
        </p>
      ) : (
        <TableScroll>
          <Table fullWidth style={{ minWidth: "560px" }}>
            <TableHead>
              <Row>
                <CellHead>Character</CellHead>
                <CellHead>Amount</CellHead>
                <CellHead>Paid</CellHead>
                <CellHead>Covered until</CellHead>
                <CellHead>How</CellHead>
              </Row>
            </TableHead>
            <TableBody>
              {recent.map((payment) => (
                <Row key={`${payment.character.id}-${payment.paid_at}`}>
                  <Cell>{payment.character.name || payment.character.id}</Cell>
                  <Cell>{formatIsk(payment.amount)}</Cell>
                  <Cell>{datetime(payment.paid_at)}</Cell>
                  <Cell>{datetime(payment.expires_at)}</Cell>
                  <Cell>{payment.via === "contract" ? "Handout contract" : "Donation"}</Cell>
                </Row>
              ))}
            </TableBody>
          </Table>
        </TableScroll>
      )}
    </details>
  );
}

function Wallet({ admin, wallet, now, onChange }) {
  const toastContext = React.useContext(ToastContext);
  const [disconnectOpen, setDisconnectOpen] = React.useState(false);
  const [disconnecting, setDisconnecting] = React.useState(false);
  const uses = admin.waitlists.filter((w) => {
    // An assigned account is the source; only a corp-only row is fed by any account of its corp
    if (w.character) return w.character.id === wallet.character.id;
    return w.corporation_id !== null && w.corporation_id === wallet.corporation_id;
  }).length;

  const confirmDisconnect = () => {
    if (disconnecting) return;
    setDisconnecting(true);
    toaster(toastContext, disconnect(wallet)).finally(() => {
      setDisconnecting(false);
      setDisconnectOpen(false);
      onChange();
    });
  };

  return (
    <WalletBlock>
      <WalletHead>
        <strong>{wallet.character.name || wallet.character.id}</strong>
        <Hint>
          wallet of {wallet.corporation_name} · used by {uses}{" "}
          {uses === 1 ? "waitlist" : "waitlists"}
        </Hint>
      </WalletHead>
      <Details>
        <dt>Corporation</dt>
        <dd>{wallet.corporation_name}</dd>
        <dt>Connected by</dt>
        <dd>
          {wallet.connected_by.name} · {datetime(wallet.connected_at)}
        </dd>
      </Details>
      {wallet.checks.length === 0 ? (
        <Hint>The wallet isn't used by a waitlist yet, so it isn't checked.</Hint>
      ) : (
        wallet.checks.map((check) => (
          <section key={check.division}>
            <h4 style={{ margin: "0.5em 0 0.3em" }}>Wallet division {check.division}</h4>
            <StatusLine wallet={wallet} check={check} now={now} />
            <RecentPayments check={check} />
          </section>
        ))
      )}
      <Buttons style={{ marginTop: "0.5em" }}>
        <Button variant="danger" disabled={disconnecting} onClick={() => setDisconnectOpen(true)}>
          {disconnecting ? "Disconnecting…" : "Disconnect"}
        </Button>
      </Buttons>
      <Confirm
        open={disconnectOpen}
        setOpen={setDisconnectOpen}
        title={`Disconnect the ${wallet.character.name || wallet.character.id} wallet`}
        onConfirm={confirmDisconnect}
      >
        <p style={{ margin: "0.5em 0 1em" }}>
          Disconnect the {wallet.character.name || wallet.character.id} wallet? Waitlists that have
          no other account connected for their corp stop showing SRP until it's reconnected.
        </p>
      </Confirm>
    </WalletBlock>
  );
}

function Wallets({ admin, now, onChange }) {
  return (
    <Step aria-labelledby="srp-step-wallets">
      <StepHeading id="srp-step-wallets">
        <StepNumber aria-hidden="true">1</StepNumber>SRP wallets
      </StepHeading>
      <Hint>
        Accounts connected for a waitlist below. Each waitlist has its own, so two waitlists can
        share a corp with separate accounts. Use Disconnect here; connect from the waitlist the
        account should pay for.
      </Hint>
      {admin.wallets.length === 0 ? (
        <Muted style={{ margin: "0.8em 0" }}>
          No SRP wallet connected yet; connect one from the waitlist it should pay for.
        </Muted>
      ) : (
        admin.wallets.map((wallet) => (
          <Wallet
            key={wallet.character.id}
            admin={admin}
            wallet={wallet}
            now={now}
            onChange={onChange}
          />
        ))
      )}
    </Step>
  );
}

async function setWaitlist(waitlistId, characterId, corporationId, rule) {
  const json = {
    waitlist_id: waitlistId,
    character_id: characterId,
    corporation_id: corporationId,
  };
  if (rule) json.rule = rule;
  await apiCall("/api/srp/admin/waitlist", { json });
}

// Connects one of the account's characters as the wallet of this waitlist: the server assigns it
// when the login comes back
function ConnectForWaitlist({ waitlist }) {
  const authContext = React.useContext(AuthContext);
  const [picked, setPicked] = React.useState(null);
  const [startError, setStartError] = React.useState(null);
  const [connecting, setConnecting] = React.useState(false);

  const characters = authContext.characters;
  const selected = picked === null ? authContext.current.id : picked;

  const connect = () => {
    setStartError(null);
    setConnecting(true);
    startConnect(selected, waitlist.id).catch((err) => {
      setStartError(err.toString());
      setConnecting(false);
    });
  };

  return (
    <details>
      <Toggle>Connect an account for {waitlist.name}</Toggle>
      <Buttons style={{ marginTop: "0.4em" }}>
        <Select
          aria-label={`Wallet character for ${waitlist.name}`}
          value={selected}
          disabled={connecting}
          onChange={(evt) => setPicked(parseInt(evt.target.value, 10))}
        >
          {characters.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </Select>
        <Button
          variant="primary"
          disabled={connecting}
          aria-label={`Connect for ${waitlist.name}`}
          onClick={connect}
        >
          {connecting ? "Connecting…" : "Connect"}
        </Button>
      </Buttons>
      {startError && <InfoNote variant="danger">{startError}</InfoNote>}
    </details>
  );
}

// The rule the waitlist applies: wallet division, minimum amount (typed in millions of ISK) and
// coverage hours. Its own button saves, so half-typed numbers aren't sent. The assignment goes
// along unchanged, so a save here can't drop the wallet.
function RuleForm({ waitlist, onChange }) {
  const toastContext = React.useContext(ToastContext);
  const [division, setDivision] = React.useState(String(waitlist.division));
  const [millions, setMillions] = React.useState(String(waitlist.min_amount / 1_000_000));
  const [hours, setHours] = React.useState(String(waitlist.coverage_hours));
  const [saving, setSaving] = React.useState(false);

  // The server's values win after a save or a refresh from elsewhere. Compared rather than reset
  // on every effect run, so a late first flush can't overwrite what the admin already typed.
  const saved = React.useRef(
    `${waitlist.division}/${waitlist.min_amount}/${waitlist.coverage_hours}`
  );
  React.useEffect(() => {
    const current = `${waitlist.division}/${waitlist.min_amount}/${waitlist.coverage_hours}`;
    if (current !== saved.current) {
      saved.current = current;
      setDivision(String(waitlist.division));
      setMillions(String(waitlist.min_amount / 1_000_000));
      setHours(String(waitlist.coverage_hours));
    }
  }, [waitlist.division, waitlist.min_amount, waitlist.coverage_hours]);

  const values = {
    division: Number(division),
    min_amount: Math.round(Number(millions) * 1_000_000),
    coverage_hours: Number(hours),
  };
  const valid =
    Number.isInteger(values.division) &&
    values.division >= 1 &&
    values.division <= 7 &&
    Number.isFinite(values.min_amount) &&
    values.min_amount >= 0 &&
    Number.isInteger(values.coverage_hours) &&
    values.coverage_hours >= 1 &&
    values.coverage_hours <= 720;
  const unchanged =
    values.division === waitlist.division &&
    values.min_amount === waitlist.min_amount &&
    values.coverage_hours === waitlist.coverage_hours;

  const save = () => {
    setSaving(true);
    const request = setWaitlist(
      waitlist.id,
      waitlist.character ? waitlist.character.id : null,
      waitlist.corporation_id,
      values
    ).then(() =>
      addToast(toastContext, {
        message: `${waitlist.name} now counts ${formatIsk(values.min_amount)}+ in division ${
          values.division
        }, covering ${formatSpan(values.coverage_hours * 3600)}`,
        variant: "success",
      })
    );
    errorToaster(toastContext, request)
      .then(onChange)
      .finally(() => setSaving(false));
  };

  return (
    <RuleRow>
      <Field className="division">
        <span className="label">Division</span>
        <Select
          value={division}
          disabled={saving}
          aria-label={`Wallet division for ${waitlist.name}`}
          onChange={(evt) => setDivision(evt.target.value)}
        >
          {[1, 2, 3, 4, 5, 6, 7].map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </Select>
      </Field>
      <Field>
        <span className="label">Min (M ISK)</span>
        <Input
          type="number"
          min="0"
          step="0.1"
          value={millions}
          disabled={saving}
          aria-label={`Minimum amount in millions of ISK for ${waitlist.name}`}
          onChange={(evt) => setMillions(evt.target.value)}
        />
      </Field>
      <Field>
        <span className="label">Cover (hours)</span>
        <Input
          type="number"
          min="1"
          max="720"
          step="1"
          value={hours}
          disabled={saving}
          aria-label={`Coverage in hours for ${waitlist.name}`}
          onChange={(evt) => setHours(evt.target.value)}
        />
      </Field>
      <Button
        variant="primary"
        disabled={saving || !valid || unchanged}
        aria-label={`Save rule for ${waitlist.name}`}
        onClick={save}
      >
        {saving ? "Saving…" : "Save rule"}
      </Button>
    </RuleRow>
  );
}

// Picking a wallet saves straight away. Each select is disabled while its own request runs, and the
// value always comes from the server's answer, so a failed save leaves it as it was.
function WaitlistPicker({ admin, onChange }) {
  const toastContext = React.useContext(ToastContext);
  const [pending, setPending] = React.useState([]);
  const { waitlists, wallets } = admin;

  const pick = (waitlist, value) => {
    // A corp fallback has no account to pick: "corp:<id>"
    const pendingCorp = value.startsWith("corp:");
    const characterId = value === "" || pendingCorp ? null : parseInt(value, 10);
    const wallet = wallets.find((w) => w.character.id === characterId);
    const kept = characterId !== null ? waitlist.character : null;
    const corporationId = pendingCorp
      ? parseInt(value.slice(5), 10)
      : wallet
      ? wallet.corporation_id
      : kept
      ? waitlist.corporation_id
      : null;
    setPending((ids) => [...ids, waitlist.id]);
    const request = setWaitlist(waitlist.id, characterId, corporationId).then(() =>
      addToast(toastContext, {
        message: wallet
          ? `${waitlist.name} now uses ${wallet.character.name || wallet.character.id} (${
              wallet.corporation_name
            })`
          : kept
          ? `${waitlist.name} keeps ${kept.name || kept.id}`
          : pendingCorp
          ? `${waitlist.name} waits for a wallet of corp ${corporationId}`
          : `SRP removed from ${waitlist.name}`,
        variant: "success",
      })
    );
    errorToaster(toastContext, request)
      .then(onChange)
      .finally(() => setPending((ids) => ids.filter((id) => id !== waitlist.id)));
  };

  return (
    <Step aria-labelledby="srp-step-waitlists">
      <StepHeading id="srp-step-waitlists">
        <StepNumber aria-hidden="true">2</StepNumber>Waitlists
      </StepHeading>
      <Hint>
        {wallets.length === 0
          ? "Connect an account in a waitlist's row below to make it that waitlist's wallet."
          : "Connect an account in a waitlist's row to make it that waitlist's wallet, or pick an already connected one. Pilots are covered by payments to that account's corp only. Each waitlist also has its own rule: division, minimum amount and cover. Wallet changes save straight away; pilots see them next time they open the waitlist."}
      </Hint>
      {waitlists.length === 0 ? (
        <p style={{ margin: "0.8em 0" }}>
          <em>There are no active waitlists.</em>
        </p>
      ) : (
        <Fieldset style={{ marginTop: "0.5em" }}>
          <HiddenLegend>SRP wallet for each waitlist</HiddenLegend>
          {waitlists.map((waitlist) => {
            const characterId = waitlist.character ? waitlist.character.id : null;
            const pendingCorp = waitlist.corporation_id;
            // A migrated waitlist kept only a corp: any connected account of that corp is what
            // feeds it, so show that
            const wallet = wallets.find((w) =>
              characterId !== null
                ? w.character.id === characterId
                : pendingCorp !== null && w.corporation_id === pendingCorp
            );
            // Its wallet was disconnected after the waitlist picked it, or the migration kept a
            // corp it used with no account left
            const orphaned = (characterId !== null || pendingCorp !== null) && !wallet;
            // Re-selecting a still-listed orphan keeps it: a character by id, a pending corp
            // with the "corp:" prefix (the id alone would be a character)
            const orphanValue = characterId !== null ? String(characterId) : `corp:${pendingCorp}`;
            const effectiveId = characterId !== null ? characterId : wallet?.character.id;
            const value =
              effectiveId !== undefined && effectiveId !== null
                ? String(effectiveId)
                : orphaned
                ? orphanValue
                : "";
            const orphanName = waitlist.character
              ? waitlist.character.name || characterId
              : `Corp ${pendingCorp}`;
            const byId = `srp-wl-${waitlist.id}-by`;
            const showBy =
              (waitlist.character || waitlist.corporation_id) && waitlist.set_by && waitlist.set_at;
            return (
              <WaitlistCard key={waitlist.id}>
                <WaitlistHead>
                  <label className="name" htmlFor={`srp-wl-${waitlist.id}`}>
                    {waitlist.name}
                  </label>
                  <div className="pick">
                    <Select
                      id={`srp-wl-${waitlist.id}`}
                      value={value}
                      disabled={pending.includes(waitlist.id)}
                      aria-describedby={showBy ? byId : undefined}
                      onChange={(evt) => pick(waitlist, evt.target.value)}
                    >
                      <option value="">No SRP</option>
                      {wallets.map((w) => (
                        <option key={w.character.id} value={String(w.character.id)}>
                          {w.character.name || w.character.id} ({w.corporation_name})
                        </option>
                      ))}
                      {orphaned && (
                        <option value={orphanValue}>{orphanName} (wallet disconnected)</option>
                      )}
                      {pendingCorp !== null && (characterId !== null || !orphaned) && (
                        <option value={`corp:${pendingCorp}`}>
                          Any wallet of corp {pendingCorp}
                        </option>
                      )}
                    </Select>
                  </div>
                </WaitlistHead>
                {showBy && (
                  <Hint as="div" id={byId} style={{ marginTop: "0.3em" }}>
                    by {waitlist.set_by.name} · {datetime(waitlist.set_at)}
                  </Hint>
                )}
                {orphaned && (
                  <Status variant="warning" style={{ marginTop: "0.5em", marginBottom: 0 }}>
                    {WARN} Its wallet isn't connected, so pilots see nothing.
                  </Status>
                )}
                <WaitlistColumns>
                  <Group aria-labelledby={`srp-wl-${waitlist.id}-account`}>
                    <GroupTitle id={`srp-wl-${waitlist.id}-account`}>Wallet account</GroupTitle>
                    <ConnectForWaitlist waitlist={waitlist} />
                  </Group>
                  <Group aria-labelledby={`srp-wl-${waitlist.id}-rule`}>
                    <GroupTitle id={`srp-wl-${waitlist.id}-rule`}>Payments counted</GroupTitle>
                    <RuleForm waitlist={waitlist} onChange={onChange} />
                  </Group>
                </WaitlistColumns>
              </WaitlistCard>
            );
          })}
        </Fieldset>
      )}
    </Step>
  );
}

// The rule a new waitlist starts with, plus the time zone of every time on this page
function Footnote({ admin }) {
  return (
    <Muted style={{ marginTop: "1.5em", fontSize: "0.9em" }}>
      New waitlists start at {formatIsk(admin.defaults.min_amount)}+ from wallet division{" "}
      {admin.defaults.division}, covering the paying character for{" "}
      {formatSpan(admin.defaults.coverage_hours * 3600)}; each waitlist's rule can be changed on its
      card. Times are EVE time (UTC).
    </Muted>
  );
}

export function SrpWallet() {
  const [admin, refresh, now, loadError] = useSrpAdmin();

  return (
    <>
      <PageTitle>SRP wallet</PageTitle>
      <Content style={{ maxWidth: "900px" }}>
        {!admin ? (
          <>
            <em>Loading SRP wallet...</em>
            {loadError && (
              <InfoNote variant="danger">
                Couldn't load the SRP wallet page: {loadError}. If SRP was just added, apply its
                database migration first.
              </InfoNote>
            )}
          </>
        ) : (
          <>
            {admin.connect_error && <InfoNote variant="danger">{admin.connect_error}</InfoNote>}
            <ConfigNotes admin={admin} />
            <Wallets admin={admin} now={now} onChange={refresh} />
            <WaitlistPicker admin={admin} onChange={refresh} />
            <Footnote admin={admin} />
          </>
        )}
      </Content>
    </>
  );
}
