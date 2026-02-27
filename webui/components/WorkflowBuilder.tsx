'use client';

import { useState } from 'react';
import type { Workflow, TriggerType, ActionType } from '@/lib/types';
import WorkflowStepEditor from './WorkflowStepEditor';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
  arrayMove,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

interface StepDraft {
  id: string;
  action_type: ActionType;
  action_config: Record<string, unknown>;
}

interface WorkflowBuilderProps {
  workflow?: Workflow;
  onSave: (data: {
    name: string;
    description?: string;
    trigger_type: TriggerType;
    trigger_config: Record<string, unknown>;
    steps: { action_type: ActionType; action_config: Record<string, unknown> }[];
  }) => Promise<void>;
  onCancel: () => void;
}

const TRIGGER_TYPES: { value: TriggerType; label: string }[] = [
  { value: 'message_posted', label: 'Message Posted' },
  { value: 'reaction_added', label: 'Reaction Added' },
  { value: 'channel_join', label: 'Channel Join' },
  { value: 'scheduled', label: 'Scheduled' },
  { value: 'webhook', label: 'Webhook' },
  { value: 'slash_command', label: 'Slash Command' },
];

const ACTION_TYPES: { value: ActionType; label: string }[] = [
  { value: 'send_message', label: 'Send Message' },
  { value: 'add_reaction', label: 'Add Reaction' },
  { value: 'create_channel', label: 'Create Channel' },
  { value: 'invite_to_channel', label: 'Invite to Channel' },
  { value: 'update_channel_topic', label: 'Update Channel Topic' },
  { value: 'call_webhook', label: 'Call Webhook' },
  { value: 'delay', label: 'Delay' },
  { value: 'create_form', label: 'Create Form' },
];

function SortableStepItem({
  step,
  stepIndex,
  onChange,
  onDelete,
}: {
  step: StepDraft;
  stepIndex: number;
  onChange: (step: { action_type: string; action_config: Record<string, unknown> }) => void;
  onDelete: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: step.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style}>
      <WorkflowStepEditor
        step={step}
        stepIndex={stepIndex}
        onChange={onChange}
        onDelete={onDelete}
        dragHandleProps={{ ...attributes, ...listeners }}
      />
    </div>
  );
}

