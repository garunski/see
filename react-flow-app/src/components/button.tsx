import { forwardRef } from 'react'
import * as Headless from '@headlessui/react'

export const Button = forwardRef<HTMLButtonElement, {
  variant?: 'solid' | 'plain'
  children: React.ReactNode
  onClick?: () => void
  className?: string
  disabled?: boolean
}>(({ variant = 'solid', children, className = '', disabled = false, ...props }, ref) => {
  const baseClasses = 'relative isolate inline-flex items-center justify-center gap-x-2 rounded-lg border text-sm font-semibold px-4 py-2.5 transition-colors min-w-[100px] focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed'
  
  const variantClasses = variant === 'solid'
    ? 'border-transparent bg-zinc-900 text-white hover:bg-zinc-700 active:bg-zinc-800 dark:bg-zinc-600 dark:hover:bg-zinc-500 dark:active:bg-zinc-700 shadow-sm'
    : 'border-zinc-300 text-zinc-700 hover:bg-zinc-50 active:bg-zinc-100 dark:border-zinc-600 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:active:bg-zinc-700'

  return (
    <Headless.Button
      ref={ref}
      className={`${baseClasses} ${variantClasses} ${className}`}
      disabled={disabled}
      {...props}
    >
      {children}
    </Headless.Button>
  )
})

Button.displayName = 'Button'