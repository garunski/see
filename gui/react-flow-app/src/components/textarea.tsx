import { forwardRef } from 'react'
import * as Headless from '@headlessui/react'

export const Textarea = forwardRef<HTMLTextAreaElement, {
  value: string
  onChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => void
  placeholder?: string
  rows?: number
  className?: string
  disabled?: boolean
  invalid?: boolean
}>(({ className = '', disabled = false, invalid = false, ...props }, ref) => {
  return (
    <span className={`relative block w-full ${className}`}>
      <Headless.Textarea
        ref={ref}
        disabled={disabled}
        className={`
          relative block w-full appearance-none rounded-lg px-3 py-2.5 text-sm
          text-zinc-900 placeholder:text-zinc-500 dark:text-white dark:placeholder:text-zinc-400
          border border-zinc-300 hover:border-zinc-400 dark:border-zinc-600 dark:hover:border-zinc-500
          bg-white dark:bg-zinc-800
          focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
          disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-zinc-50 dark:disabled:bg-zinc-900
          ${invalid ? 'border-red-500 focus:ring-red-500 dark:border-red-600' : ''}
          resize-y transition-colors
        `}
        {...props}
      />
    </span>
  )
})

Textarea.displayName = 'Textarea'