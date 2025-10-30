/**
 * Form field configuration types for the Factory Pattern
 */

export type FieldType = 'text' | 'textarea' | 'select' | 'checkbox' | 'json' | 'array'

export interface BaseFieldConfig {
  name: string
  label: string
  type: FieldType
  placeholder?: string
  disabled?: boolean
  error?: string
}

export interface TextFieldConfig extends BaseFieldConfig {
  type: 'text'
  value: string
  onChange: (value: string) => void
}

export interface TextareaFieldConfig extends BaseFieldConfig {
  type: 'textarea'
  value: string
  onChange: (value: string) => void
  rows: number
}

export interface SelectFieldConfig extends BaseFieldConfig {
  type: 'select'
  value: string
  onChange: (value: string) => void
  options: { value: string; label: string }[]
}

export interface CheckboxFieldConfig extends BaseFieldConfig {
  type: 'checkbox'
  value: boolean
  onChange: (value: boolean) => void
}

export interface JsonFieldConfig extends BaseFieldConfig {
  type: 'json'
  value: string
  onChange: (value: string) => void
  onBlur: () => void
  error?: string
  rows: number
}

export interface ArrayFieldConfig extends BaseFieldConfig {
  type: 'array'
  value: string[]
  onChange: (value: string[]) => void
  itemPlaceholder?: string
}

export type FieldConfig = 
  | TextFieldConfig 
  | TextareaFieldConfig 
  | SelectFieldConfig 
  | CheckboxFieldConfig 
  | JsonFieldConfig
  | ArrayFieldConfig

