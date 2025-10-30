import { useState, useEffect } from 'react'
import { Dialog, DialogActions, DialogBody, DialogTitle } from './dialog'
import { Field, FieldGroup, Label } from './fieldset'
import { Input } from './input'
import { Select } from './select'
import { Button } from './button'
import { FunctionFormFields } from './FunctionFormFields'
import { WorkflowTask } from '../types'
import { validateJson, buildTaskFunction } from '../utils/functionBuilder'

interface NodeEditorModalProps {
  isOpen: boolean
  node: WorkflowTask | null
  onSave: (updatedNode: WorkflowTask) => void
  onClose: () => void
}

export function NodeEditorModal({ isOpen, node, onSave, onClose }: NodeEditorModalProps) {
  const [name, setName] = useState('')
  const [functionType, setFunctionType] = useState<WorkflowTask['function']['name']>('cli_command')
  
  // CLI Command fields
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  
  // Cursor Agent fields
  const [prompt, setPrompt] = useState('')
  const [configJson, setConfigJson] = useState('{}')
  const [configError, setConfigError] = useState('')
  
  // User Input fields
  const [inputType, setInputType] = useState('string')
  const [required, setRequired] = useState(true)
  const [defaultValue, setDefaultValue] = useState('')
  
  // Custom fields
  const [customName, setCustomName] = useState('custom')
  const [customInputJson, setCustomInputJson] = useState('{}')
  const [customInputError, setCustomInputError] = useState('')

  // Load node data into form
  useEffect(() => {
    if (!node) return;

    setName(node.name || '')
    setFunctionType(node.function.name)
    
    switch (node.function.name) {
      case 'cli_command':
        setCommand(node.function.input.command || '')
        setArgs(node.function.input.args?.join(', ') || '')
        break
        
      case 'cursor_agent':
        setPrompt(node.function.input.prompt || '')
        setConfigJson(JSON.stringify(node.function.input.config || {}, null, 2))
        break
        
      case 'user_input':
        setPrompt(node.function.input.prompt || '')
        setInputType(node.function.input.input_type || 'string')
        setRequired(node.function.input.required !== false)
        setDefaultValue(
          node.function.input.default !== undefined && node.function.input.default !== null
            ? String(node.function.input.default)
            : ''
        )
        break
        
      case 'custom':
        setCustomName(node.function.name || 'custom')
        setCustomInputJson(JSON.stringify(node.function.input || {}, null, 2))
        break
    }
  }, [node])

  const handleConfigBlur = () => {
    setConfigError(validateJson(configJson) ? '' : 'Invalid JSON')
  }

  const handleCustomInputBlur = () => {
    setCustomInputError(validateJson(customInputJson) ? '' : 'Invalid JSON')
  }

  const handleSave = () => {
    if (!node) return

    // Validate JSON fields before saving
    if (functionType === 'cursor_agent' && !validateJson(configJson)) {
      setConfigError('Invalid JSON - cannot save')
      return
    }
    
    if (functionType === 'custom' && !validateJson(customInputJson)) {
      setCustomInputError('Invalid JSON - cannot save')
      return
    }

    // Build function using helper
    const updatedFunction = buildTaskFunction({
      functionType,
      command,
      args,
      prompt,
      configJson,
      inputType,
      required,
      defaultValue,
      customName,
      customInputJson
    })

    if (!updatedFunction) return

    const updatedNode: WorkflowTask = {
      ...node,
      name,
      function: updatedFunction
    }
    
    onSave(updatedNode)
    onClose()
  }

  const handleCancel = () => {
    // Reset errors
    setConfigError('')
    setCustomInputError('')
    onClose()
  }

  return (
    <Dialog open={isOpen} onClose={handleCancel} size="xl">
      <DialogTitle>Edit Node</DialogTitle>
      <DialogBody>
        <FieldGroup>
          <Field>
            <Label>Node Name</Label>
            <Input 
              value={name} 
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter node name"
            />
          </Field>

          <Field>
            <Label>Function Type</Label>
            <Select 
              value={functionType} 
              onChange={(e) => setFunctionType(e.target.value as WorkflowTask['function']['name'])}
            >
              <option value="cli_command">CLI Command</option>
              <option value="cursor_agent">Cursor Agent</option>
              <option value="user_input">User Input</option>
              <option value="custom">Custom</option>
            </Select>
          </Field>

          <FunctionFormFields
            functionType={functionType}
            command={command}
            args={args}
            onCommandChange={setCommand}
            onArgsChange={setArgs}
            prompt={prompt}
            configJson={configJson}
            configError={configError}
            onPromptChange={setPrompt}
            onConfigJsonChange={setConfigJson}
            onConfigBlur={handleConfigBlur}
            inputType={inputType}
            required={required}
            defaultValue={defaultValue}
            onInputTypeChange={setInputType}
            onRequiredChange={setRequired}
            onDefaultValueChange={setDefaultValue}
            customName={customName}
            customInputJson={customInputJson}
            customInputError={customInputError}
            onCustomNameChange={setCustomName}
            onCustomInputJsonChange={setCustomInputJson}
            onCustomInputBlur={handleCustomInputBlur}
          />
        </FieldGroup>
      </DialogBody>
      <DialogActions>
        <Button variant="plain" onClick={handleCancel}>Cancel</Button>
        <Button onClick={handleSave}>Save Changes</Button>
      </DialogActions>
    </Dialog>
  )
}
