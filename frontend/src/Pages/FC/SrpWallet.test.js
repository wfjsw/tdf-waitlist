import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import { ThemeProvider } from "styled-components";
import theme from "../../App/theme";
import { AuthContext, ToastContext } from "../../contexts";

const mockApiCall = jest.fn();
jest.mock("../../api", () => ({
  apiCall: (...args) => mockApiCall(...args),
  errorToaster: (ctx, p) => Promise.resolve(p).catch(() => {}),
  toaster: (ctx, p) => Promise.resolve(p).catch(() => {}),
}));
jest.mock("../../Components/Toast", () => ({ addToast: jest.fn() }));

// Import after the mocks are registered
const { SrpWallet } = require("./SrpWallet");

const NOW = Math.floor(Date.now() / 1000);

function adminResponse(overrides = {}) {
  return {
    defaults: { division: 1, min_amount: 50000000, coverage_hours: 24 },
    connect_error: null,
    wallets: [],
    waitlists: [],
    ...overrides,
  };
}

function wallet(id, name, corp) {
  return {
    character: { id, name },
    corporation_id: corp,
    corporation_name: `Corp ${corp}`,
    connected_by: { id: 90, name: "Admin" },
    connected_at: NOW - 60,
    checks: [
      {
        division: 1,
        min_amount: 50000000,
        coverage_hours: 24,
        checked_at: NOW - 10,
        last_error: null,
        recent: [],
      },
    ],
  };
}

function waitlist(id, name, character, corporationId = null, rule = {}) {
  return {
    id,
    name,
    character,
    corporation_id: corporationId,
    division: 1,
    min_amount: 50000000,
    coverage_hours: 24,
    set_by: { id: 90, name: "Admin" },
    set_at: NOW - 60,
    ...rule,
  };
}

function renderPage() {
  return render(
    <ThemeProvider theme={theme.Light}>
      <ToastContext.Provider value={() => {}}>
        <AuthContext.Provider
          value={{
            current: { id: 1, name: "Alpha" },
            characters: [
              { id: 1, name: "Alpha" },
              { id: 3, name: "Beta" },
            ],
            access: { "srp-manage": true },
          }}
        >
          <SrpWallet />
        </AuthContext.Provider>
      </ToastContext.Provider>
    </ThemeProvider>
  );
}

beforeEach(() => {
  mockApiCall.mockReset();
});

test("renders an empty page without crashing", async () => {
  mockApiCall.mockResolvedValue(adminResponse());
  renderPage();
  expect(await screen.findByText(/No SRP wallet connected yet/i)).toBeInTheDocument();
  expect(screen.getByText(/There are no active waitlists/i)).toBeInTheDocument();
});

test("two wallets in the same corp are both offered per waitlist", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9), wallet(3, "Beta", 9)],
      waitlists: [waitlist(10, "WL one", null)],
    })
  );
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  const labels = [...select.options].map((o) => o.textContent);
  expect(labels).toEqual(["No SRP", "Alpha (Corp 9)", "Beta (Corp 9)"]);
});

test("orphaned assignment shows a disconnected option and warning", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9)],
      waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" })],
    })
  );
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  expect(select.value).toBe("99");
  const labels = [...select.options].map((o) => o.textContent);
  expect(labels).toContain("Gone (wallet disconnected)");
  expect(screen.getByText(/Its wallet isn't connected/i)).toBeInTheDocument();
});

test("picking a wallet saves by character id", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9), wallet(3, "Beta", 8)],
          waitlists: [waitlist(10, "WL one", null)],
        })
      );
    return Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("WL one");
  fireEvent.change(container.querySelector("#srp-wl-10"), { target: { value: "3" } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: 3, corporation_id: 8 },
    })
  );
});

test("an open per-waitlist connect form survives a poll refresh", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", null)],
        })
      );
    return Promise.resolve("https://seat.example/authorize");
  });
  renderPage();
  await screen.findByText("WL one");
  const summary = screen.getByText(/^Connect an account for /);
  const details = summary.closest("details");
  fireEvent.click(summary);
  expect(details.open).toBe(true);

  fireEvent(document, new Event("visibilitychange"));
  await waitFor(() => expect(mockApiCall.mock.calls.length).toBeGreaterThan(1));
  await waitFor(() => expect(screen.getByText("WL one")).toBeInTheDocument());
  expect(summary.closest("details")).toBe(details);
  expect(details.open).toBe(true);
});

test("a failed save reverts the picker and clears its pending state", async () => {
  let fail = false;
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9), wallet(3, "Beta", 9)],
          waitlists: [waitlist(10, "WL one", null)],
        })
      );
    return fail ? Promise.reject("That wallet isn't connected") : Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  fail = true;
  fireEvent.change(select, { target: { value: "3" } });
  await waitFor(() => expect(select.disabled).toBe(true));
  await waitFor(() => expect(select.disabled).toBe(false));
  expect(select.value).toBe("");
});

