/**
 * Strategy Pattern: Renders different field types based on configuration
 */

import { Field, Label } from '../fieldset'
import { Input } from '../input'
import { Select } from '../select'
import { Textarea } from '../textarea'
import { FieldConfig } from './types'

interface FieldRendererProps {
  config: FieldConfig
}

/**
 * Strategy for rendering form fields based on their type
 */
export function FieldRenderer({ config }: FieldRendererProps) {
  switch (config.type) {
    case 'text':
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Input
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            placeholder={config.placeholder}
          />
        </Field>
      )

    case 'textarea':
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Textarea
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            rows={config.rows}
            placeholder={config.placeholder}
          />
        </Field>
      )

    case 'select':
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Select
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
          >
            {config.options.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </Select>
        </Field>
      )

    case 'checkbox':
      return (
        <Field key={config.name}>
          <Label>
            <input
              type="checkbox"
              checked={config.value}
              onChange={(e) => config.onChange(e.target.checked)}
              className="mr-2"
            />
            {config.label}
          </Label>
        </Field>
      )

    case 'json':
      return (
        <Field key={config.name}>
          <Label>{config.label}</Label>
          <Textarea
            value={config.value}
            onChange={(e) => config.onChange(e.target.value)}
            onBlur={config.onBlur}
            rows={config.rows}
            placeholder={config.placeholder}
            className={config.error ? 'border-red-500' : ''}
          />
          {config.error && (
            <p className="text-red-600 dark:text-red-400 text-sm mt-1">
              {config.error}
            </p>
          )}
        </Field>
      )

    default:
      return null
  }
}

