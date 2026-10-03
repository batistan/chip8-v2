// @vitest-environment node
import { describe, test, expect, afterEach } from "vitest";
import {
  addError,
  clearErrors,
  dismissError,
  emulationState,
  getErrors,
  isModalOpen,
  modalClosed,
  modalOpened,
  setEmulationState,
  togglePause,
} from "./state";

afterEach(() => clearErrors());

describe("clearErrors", () => {
  test("should empty the list of errors", () => {
    addError("foo");
    addError("bar");
    addError("baz");

    expect(getErrors()).not.toHaveLength(0);

    clearErrors();

    expect(getErrors()).toHaveLength(0);
  });
});

describe("addError", () => {
  test("should append an entry whose message matches", () => {
    expect(getErrors()).toHaveLength(0);

    addError("some err");

    const errs = getErrors();
    expect(errs).toHaveLength(1);
    expect(errs[0].message).toBe("some err");
  });
});

describe("dismissError", () => {
  test("should remove the error with the specified ID", () => {
    addError("same error");
    addError("same error");
    addError("same error");

    const [e1, e2, e3] = getErrors();

    expect(e1).toBeDefined();
    expect(e2).toBeDefined();
    expect(e3).toBeDefined();

    dismissError(e2.id);

    const [first, last] = getErrors();

    expect(e1).toBe(first);
    expect(e3).toBe(last);
  });

  test("should silently no-op if the error with the specified ID does not exist", () => {
    addError("same error");

    const [e] = getErrors();

    expect(e).toBeDefined();

    dismissError(e.id + "foo");

    expect(getErrors()).toHaveLength(1);
  });
});

describe("togglePause", () => {
  afterEach(() => setEmulationState("stopped"));

  test("should flip between running and paused", () => {
    setEmulationState("running");
    togglePause();
    expect(emulationState()).toBe("paused");
    togglePause();
    expect(emulationState()).toBe("running");
  });

  test.each(["stopped", "error"] as const)(
    "should leave %s untouched",
    (state) => {
      setEmulationState(state);
      togglePause();
      expect(emulationState()).toBe(state);
    },
  );
});

describe("modalOpened / modalClosed", () => {
  afterEach(() => {
    while (isModalOpen()) modalClosed();
    setEmulationState("stopped");
  });

  test("should pause a running game and resume it on close", () => {
    setEmulationState("running");

    modalOpened();
    expect(isModalOpen()).toBe(true);
    expect(emulationState()).toBe("paused");

    modalClosed();
    expect(isModalOpen()).toBe(false);
    expect(emulationState()).toBe("running");
  });

  test("should leave a game the user paused paused on close", () => {
    setEmulationState("paused");

    modalOpened();
    modalClosed();

    expect(emulationState()).toBe("paused");
  });

  test("should only resume once every open modal has closed", () => {
    setEmulationState("running");

    modalOpened();
    modalOpened();
    modalClosed();
    expect(isModalOpen()).toBe(true);
    expect(emulationState()).toBe("paused");

    modalClosed();
    expect(emulationState()).toBe("running");
  });

  test.each(["stopped", "error"] as const)(
    "should leave %s untouched",
    (state) => {
      setEmulationState(state);
      modalOpened();
      expect(emulationState()).toBe(state);
      modalClosed();
      expect(emulationState()).toBe(state);
    },
  );

  test("should not resume if the game stopped while the modal was open", () => {
    setEmulationState("running");

    modalOpened();
    setEmulationState("error");
    modalClosed();

    expect(emulationState()).toBe("error");
  });

  test("an extra close should not drive the count negative", () => {
    modalClosed();
    expect(isModalOpen()).toBe(false);

    modalOpened();
    expect(isModalOpen()).toBe(true);
  });
});
