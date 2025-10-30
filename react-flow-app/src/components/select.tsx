import { forwardRef } from "react";
import * as Headless from "@headlessui/react";

export const Select = forwardRef<
  HTMLSelectElement,
  {
    value: string;
    onChange: (e: React.ChangeEvent<HTMLSelectElement>) => void;
    children: React.ReactNode;
    className?: string;
    disabled?: boolean;
    invalid?: boolean;
  }
>(
  (
    { className = "", children, disabled = false, invalid = false, ...props },
    ref,
  ) => {
    return (
      <span className={`group relative block w-full ${className}`}>
        <Headless.Select
          ref={ref}
          disabled={disabled}
          className={`
          relative block w-full appearance-none rounded-lg px-3 py-2.5 pr-10 text-sm
          text-zinc-900 dark:text-white
          border border-zinc-300 hover:border-zinc-400 dark:border-zinc-600 dark:hover:border-zinc-500
          bg-white dark:bg-zinc-800
          focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
          disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-zinc-50 dark:disabled:bg-zinc-900
          ${invalid ? "border-red-500 focus:ring-red-500 dark:border-red-600" : ""}
          transition-colors
        `}
          {...props}
        >
          {children}
        </Headless.Select>
        <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
          <svg
            className="h-4 w-4 stroke-zinc-500 group-disabled:stroke-zinc-400 dark:stroke-zinc-400"
            viewBox="0 0 16 16"
            aria-hidden="true"
            fill="none"
          >
            <path
              d="M5.75 10.75L8 13L10.25 10.75"
              strokeWidth={1.5}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <path
              d="M10.25 5.25L8 3L5.75 5.25"
              strokeWidth={1.5}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </span>
      </span>
    );
  },
);

Select.displayName = "Select";
