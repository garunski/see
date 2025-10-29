import { useState, useEffect } from 'react'
import { Dialog, DialogActions, DialogBody, DialogTitle } from './dialog'
import { Field, FieldGroup, Label } from './fieldset'
import { Input } from './input'
import { Select } from './select'
import { Textarea } from './textarea'
import { Button } from './button'
import { WorkflowTask } from '../types'

interface NodeEditorModalProps {
  isOpen: boolean
  node: WorkflowTask | null
  onSave: (updatedNode: WorkflowTask) => void
  onClose: () => void
}

export function NodeEditorModal({ isOpen, node, onSave, onClose }: NodeEditorModalProps) {
  const [name, setName] = useState('')
  const [functionType, setFunctionType] = useState('cli_command')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  const [prompt, setPrompt] = useState('')
  const [inputType, setInputType] = useState('text')
  const [required, setRequired] = useState(true)

  // Update form when node changes
  useEffect(() => {
    if (node) {
      setName(node.name || '')
      setFunctionType(node.function?.name || 'cli_command')
      setCommand(node.function?.input?.command || '')
      setArgs(node.function?.input?.args?.join(', ') || '')
      setPrompt(node.function?.input?.prompt || '')
      setInputType(node.function?.input?.input_type || 'text')
      setRequired(node.function?.input?.required !== false)
    }
  }, [node])

  const handleSave = () => {
    if (!node) return

    // Build updated node
    const updatedNode: WorkflowTask = {
      ...node,
      name,
      function: {
        name: functionType,
        input: functionType === 'cli_command'
          ? { 
              command, 
              args: args.split(',').map(s => s.trim()).filter(Boolean) 
            }
          : functionType === 'user_input'
          ? {
              prompt,
              input_type: inputType,
              required
            }
          : { 
              prompt 
            }
      }
    }
    
    onSave(updatedNode)
    onClose()
  }

  const handleCancel = () => {
    // Reset form to original values
    if (node) {
      setName(node.name || '')
      setFunctionType(node.function?.name || 'cli_command')
      setCommand(node.function?.input?.command || '')
      setArgs(node.function?.input?.args?.join(', ') || '')
      setPrompt(node.function?.input?.prompt || '')
      setInputType(node.function?.input?.input_type || 'text')
      setRequired(node.function?.input?.required !== false)
    }
    onClose()
  }

  return (
    <Dialog open={isOpen} onClose={handleCancel} size="md">
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
              onChange={(e) => setFunctionType(e.target.value)}
            >
              <option value="cli_command">CLI Command</option>
              <option value="cursor_agent">Cursor Agent</option>
              <option value="user_input">User Input</option>
            </Select>
          </Field>

          {functionType === 'cli_command' ? (
            <>
              <Field>
                <Label>Command</Label>
                <Input 
                  value={command} 
                  onChange={(e) => setCommand(e.target.value)}
                  placeholder="e.g., echo, ls, curl"
                />
              </Field>
              <Field>
                <Label>Arguments (comma-separated)</Label>
                <Input 
                  value={args} 
                  onChange={(e) => setArgs(e.target.value)}
                  placeholder="e.g., Hello World, -l, /path/to/file"
                />
              </Field>
            </>
          ) : functionType === 'user_input' ? (
            <>
              <Field>
                <Label>Prompt</Label>
                <Textarea 
                  value={prompt} 
                  onChange={(e) => setPrompt(e.target.value)}
                  rows={4}
                  placeholder="Enter prompt to show the user"
                />
              </Field>
              <Field>
                <Label>Input Type</Label>
                <Select 
                  value={inputType} 
                  onChange={(e) => setInputType(e.target.value)}
                >
                  <option value="text">Text</option>
                  <option value="number">Number</option>
                  <option value="boolean">Boolean</option>
                </Select>
              </Field>
              <Field>
                <Label>
                  <input 
                    type="checkbox" 
                    checked={required} 
                    onChange={(e) => setRequired(e.target.checked)}
                    className="mr-2"
                  />
                  Required
                </Label>
              </Field>
            </>
          ) : (
            <Field>
              <Label>Prompt</Label>
              <Textarea 
                value={prompt} 
                onChange={(e) => setPrompt(e.target.value)}
                rows={4}
                placeholder="Enter your prompt for the Cursor agent"
              />
            </Field>
          )}
        </FieldGroup>
      </DialogBody>
      <DialogActions>
        <Button variant="plain" onClick={handleCancel}>Cancel</Button>
        <Button onClick={handleSave}>Save Changes</Button>
      </DialogActions>
    </Dialog>
  )
}