export default function WorkflowBuilder({ workflow, onSave, onCancel }: WorkflowBuilderProps) {
  const [name, setName] = useState(workflow?.name || '');
  const [description, setDescription] = useState(workflow?.description || '');
  const [triggerType, setTriggerType] = useState<TriggerType>(workflow?.trigger_type || 'message_posted');
  const [triggerConfig, setTriggerConfig] = useState<Record<string, unknown>>(
    workflow?.trigger_config || {}
  );
  const [steps, setSteps] = useState<StepDraft[]>(
    workflow?.steps?.map((s, i) => ({
      id: `step-${i}-${Date.now()}`,
      action_type: s.action_type,
      action_config: s.action_config,
    })) || []
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      setSteps((prev) => {
        const oldIndex = prev.findIndex((s) => s.id === active.id);
        const newIndex = prev.findIndex((s) => s.id === over.id);
        return arrayMove(prev, oldIndex, newIndex);
      });
    }
  };

  const addStep = (actionType: ActionType) => {
    setSteps((prev) => [
      ...prev,
      {
        id: `step-${prev.length}-${Date.now()}`,
        action_type: actionType,
        action_config: {},
      },
    ]);
  };

  const updateStep = (index: number, data: { action_type: string; action_config: Record<string, unknown> }) => {
    setSteps((prev) => {
      const next = [...prev];
      next[index] = { ...next[index], action_type: data.action_type as ActionType, action_config: data.action_config };
      return next;
    });
  };

  const removeStep = (index: number) => {
    setSteps((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSave = async () => {
    if (!name.trim()) {
      setError('Workflow name is required');
      return;
    }
    if (steps.length === 0) {
      setError('At least one step is required');
      return;
    }

    try {
      setSaving(true);
      setError(null);
      await onSave({
        name: name.trim(),
        description: description.trim() || undefined,
        trigger_type: triggerType,
        trigger_config: triggerConfig,
        steps: steps.map((s) => ({
          action_type: s.action_type,
          action_config: s.action_config,
        })),
      });
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setSaving(false);
    }
  };

  const setTriggerConfigField = (key: string, value: unknown) => {
    setTriggerConfig((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <div className="space-y-6">
      {error && (
        <div className="rounded-lg border border-red-700 bg-red-900/50 p-3 text-sm text-red-300">
          {error}
        </div>
      )}

      {/* Name & Description */}
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-300">Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Workflow"
            className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-300">Description</label>
          <input
            type="text"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Optional description"
            className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
          />
        </div>
      </div>

      {/* Trigger Configuration */}
      <div className="rounded-lg border border-gray-800 bg-gray-900 p-4">
        <h3 className="mb-3 text-sm font-semibold text-white">Trigger</h3>
        <div className="space-y-3">
          <div>
            <label className="mb-1 block text-xs text-gray-400">When this happens:</label>
            <select
              value={triggerType}
              onChange={(e) => {
                setTriggerType(e.target.value as TriggerType);
                setTriggerConfig({});
              }}
              className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-white focus:border-blue-500 focus:outline-none"
            >
              {TRIGGER_TYPES.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          </div>

          {/* Trigger-specific config */}
          {triggerType === 'message_posted' && (
            <div className="space-y-2">
              <div>
                <label className="mb-1 block text-xs text-gray-400">Channel ID (optional)</label>
                <input
                  type="text"
                  value={(triggerConfig.channel_id as string) || ''}
                  onChange={(e) => setTriggerConfigField('channel_id', e.target.value)}
                  placeholder="Leave empty for all channels"
                  className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
                />
              </div>
              <div>
                <label className="mb-1 block text-xs text-gray-400">Keyword filter (optional)</label>
                <input
                  type="text"
                  value={(triggerConfig.keyword as string) || ''}
                  onChange={(e) => setTriggerConfigField('keyword', e.target.value)}
                  placeholder="e.g. help"
                  className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
                />
              </div>
            </div>
          )}

          {triggerType === 'reaction_added' && (
            <div>
              <label className="mb-1 block text-xs text-gray-400">Emoji (optional)</label>
              <input
                type="text"
                value={(triggerConfig.emoji as string) || ''}
                onChange={(e) => setTriggerConfigField('emoji', e.target.value)}
                placeholder="e.g. white_check_mark"
                className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
          )}

          {triggerType === 'channel_join' && (
            <div>
              <label className="mb-1 block text-xs text-gray-400">Channel ID (optional)</label>
              <input
                type="text"
                value={(triggerConfig.channel_id as string) || ''}
                onChange={(e) => setTriggerConfigField('channel_id', e.target.value)}
                placeholder="Leave empty for all channels"
                className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
          )}

          {triggerType === 'scheduled' && (
            <div>
              <label className="mb-1 block text-xs text-gray-400">Cron expression or interval</label>
              <input
                type="text"
                value={(triggerConfig.cron as string) || ''}
                onChange={(e) => setTriggerConfigField('cron', e.target.value)}
                placeholder="e.g. 0 9 * * 1-5 (weekdays at 9am)"
                className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
          )}

          {triggerType === 'webhook' && (
            <p className="text-xs text-gray-500">
              A unique webhook URL will be generated when you save this workflow.
            </p>
          )}

          {triggerType === 'slash_command' && (
            <div>
              <label className="mb-1 block text-xs text-gray-400">Command name</label>
              <input
                type="text"
                value={(triggerConfig.command_name as string) || ''}
                onChange={(e) => setTriggerConfigField('command_name', e.target.value)}
                placeholder="e.g. deploy"
                className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
              />
            </div>
          )}
        </div>
      </div>

      {/* Steps */}
      <div>
        <h3 className="mb-3 text-sm font-semibold text-white">Steps</h3>
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
          <SortableContext items={steps.map((s) => s.id)} strategy={verticalListSortingStrategy}>
            <div className="space-y-2">
              {steps.map((step, i) => (
                <SortableStepItem
                  key={step.id}
                  step={step}
                  stepIndex={i}
                  onChange={(data) => updateStep(i, data)}
                  onDelete={() => removeStep(i)}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>

        {steps.length === 0 && (
          <div className="rounded-lg border border-dashed border-gray-700 p-6 text-center text-sm text-gray-500">
            No steps yet. Add a step below.
          </div>
        )}

        <div className="mt-3">
          <select
            onChange={(e) => {
              if (e.target.value) {
                addStep(e.target.value as ActionType);
                e.target.value = '';
              }
            }}
            defaultValue=""
            className="rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none"
          >
            <option value="" disabled>
              + Add Step...
            </option>
            {ACTION_TYPES.map((a) => (
              <option key={a.value} value={a.value}>
                {a.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center justify-end gap-3 border-t border-gray-800 pt-4">
        <button
          onClick={onCancel}
          className="rounded-lg border border-gray-700 px-4 py-2 text-sm text-gray-300 hover:bg-gray-800"
        >
          Cancel
        </button>
        <button
          onClick={handleSave}
          disabled={saving}
          className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {saving ? 'Saving...' : workflow ? 'Update Workflow' : 'Create Workflow'}
        </button>
      </div>
    </div>
  );
}