test("disconnect sends the wallet character id", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(adminResponse({ wallets: [wallet(1, "Alpha", 9)], waitlists: [] }));
    return Promise.resolve("OK");
  });
  renderPage();
  fireEvent.click(await screen.findByText("Disconnect"));
  fireEvent.click(screen.getByText("Yes"));
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/disconnect", {
      json: { character_id: 1 },
    })
  );
});

test("per-waitlist connect passes the waitlist id", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", null)],
        })
      );
    return Promise.resolve("https://seat.example/authorize");
  });
  renderPage();
  await screen.findByText("WL one");
  fireEvent.click(screen.getByText(/^Connect an account for /));
  const details = screen.getByText(/^Connect an account for /).closest("details");
  fireEvent.click(within(details).getByText("Connect"));
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith(
      "/api/srp/admin/connect?character_id=1&waitlist_id=10",
      {}
    )
  );
});

test("axe scan of the populated page", async () => {
  const axe = require("axe-core");
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9), wallet(3, "Beta", 9)],
      waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" }), waitlist(11, "WL two", null)],
    })
  );
  const { container } = renderPage();
  await screen.findByText("WL one");
  const results = await axe.run(container, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] },
  });
  expect(results.violations.map((v) => v.id)).toEqual([]);
});

test("a failed connect start shows the error in the row form", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", null)],
        })
      );
    return Promise.reject("The character you picked isn't on the SeAT account you logged in with.");
  });
  renderPage();
  await screen.findByText("WL one");
  fireEvent.click(screen.getByText(/^Connect an account for /));
  const details = screen.getByText(/^Connect an account for /).closest("details");
  fireEvent.click(within(details).getByText("Connect"));
  expect(await within(details).findByText(/isn't on the SeAT account/i)).toBeInTheDocument();
});

test("a failed first load explains itself", async () => {
  mockApiCall.mockRejectedValue("Internal Server Error");
  renderPage();
  expect(await screen.findByText(/Couldn't load the SRP wallet page/i)).toBeInTheDocument();
});

test("the server connect error is shown on load", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({ connect_error: "SeAT didn't return a refresh token.", wallets: [] })
  );
  renderPage();
  expect(await screen.findByText(/SeAT didn't return a refresh token/i)).toBeInTheDocument();
});

test("top warning is suppressed when every assignment is orphaned", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9)],
      waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" })],
    })
  );
  renderPage();
  await screen.findByText("WL one");
  // The orphaned assignment still counts as SRP being shown somewhere
  expect(screen.queryByText(/SRP isn't shown on any waitlist yet/i)).toBeNull();
});

test("picking No SRP sends character_id null", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", { id: 1, name: "Alpha" })],
        })
      );
    return Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("WL one");
  fireEvent.change(container.querySelector("#srp-wl-10"), { target: { value: "" } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: null, corporation_id: null },
    })
  );
});

test("the disconnected-character option is the selected option", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9)],
      waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" })],
    })
  );
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  const orphan = [...select.options].find((o) => o.textContent.includes("wallet disconnected"));
  expect(select.value).toBe("99");
  expect(orphan.value).toBe("99");
});

test("picking the disconnected-character option keeps the character", async () => {
  const { addToast } = require("../../Components/Toast");
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" })],
        })
      );
    return Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  const orphan = [...select.options].find((o) => o.textContent.includes("wallet disconnected"));
  addToast.mockClear();
  fireEvent.change(select, { target: { value: orphan.value } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: 99, corporation_id: null },
    })
  );
  expect(addToast).toHaveBeenCalledWith(
    expect.anything(),
    expect.objectContaining({
      message: "WL one keeps Gone",
    })
  );
});

test("a migrated pending corp with no wallet shows the corp and keeps it when picked", async () => {
  const { addToast } = require("../../Components/Toast");
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({ wallets: [], waitlists: [waitlist(10, "Corp WL", null, 5)] })
      );
    return Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("Corp WL");
  const select = container.querySelector("#srp-wl-10");
  const orphan = [...select.options].find((o) => o.textContent.includes("wallet disconnected"));
  expect(select.value).toBe("corp:5");
  expect(orphan.value).toBe("corp:5");
  addToast.mockClear();
  fireEvent.change(select, { target: { value: orphan.value } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: null, corporation_id: 5 },
    })
  );
  expect(addToast).toHaveBeenCalledWith(
    expect.anything(),
    expect.objectContaining({
      message: "Corp WL waits for a wallet of corp 5",
    })
  );
});

test("a dead character with a kept corp still offers the corp fallback", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 5)],
          waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" }, 5)],
        })
      );
    return Promise.resolve("OK");
  });
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  const corp = [...select.options].find((o) => o.textContent.includes("Any wallet of corp"));
  expect(corp).toBeTruthy();
  // Its value differs from the selected "99", so the change really fires
  fireEvent.change(select, { target: { value: corp.value } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: null, corporation_id: 5 },
    })
  );
});

