'use client';

import { useState } from 'react';

interface WorkflowStepEditorProps {
  step: { action_type: string; action_config: Record<string, unknown> };
  stepIndex: number;
  onChange: (step: { action_type: string; action_config: Record<string, unknown> }) => void;
  onDelete: () => void;
  dragHandleProps?: Record<string, unknown>;
}

const ACTION_TYPES = [
  { value: 'send_message', label: 'Send Message' },
  { value: 'add_reaction', label: 'Add Reaction' },
  { value: 'create_channel', label: 'Create Channel' },
  { value: 'invite_to_channel', label: 'Invite to Channel' },
  { value: 'update_channel_topic', label: 'Update Channel Topic' },
  { value: 'call_webhook', label: 'Call Webhook' },
  { value: 'delay', label: 'Delay' },
  { value: 'create_form', label: 'Create Form' },
] as const;

function getActionLabel(actionType: string): string {
  return ACTION_TYPES.find((a) => a.value === actionType)?.label ?? actionType;
}

function inputClass(extra = '') {
  return `w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 ${extra}`.trim();
}

function labelClass() {
  return 'mb-1 block text-sm font-medium text-gray-300';
}

function hintClass() {
  return 'mt-1 text-xs text-gray-500';
}

interface ConfigFieldsProps {
  actionType: string;
  config: Record<string, unknown>;
  onChange: (config: Record<string, unknown>) => void;
}

