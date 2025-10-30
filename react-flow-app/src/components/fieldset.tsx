import * as Headless from "@headlessui/react";

export function FieldGroup({ children }: { children: React.ReactNode }) {
  return <div className="space-y-6">{children}</div>;
}

export function Field({ children }: { children: React.ReactNode }) {
  return <Headless.Field className="space-y-2">{children}</Headless.Field>;
}

export function Label({ children }: { children: React.ReactNode }) {
  return (
    <Headless.Label className="block text-sm font-semibold text-zinc-900 dark:text-white">
      {children}
    </Headless.Label>
  );
}