test("a pending corp with a connected wallet of that corp uses that wallet", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 5)],
      waitlists: [waitlist(10, "Corp WL", null, 5)],
    })
  );
  const { container } = renderPage();
  await screen.findByText("Corp WL");
  const select = container.querySelector("#srp-wl-10");
  expect(select.value).toBe("1");
  expect(screen.queryByText(/Its wallet isn't connected/i)).toBeNull();
  // The corp fallback can still be picked back
  fireEvent.change(select, { target: { value: "corp:5" } });
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: { waitlist_id: 10, character_id: null, corporation_id: 5 },
    })
  );
});

test("per-waitlist disclosures have distinct accessible names", async () => {
  const { computeAccessibleName } = require("dom-accessibility-api");
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9)],
      waitlists: [
        waitlist(10, "WL one", null),
        waitlist(11, "WL two", null),
        waitlist(12, "WL three", null),
      ],
    })
  );
  renderPage();
  await screen.findByText("WL one");
  const names = screen
    .getAllByText(/^Connect an account for /)
    .map((el) => computeAccessibleName(el));
  expect(new Set(names).size).toBe(names.length);
});

test("saving a rule sends it with the assignment unchanged", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({
          wallets: [wallet(1, "Alpha", 9)],
          waitlists: [waitlist(10, "WL one", { id: 1, name: "Alpha" })],
        })
      );
    return Promise.resolve("OK");
  });
  renderPage();
  await screen.findByText("WL one");
  fireEvent.change(screen.getByLabelText("Wallet division for WL one"), {
    target: { value: "3" },
  });
  fireEvent.change(screen.getByLabelText(/Minimum amount in millions of ISK for WL one/), {
    target: { value: "75" },
  });
  fireEvent.change(screen.getByLabelText("Coverage in hours for WL one"), {
    target: { value: "48" },
  });
  fireEvent.click(screen.getByLabelText("Save rule for WL one"));
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: {
        waitlist_id: 10,
        character_id: 1,
        corporation_id: null,
        rule: { division: 3, min_amount: 75000000, coverage_hours: 48 },
      },
    })
  );
});

test("an out-of-range or unchanged rule can't be saved", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({ wallets: [], waitlists: [waitlist(10, "WL one", null)] })
  );
  renderPage();
  await screen.findByText("WL one");
  const save = screen.getByLabelText("Save rule for WL one");
  expect(save.disabled).toBe(true);
  const hours = screen.getByLabelText("Coverage in hours for WL one");
  fireEvent.change(hours, { target: { value: "0" } });
  expect(save.disabled).toBe(true);
  fireEvent.change(hours, { target: { value: "5" } });
  expect(save.disabled).toBe(false);
  fireEvent.change(screen.getByLabelText("Wallet division for WL one"), {
    target: { value: "8" },
  });
  expect(save.disabled).toBe(true);
});

test("a config-only save on a waitlist without SRP keeps it that way", async () => {
  mockApiCall.mockImplementation((path) => {
    if (path === "/api/srp/admin")
      return Promise.resolve(
        adminResponse({ wallets: [], waitlists: [waitlist(10, "WL one", null)] })
      );
    return Promise.resolve("OK");
  });
  renderPage();
  await screen.findByText("WL one");
  fireEvent.change(screen.getByLabelText(/Minimum amount in millions of ISK for WL one/), {
    target: { value: "80" },
  });
  fireEvent.click(screen.getByLabelText("Save rule for WL one"));
  await waitFor(() =>
    expect(mockApiCall).toHaveBeenCalledWith("/api/srp/admin/waitlist", {
      json: {
        waitlist_id: 10,
        character_id: null,
        corporation_id: null,
        rule: { division: 1, min_amount: 80000000, coverage_hours: 24 },
      },
    })
  );
});

test("a wallet shows one status line per division a waitlist reads", async () => {
  const both = wallet(1, "Alpha", 9);
  both.checks = [
    { ...both.checks[0], division: 1 },
    { ...both.checks[0], division: 2, checked_at: null },
  ];
  mockApiCall.mockResolvedValue(adminResponse({ wallets: [both], waitlists: [] }));
  renderPage();
  await waitFor(() => expect(screen.getByText("Wallet division 1")).toBeInTheDocument());
  expect(screen.getByText("Wallet division 2")).toBeInTheDocument();
  // Just connected, so the missing check on division 2 reads as still being checked
  expect(screen.getByText(/Checking the wallet now/)).toBeInTheDocument();
  expect(screen.getAllByText(/Working · last check/)).toHaveLength(1);
});

test("disconnected character row selected index", async () => {
  mockApiCall.mockResolvedValue(
    adminResponse({
      wallets: [wallet(1, "Alpha", 9)],
      waitlists: [waitlist(10, "WL one", { id: 99, name: "Gone" })],
    })
  );
  const { container } = renderPage();
  await screen.findByText("WL one");
  const select = container.querySelector("#srp-wl-10");
  expect(select.value).toBe("99");
  expect(select.options[select.selectedIndex].textContent).toContain("wallet disconnected");
});