function ConfigFields({ actionType, config, onChange }: ConfigFieldsProps) {
  function set(key: string, value: unknown) {
    onChange({ ...config, [key]: value });
  }

  function str(key: string): string {
    return typeof config[key] === 'string' ? (config[key] as string) : '';
  }

  function num(key: string): number | '' {
    return typeof config[key] === 'number' ? (config[key] as number) : '';
  }

  switch (actionType) {
    case 'send_message':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Channel ID</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. {{channel_id}} or a specific ID"
              value={str('channel_id')}
              onChange={(e) => set('channel_id', e.target.value)}
            />
            <p className={hintClass()}>
              Use <code className="rounded bg-gray-800 px-1 text-blue-400">{'{{variable}}'}</code> for dynamic values.
            </p>
          </div>
          <div>
            <label className={labelClass()}>Message Content</label>
            <textarea
              className={inputClass()}
              rows={4}
              placeholder="Hello {{user_name}}, welcome to {{channel_name}}!"
              value={str('content')}
              onChange={(e) => set('content', e.target.value)}
            />
            <p className={hintClass()}>
              Supports{' '}
              <code className="rounded bg-gray-800 px-1 text-blue-400">{'{{variable}}'}</code>{' '}
              template syntax for dynamic substitution.
            </p>
          </div>
        </div>
      );

    case 'add_reaction':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Message ID</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. {{message_id}}"
              value={str('message_id')}
              onChange={(e) => set('message_id', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>Emoji</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. 👍 or :thumbsup:"
              value={str('emoji')}
              onChange={(e) => set('emoji', e.target.value)}
            />
          </div>
        </div>
      );

    case 'create_channel':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Channel Name</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. project-{{project_name}}"
              value={str('name')}
              onChange={(e) => set('name', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>Description</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="Short description of the channel"
              value={str('description')}
              onChange={(e) => set('description', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>Channel Type</label>
            <select
              className={inputClass()}
              value={str('channel_type') || 'public'}
              onChange={(e) => set('channel_type', e.target.value)}
            >
              <option value="public">Public</option>
              <option value="private">Private</option>
            </select>
          </div>
        </div>
      );

    case 'invite_to_channel':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Channel ID</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. {{channel_id}}"
              value={str('channel_id')}
              onChange={(e) => set('channel_id', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>User ID</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. {{user_id}}"
              value={str('user_id')}
              onChange={(e) => set('user_id', e.target.value)}
            />
          </div>
        </div>
      );

    case 'update_channel_topic':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Channel ID</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. {{channel_id}}"
              value={str('channel_id')}
              onChange={(e) => set('channel_id', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>Topic / Description</label>
            <textarea
              className={inputClass()}
              rows={3}
              placeholder="New topic for the channel"
              value={str('description')}
              onChange={(e) => set('description', e.target.value)}
            />
          </div>
        </div>
      );

    case 'call_webhook':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>URL</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="https://example.com/webhook"
              value={str('url')}
              onChange={(e) => set('url', e.target.value)}
            />
          </div>
          <div>
            <label className={labelClass()}>Method</label>
            <select
              className={inputClass()}
              value={str('method') || 'POST'}
              onChange={(e) => set('method', e.target.value)}
            >
              <option value="GET">GET</option>
              <option value="POST">POST</option>
              <option value="PUT">PUT</option>
              <option value="DELETE">DELETE</option>
            </select>
          </div>
          <div>
            <label className={labelClass()}>Body (JSON)</label>
            <textarea
              className={`${inputClass()} font-mono text-sm`}
              rows={5}
              placeholder={'{\n  "key": "{{variable}}"\n}'}
              value={str('body')}
              onChange={(e) => set('body', e.target.value)}
            />
            <p className={hintClass()}>Optional. Must be valid JSON. Supports template variables.</p>
          </div>
        </div>
      );

    case 'delay':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Delay (seconds)</label>
            <input
              type="number"
              min={0}
              className={inputClass()}
              placeholder="60"
              value={num('seconds')}
              onChange={(e) => set('seconds', e.target.value === '' ? '' : Number(e.target.value))}
            />
            <p className={hintClass()}>Pause workflow execution for this many seconds before continuing.</p>
          </div>
        </div>
      );

    case 'create_form':
      return (
        <div className="space-y-3">
          <div>
            <label className={labelClass()}>Form Title</label>
            <input
              type="text"
              className={inputClass()}
              placeholder="e.g. Onboarding Survey"
              value={str('title')}
              onChange={(e) => set('title', e.target.value)}
            />
          </div>
          <div className="rounded-lg border border-dashed border-gray-700 bg-gray-900 px-4 py-5 text-center">
            <p className="text-sm text-gray-400">Form field editor coming in Phase 8.3.</p>
            <p className="mt-1 text-xs text-gray-600">
              Fields, validation, and submission handling will be configurable here.
            </p>
          </div>
        </div>
      );

    default:
      return (
        <div className="rounded-lg border border-gray-700 bg-gray-900 px-4 py-3 text-sm text-gray-500">
          No configuration available for this action type.
        </div>
      );
  }
}

export default function WorkflowStepEditor({
  step,
  stepIndex,
  onChange,
  onDelete,
  dragHandleProps,
}: WorkflowStepEditorProps) {
  const [collapsed, setCollapsed] = useState(false);

  function handleActionTypeChange(newType: string) {
    onChange({ action_type: newType, action_config: {} });
  }

  function handleConfigChange(newConfig: Record<string, unknown>) {
    onChange({ ...step, action_config: newConfig });
  }

  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800 shadow-sm">
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2">
        {/* Drag handle */}
        <div
          {...(dragHandleProps as React.HTMLAttributes<HTMLDivElement>)}
          className="flex cursor-grab items-center text-gray-500 hover:text-gray-300 active:cursor-grabbing"
          title="Drag to reorder"
          aria-label="Drag handle"
        >
          <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 16 16">
            <circle cx="5" cy="4" r="1.25" />
            <circle cx="11" cy="4" r="1.25" />
            <circle cx="5" cy="8" r="1.25" />
            <circle cx="11" cy="8" r="1.25" />
            <circle cx="5" cy="12" r="1.25" />
            <circle cx="11" cy="12" r="1.25" />
          </svg>
        </div>

        {/* Step badge */}
        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-blue-600 text-xs font-semibold text-white">
          {stepIndex + 1}
        </span>

        {/* Collapse toggle + action type label */}
        <button
          type="button"
          onClick={() => setCollapsed((c) => !c)}
          className="flex flex-1 items-center gap-2 text-left"
        >
          <span className="text-sm font-medium text-white">{getActionLabel(step.action_type)}</span>
          <svg
            className={`h-4 w-4 shrink-0 text-gray-400 transition-transform ${collapsed ? '-rotate-90' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        {/* Delete button */}
        <button
          type="button"
          onClick={onDelete}
          className="ml-auto rounded p-1 text-gray-500 hover:bg-gray-700 hover:text-red-400"
          title="Delete step"
          aria-label="Delete step"
        >
          <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
          </svg>
        </button>
      </div>

      {/* Body */}
      {!collapsed && (
        <div className="border-t border-gray-700 px-4 py-4 space-y-4">
          {/* Action type selector */}
          <div>
            <label className={labelClass()}>Action Type</label>
            <select
              className={inputClass()}
              value={step.action_type}
              onChange={(e) => handleActionTypeChange(e.target.value)}
            >
              {ACTION_TYPES.map((a) => (
                <option key={a.value} value={a.value}>
                  {a.label}
                </option>
              ))}
            </select>
          </div>

          {/* Config fields */}
          <ConfigFields
            actionType={step.action_type}
            config={step.action_config}
            onChange={handleConfigChange}
          />
        </div>
      )}
    </div>
  );
}
