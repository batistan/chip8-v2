import { createEffect, type JSX } from "solid-js";

interface DialogProps {
  title: string;
  open: boolean;
  onClose: () => void;
  children: JSX.Element;
}

export function Dialog(props: DialogProps) {
  let dialogRef!: HTMLDialogElement;
  const titleId = `dialog-title-${Math.random().toString(36).substring(2, 8)}`;

  createEffect(() => {
    if (props.open && !dialogRef.open) {
      dialogRef.showModal();
    } else if (!props.open && dialogRef.open) {
      dialogRef.close();
    }
  });

  return (
    <dialog
      ref={dialogRef}
      class="window dialog"
      aria-labelledby={titleId}
      onClose={() => props.onClose()}
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
