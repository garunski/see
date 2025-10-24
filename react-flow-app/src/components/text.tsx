import clsx from 'clsx'
import type React from 'react'

const weights = {
  thin: 'font-thin',
  extralight: 'font-extralight',
  light: 'font-light',
  normal: 'font-normal',
  medium: 'font-medium',
  semibold: 'font-semibold',
  bold: 'font-bold',
  extrabold: 'font-extrabold',
  black: 'font-black',
}

const sizes = {
  xs: 'text-xs',
  sm: 'text-sm',
  base: 'text-base',
  lg: 'text-lg',
  xl: 'text-xl',
  '2xl': 'text-2xl',
  '3xl': 'text-3xl',
  '4xl': 'text-4xl',
  '5xl': 'text-5xl',
  '6xl': 'text-6xl',
  '7xl': 'text-7xl',
  '8xl': 'text-8xl',
  '9xl': 'text-9xl',
}

const colors = {
  zinc: 'text-zinc-950 dark:text-white',
  red: 'text-red-600 dark:text-red-400',
  orange: 'text-orange-600 dark:text-orange-400',
  amber: 'text-amber-600 dark:text-amber-400',
  yellow: 'text-yellow-600 dark:text-yellow-400',
  lime: 'text-lime-600 dark:text-lime-400',
  green: 'text-green-600 dark:text-green-400',
  emerald: 'text-emerald-600 dark:text-emerald-400',
  teal: 'text-teal-600 dark:text-teal-400',
  cyan: 'text-cyan-600 dark:text-cyan-400',
  sky: 'text-sky-600 dark:text-sky-400',
  blue: 'text-blue-600 dark:text-blue-400',
  indigo: 'text-indigo-600 dark:text-indigo-400',
  violet: 'text-violet-600 dark:text-violet-400',
  purple: 'text-purple-600 dark:text-purple-400',
  fuchsia: 'text-fuchsia-600 dark:text-fuchsia-400',
  pink: 'text-pink-600 dark:text-pink-400',
  rose: 'text-rose-600 dark:text-rose-400',
}

export function Text({
  size = 'base',
  weight = 'normal',
  color = 'zinc',
  className,
  ...props
}: {
  size?: keyof typeof sizes
  weight?: keyof typeof weights
  color?: keyof typeof colors
  className?: string
} & Omit<React.HTMLAttributes<HTMLSpanElement>, 'className'>) {
  return (
    <span
      {...props}
      className={clsx(className, sizes[size], weights[weight], colors[color])}
    />
  )
}
