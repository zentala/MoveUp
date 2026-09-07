/**
 * PairScreen.test.tsx — the four shadow paths of the pairing form (E022-T09).
 *
 * happy: a deep link pairs on its own and stores the record.
 * nil:   `#/pair` with no parameters shows the manual form and sends nothing.
 * empty: an empty code is refused before a request is made.
 * error: `bad_code` clears the field, `pairing_locked` names the retry time.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { PairScreen } from "./PairScreen";
import { RELAY_STORAGE_KEY, loadRelayRecord, saveRelayRecord } from "./storage";

const okResponse = (body: unknown) =>
  ({ ok: true, status: 201, json: async () => body }) as Response;

const errResponse = (status: number, error: unknown) =>
  ({ ok: false, status, json: async () => ({ error }) }) as Response;

describe("PairScreen", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("pairs on its own from a deep link and stores the record", async () => {
    vi.mocked(fetch).mockResolvedValue(
      okResponse({ viewer_id: "v1", viewer_token: "mu_v_tok", desk_name: "Studio" }),
    );
    const onPaired = vi.fn();

    render(
      <PairScreen
        hash="#/pair?d=desk-42&c=ABCD2345&r=https://relay.test"
        onPaired={onPaired}
      />,
    );

    await waitFor(() => expect(onPaired).toHaveBeenCalledOnce());

    const [url, init] = vi.mocked(fetch).mock.calls[0] as [string, RequestInit];
    expect(url).toBe("https://relay.test/v1/pair");
    expect(JSON.parse(init.body as string)).toMatchObject({
      desk_id: "desk-42",
      code: "ABCD2345",
    });
    expect(loadRelayRecord()).toMatchObject({
      relay_url: "https://relay.test",
      desk_id: "desk-42",
      viewer_id: "v1",
      viewer_token: "mu_v_tok",
      desk_name: "Studio",
    });
  });

  it("shows the manual form and sends nothing without link parameters", () => {
    render(<PairScreen hash="#/pair" />);

    expect(screen.getByTestId("pair-desk")).toHaveValue("");
    expect(screen.getByTestId("pair-code")).toHaveValue("");
    expect(fetch).not.toHaveBeenCalled();
  });

  it("refuses an empty code before making a request", () => {
    render(<PairScreen hash="#/pair" />);

    fireEvent.change(screen.getByTestId("pair-desk"), { target: { value: "desk-42" } });
    fireEvent.submit(screen.getByTestId("pair-form"));

    expect(fetch).not.toHaveBeenCalled();
    expect(screen.getByTestId("pair-error")).toHaveTextContent("Enter the desk ID");
  });

  it("clears the code field on bad_code", async () => {
    vi.mocked(fetch).mockResolvedValue(
      errResponse(401, { code: "bad_code", message: "wrong code" }),
    );

    render(<PairScreen hash="#/pair" onPaired={vi.fn()} />);
    fireEvent.change(screen.getByTestId("pair-desk"), { target: { value: "desk-42" } });
    fireEvent.change(screen.getByTestId("pair-code"), { target: { value: "WRONG123" } });
    fireEvent.submit(screen.getByTestId("pair-form"));

    await waitFor(() =>
      expect(screen.getByTestId("pair-error")).toHaveTextContent("not right"),
    );
    expect(screen.getByTestId("pair-code")).toHaveValue("");
  });

  it("names the retry time on pairing_locked", async () => {
    vi.mocked(fetch).mockResolvedValue(
      errResponse(429, {
        code: "pairing_locked",
        message: "locked",
        retry_after_secs: 900,
      }),
    );

    render(
      <PairScreen hash="#/pair?d=desk-42&c=ABCD2345&r=https://relay.test" onPaired={vi.fn()} />,
    );

    await waitFor(() =>
      expect(screen.getByTestId("pair-error")).toHaveTextContent("Try again in 15 min"),
    );
  });

  it("says so when the relay cannot be reached", async () => {
    vi.mocked(fetch).mockRejectedValue(new Error("offline"));

    render(
      <PairScreen hash="#/pair?d=desk-42&c=ABCD2345&r=https://relay.test" onPaired={vi.fn()} />,
    );

    await waitFor(() =>
      expect(screen.getByTestId("pair-error")).toHaveTextContent("Could not reach the relay"),
    );
  });

  it("forgets a stored desk", () => {
    saveRelayRecord({
      relay_url: "https://relay.test",
      desk_id: "desk-42",
      viewer_id: "v1",
      viewer_token: "mu_v_tok",
      desk_name: "Studio",
    });
    const onForget = vi.fn();

    render(<PairScreen hash="#/pair" onForget={onForget} />);
    fireEvent.click(screen.getByTestId("pair-forget"));

    expect(localStorage.getItem(RELAY_STORAGE_KEY)).toBeNull();
    expect(onForget).toHaveBeenCalledOnce();
  });

  it("offers no forget button when nothing is paired", () => {
    render(<PairScreen hash="#/pair" />);
    expect(screen.queryByTestId("pair-forget")).toBeNull();
  });
});
