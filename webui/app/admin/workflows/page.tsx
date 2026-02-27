'use client';

import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '@/lib/api';
import type { WorkflowListItem, Workflow, TriggerType, ActionType } from '@/lib/types';
import WorkflowBuilder from '@/components/WorkflowBuilder';
import WorkflowExecutionHistory from '@/components/WorkflowExecutionHistory';

const TRIGGER_LABELS: Record<TriggerType, string> = {
  message_posted: 'Message Posted',
  reaction_added: 'Reaction Added',
  channel_join: 'Channel Join',
  scheduled: 'Scheduled',
  webhook: 'Webhook',
  slash_command: 'Slash Command',
};

const TRIGGER_COLORS: Record<TriggerType, string> = {
  message_posted: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  reaction_added: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
  channel_join: 'bg-green-500/20 text-green-400 border-green-500/30',
  scheduled: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
  webhook: 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30',
  slash_command: 'bg-pink-500/20 text-pink-400 border-pink-500/30',
};

export default function WorkflowsAdminPage() {
  const [workflows, setWorkflows] = useState<WorkflowListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Builder state
  const [showBuilder, setShowBuilder] = useState(false);
  const [editingWorkflow, setEditingWorkflow] = useState<Workflow | null>(null);
  const [loadingWorkflow, setLoadingWorkflow] = useState(false);

  // Execution history
  const [showHistory, setShowHistory] = useState<string | null>(null);

  const fetchWorkflows = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.listWorkflows();
      setWorkflows(data);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkflows();
  }, [fetchWorkflows]);

  const handleCreate = () => {
    setEditingWorkflow(null);
    setShowBuilder(true);
  };

  const handleEdit = async (id: string) => {
    try {
      setLoadingWorkflow(true);
      const workflow = await apiClient.getWorkflow(id);
      setEditingWorkflow(workflow);
      setShowBuilder(true);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoadingWorkflow(false);
    }
  };

  const handleSave = async (data: {
    name: string;
    description?: string;
    trigger_type: TriggerType;
    trigger_config: Record<string, unknown>;
    steps: { action_type: ActionType; action_config: Record<string, unknown> }[];
  }) => {
    if (editingWorkflow) {
      await apiClient.updateWorkflow(editingWorkflow.id, data);
    } else {
      await apiClient.createWorkflow(data);
    }
    setShowBuilder(false);
    setEditingWorkflow(null);
    await fetchWorkflows();
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this workflow?')) return;
    try {
      await apiClient.deleteWorkflow(id);
      setWorkflows((prev) => prev.filter((w) => w.id !== id));
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const handleToggleEnabled = async (workflow: WorkflowListItem) => {
    try {
      const updated = workflow.enabled
        ? await apiClient.disableWorkflow(workflow.id)
        : await apiClient.enableWorkflow(workflow.id);
      setWorkflows((prev) =>
        prev.map((w) => (w.id === updated.id ? updated : w))
      );
    } catch (err) {
      setError((err as Error).message);
    }
  };

  const handleTest = async (id: string) => {
    try {
      await apiClient.testWorkflow(id);
      setError(null);
    } catch (err) {
      setError((err as Error).message);
    }
  };

  if (showBuilder) {
    return (
      <div className="min-h-screen bg-black p-8">
        <div className="mx-auto max-w-4xl">
          <h1 className="mb-6 text-2xl font-bold text-white">
            {editingWorkflow ? 'Edit Workflow' : 'Create Workflow'}
          </h1>
          <WorkflowBuilder
            workflow={editingWorkflow || undefined}
            onSave={handleSave}
            onCancel={() => {
              setShowBuilder(false);
              setEditingWorkflow(null);
            }}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black p-8">
      <div className="mx-auto max-w-4xl">
        {/* Header */}
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Workflows</h1>
            <p className="mt-1 text-sm text-gray-400">
              Automate actions with triggers and steps
            </p>
          </div>
          <button
            onClick={handleCreate}
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
          >
            Create Workflow
          </button>
        </div>

        {/* Error */}
        {error && (
          <div className="mb-4 rounded-lg border border-red-700 bg-red-900/50 p-3 text-sm text-red-300">
            {error}
            <button
              onClick={() => setError(null)}
              className="ml-2 text-red-400 hover:text-red-300"
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Loading */}
        {loading && (
          <div className="py-12 text-center text-gray-500">Loading workflows...</div>
        )}

        {loadingWorkflow && (
          <div className="py-12 text-center text-gray-500">Loading workflow details...</div>
        )}

        {/* Empty */}
        {!loading && workflows.length === 0 && (
          <div className="rounded-lg border border-dashed border-gray-700 p-12 text-center">
            <p className="text-gray-400">No workflows yet</p>
            <p className="mt-1 text-sm text-gray-500">
              Create your first workflow to automate actions
            </p>
          </div>
        )}

        {/* Workflow list */}
        {!loading && workflows.length > 0 && (
          <div className="space-y-2">
            {workflows.map((workflow) => (
              <div
                key={workflow.id}
                className="rounded-lg border border-gray-800 bg-gray-900"
              >
                <div className="flex items-center gap-3 p-4">
                  {/* Enabled toggle */}
                  <button
                    onClick={() => handleToggleEnabled(workflow)}
                    className={`relative h-6 w-11 shrink-0 rounded-full transition-colors ${
                      workflow.enabled ? 'bg-blue-600' : 'bg-gray-700'
                    }`}
                    title={workflow.enabled ? 'Disable workflow' : 'Enable workflow'}
                  >
                    <span
                      className={`absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
                        workflow.enabled ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>

                  {/* Info */}
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleEdit(workflow.id)}
                        className="truncate text-sm font-medium text-white hover:text-blue-400"
                      >
                        {workflow.name}
                      </button>
                      <span
                        className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${
                          TRIGGER_COLORS[workflow.trigger_type] || 'bg-gray-500/20 text-gray-400 border-gray-500/30'
                        }`}
                      >
                        {TRIGGER_LABELS[workflow.trigger_type] || workflow.trigger_type}
                      </span>
                    </div>
                    {workflow.description && (
                      <p className="mt-0.5 truncate text-xs text-gray-500">
                        {workflow.description}
                      </p>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() =>
                        setShowHistory(showHistory === workflow.id ? null : workflow.id)
                      }
                      className="rounded p-1.5 text-gray-400 hover:bg-gray-800 hover:text-white"
                      title="View executions"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleTest(workflow.id)}
                      className="rounded p-1.5 text-gray-400 hover:bg-gray-800 hover:text-white"
                      title="Test workflow"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleEdit(workflow.id)}
                      className="rounded p-1.5 text-gray-400 hover:bg-gray-800 hover:text-white"
                      title="Edit workflow"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleDelete(workflow.id)}
                      className="rounded p-1.5 text-gray-400 hover:bg-gray-800 hover:text-red-400"
                      title="Delete workflow"
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>

                {/* Execution history panel */}
                {showHistory === workflow.id && (
                  <div className="border-t border-gray-800 p-4">
                    <WorkflowExecutionHistory workflowId={workflow.id} />
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
