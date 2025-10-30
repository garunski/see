/**
 * FunctionFormFields - Combines Factory and Strategy patterns
 * 
 * - Factory Pattern: Creates field configurations based on function type
 * - Strategy Pattern: Renders fields based on their configuration
 */

import { TaskFunction } from '../types'
import { FieldConfigFactory, FieldFactoryProps } from './formFields/fieldFactory'
import { FieldRenderer } from './formFields/FieldRenderer'

interface FunctionFormFieldsProps extends FieldFactoryProps {
  functionType: TaskFunction['name']
}

export function FunctionFormFields(props: FunctionFormFieldsProps) {
  const { functionType, ...factoryProps } = props
  
  // Factory Pattern: Create field configurations
  const fieldConfigs = FieldConfigFactory.createFields(functionType, factoryProps)
  
  // Strategy Pattern: Render each field using its strategy
  return (
    <>
      {fieldConfigs.map((config) => (
        <FieldRenderer key={config.name} config={config} />
      ))}
    </>
  )
}
