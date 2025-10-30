import * as Headless from "@headlessui/react";

export function Dialog({
  open,
  onClose,
  size = "md",
  children,
}: {
  open: boolean;
  onClose: () => void;
  size?: "sm" | "md" | "lg" | "xl";
  children: React.ReactNode;
}) {
  const sizeClasses = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-xl",
  };

  return (
    <Headless.Dialog open={open} onClose={onClose} className="relative z-50">
      <div
        className="fixed inset-0 bg-black/30 dark:bg-black/50"
        aria-hidden="true"
      />

      <div className="fixed inset-0 flex items-center justify-center p-4">
        <Headless.DialogPanel
          className={`${sizeClasses[size]} w-full bg-white dark:bg-zinc-900 rounded-t-3xl sm:rounded-2xl p-6 shadow-xl ring-1 ring-zinc-950/10 dark:ring-white/10 transition duration-100 will-change-transform`}
        >
          {children}
        </Headless.DialogPanel>
      </div>
    </Headless.Dialog>
  );
}

export function DialogTitle({ children }: { children: React.ReactNode }) {
  return (
    <Headless.DialogTitle className="text-lg font-semibold text-zinc-900 dark:text-white mb-4">
      {children}
    </Headless.DialogTitle>
  );
}

export function DialogBody({ children }: { children: React.ReactNode }) {
  return <div className="mt-6">{children}</div>;
}

export function DialogActions({ children }: { children: React.ReactNode }) {
  return <div className="mt-8 flex gap-3 justify-end">{children}</div>;
}
