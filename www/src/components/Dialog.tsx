import { createEffect, onCleanup, untrack, type JSX } from "solid-js";
import { modalClosed, modalOpened } from "../state";

interface DialogProps {
  title: string;
  open: boolean;
  onClose: () => void;
  children: JSX.Element;
}

export function Dialog(props: DialogProps) {
  let dialogRef!: HTMLDialogElement;
  const titleId = `dialog-title-${Math.random().toString(36).substring(2, 8)}`;

  let registered = false;

  function unregister() {
    if (!registered) return;
    registered = false;
    modalClosed();
  }

  createEffect(() => {
    if (props.open && !dialogRef.open) {
      dialogRef.showModal();
      if (!registered) {
        registered = true;
        untrack(modalOpened);
      }
    } else if (!props.open && dialogRef.open) {
      dialogRef.close();
    }
  });

  onCleanup(() => {
    if (dialogRef.open) dialogRef.close();
    unregister();
  });

  function handleClose() {
    unregister();
    props.onClose();
  }

  return (
    <dialog
      ref={dialogRef}
      class="window dialog"
      aria-labelledby={titleId}
      onClose={handleClose}
    >
      <div class="window-titlebar">
        <span id={titleId} class="window-title">{props.title}</span>
        <div class="window-controls">
          <button
            type="button"
            class="window-control"
            aria-label="Close"
            onClick={() => props.onClose()}
          >
            ×
          </button>
        </div>
      </div>
      <div class="window-body dialog-body">{props.children}</div>
    </dialog>
  );
}
