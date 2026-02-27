'use client';

import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '@/lib/api';
import type { WorkflowExecution, WorkflowExecutionStep, ExecutionStatus, StepStatus } from '@/lib/types';

interface WorkflowExecutionHistoryProps {
  workflowId: string;
}

const PAGE_SIZE = 20;

function executionStatusClasses(status: ExecutionStatus): string {
  switch (status) {
    case 'running':
      return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30';
    case 'completed':
      return 'bg-green-500/20 text-green-400 border border-green-500/30';
    case 'failed':
      return 'bg-red-500/20 text-red-400 border border-red-500/30';
    default:
      return 'bg-gray-500/20 text-gray-400 border border-gray-500/30';
  }
}

function stepStatusClasses(status: StepStatus): string {
  switch (status) {
    case 'running':
      return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30';
    case 'completed':
      return 'bg-green-500/20 text-green-400 border border-green-500/30';
    case 'failed':
      return 'bg-red-500/20 text-red-400 border border-red-500/30';
    case 'pending':
    default:
      return 'bg-gray-500/20 text-gray-400 border border-gray-500/30';
  }
}

function formatTimestamp(ts: string): string {
  const date = new Date(ts);
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

function formatDuration(startedAt: string, completedAt: string): string {
  const ms = new Date(completedAt).getTime() - new Date(startedAt).getTime();
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  return `${minutes}m ${seconds}s`;
}

interface StepTimelineProps {
  steps: WorkflowExecutionStep[];
}

function StepTimeline({ steps }: StepTimelineProps) {
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(new Set());

  const toggleStep = (stepId: string) => {
    setExpandedSteps((prev) => {
      const next = new Set(prev);
      if (next.has(stepId)) {
        next.delete(stepId);
      } else {
        next.add(stepId);
      }
      return next;
    });
  };

  if (steps.length === 0) {
    return (
      <div className="px-4 py-3 text-sm text-gray-500">
        No steps recorded for this execution.
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-800">
      {steps.map((step, index) => {
        const isExpanded = expandedSteps.has(step.id);
        const hasDuration = step.started_at && step.completed_at;
        const hasInputOutput =
          (step.input_data && Object.keys(step.input_data).length > 0) ||
          (step.output_data && Object.keys(step.output_data).length > 0);

        return (
          <div key={step.id} className="px-4 py-3">
            <button
              onClick={() => toggleStep(step.id)}
              className="flex w-full items-start gap-3 text-left"
            >
              <div className="flex flex-col items-center gap-1">
                <span className="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-gray-700 text-xs font-medium text-gray-400">
                  {index + 1}
                </span>
                {index < steps.length - 1 && (
                  <div className="h-full w-px bg-gray-700" style={{ minHeight: '12px' }} />
                )}
              </div>
              <div className="flex flex-1 flex-wrap items-center gap-2 min-w-0">
                <span
                  className={`inline-flex items-center rounded px-2 py-0.5 text-xs font-medium ${stepStatusClasses(step.status)}`}
                >
                  {step.status}
                </span>
                <span className="text-xs text-gray-500 font-mono truncate">
                  step_id: {step.step_id}
                </span>
                {hasDuration && (
                  <span className="text-xs text-gray-500">
                    {formatDuration(step.started_at!, step.completed_at!)}
                  </span>
                )}
                {(hasInputOutput || step.error_message) && (
                  <svg
                    className={`ml-auto h-4 w-4 flex-shrink-0 text-gray-500 transition-transform duration-150 ${isExpanded ? 'rotate-180' : ''}`}
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                )}
              </div>
            </button>

            {isExpanded && (
              <div className="mt-3 ml-9 space-y-3">
                {step.error_message && (
                  <div className="rounded bg-red-500/10 border border-red-500/20 px-3 py-2">
                    <p className="text-xs font-medium text-red-400 mb-1">Error</p>
                    <p className="text-xs text-red-300 font-mono whitespace-pre-wrap break-words">
                      {step.error_message}
                    </p>
                  </div>
                )}
                {step.input_data && Object.keys(step.input_data).length > 0 && (
                  <div>
                    <p className="text-xs font-medium text-gray-400 mb-1">Input</p>
                    <pre className="rounded bg-gray-800 border border-gray-700 px-3 py-2 text-xs text-gray-300 overflow-x-auto whitespace-pre-wrap break-words">
                      {JSON.stringify(step.input_data, null, 2)}
                    </pre>
                  </div>
                )}
                {step.output_data && Object.keys(step.output_data).length > 0 && (
                  <div>
                    <p className="text-xs font-medium text-gray-400 mb-1">Output</p>
                    <pre className="rounded bg-gray-800 border border-gray-700 px-3 py-2 text-xs text-gray-300 overflow-x-auto whitespace-pre-wrap break-words">
                      {JSON.stringify(step.output_data, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

interface ExecutionRowProps {
  execution: WorkflowExecution;
  workflowId: string;
}

function ExecutionRow({ execution, workflowId }: ExecutionRowProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [steps, setSteps] = useState<WorkflowExecutionStep[]>(execution.steps ?? []);
  const [loadingSteps, setLoadingSteps] = useState(false);
  const [stepsError, setStepsError] = useState<string | null>(null);

  const handleToggle = async () => {
    if (!isExpanded && steps.length === 0) {
      setLoadingSteps(true);
      setStepsError(null);
      try {
        const full = await apiClient.getWorkflowExecution(workflowId, execution.id);
        setSteps(full.steps ?? []);
      } catch (err) {
        setStepsError(err instanceof Error ? err.message : 'Failed to load steps');
      } finally {
        setLoadingSteps(false);
      }
    }
    setIsExpanded((prev) => !prev);
  };

  const hasDuration = execution.started_at && execution.completed_at;

  return (
    <div className="rounded-lg border border-gray-800 bg-gray-900 overflow-hidden">
      <button
        onClick={handleToggle}
        className="flex w-full items-start gap-3 px-4 py-3 text-left hover:bg-gray-800/50 transition-colors duration-100"
      >
        <div className="flex flex-1 flex-wrap items-center gap-2 min-w-0">
          <span
            className={`inline-flex items-center rounded px-2 py-0.5 text-xs font-medium ${executionStatusClasses(execution.status)}`}
          >
            {execution.status}
          </span>
          <span className="text-sm text-gray-300 font-mono truncate">
            {execution.id}
          </span>
          <span className="text-xs text-gray-500 ml-auto flex-shrink-0">
            {formatTimestamp(execution.started_at)}
          </span>
          {hasDuration && (
            <span className="text-xs text-gray-500 flex-shrink-0">
              {formatDuration(execution.started_at, execution.completed_at!)}
            </span>
          )}
        </div>
        <svg
          className={`h-4 w-4 flex-shrink-0 text-gray-500 transition-transform duration-150 mt-0.5 ${isExpanded ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {execution.error_message && (
        <div className="px-4 pb-3">
          <p className="text-xs text-red-400 font-mono">
            Error: {execution.error_message}
          </p>
        </div>
      )}

      {isExpanded && (
        <div className="border-t border-gray-800">
          {loadingSteps ? (
            <div className="flex items-center justify-center py-6 text-sm text-gray-500">
              Loading steps...
            </div>
          ) : stepsError ? (
            <div className="px-4 py-3 text-sm text-red-400">
              {stepsError}
            </div>
          ) : (
            <StepTimeline steps={steps} />
          )}
        </div>
      )}
    </div>
  );
}

export default function WorkflowExecutionHistory({ workflowId }: WorkflowExecutionHistoryProps) {
  const [executions, setExecutions] = useState<WorkflowExecution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  const fetchExecutions = useCallback(async (currentOffset: number, replace: boolean) => {
    if (replace) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }
    setError(null);

    try {
      const results = await apiClient.listWorkflowExecutions(workflowId, PAGE_SIZE, currentOffset);
      setHasMore(results.length === PAGE_SIZE);
      setExecutions((prev) => (replace ? results : [...prev, ...results]));
      setOffset(currentOffset + results.length);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load execution history');
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, [workflowId]);

  useEffect(() => {
    fetchExecutions(0, true);
  }, [fetchExecutions]);

  const handleRefresh = () => {
    setOffset(0);
    fetchExecutions(0, true);
  };

  const handleLoadMore = () => {
    fetchExecutions(offset, false);
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-gray-200">Execution History</h3>
        <button
          onClick={handleRefresh}
          disabled={loading}
          className="flex items-center gap-1.5 rounded bg-gray-800 px-3 py-1.5 text-xs text-gray-300 hover:bg-gray-700 hover:text-white disabled:opacity-50 transition-colors duration-100"
        >
          <svg
            className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          Refresh
        </button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center rounded-lg border border-gray-800 bg-gray-900 py-12 text-sm text-gray-500">
          Loading execution history...
        </div>
      ) : error ? (
        <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-4 text-sm text-red-400">
          {error}
        </div>
      ) : executions.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-gray-800 bg-gray-900 py-12 gap-2">
          <svg
            className="h-8 w-8 text-gray-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <p className="text-sm text-gray-500">No executions yet</p>
          <p className="text-xs text-gray-600">This workflow has not been triggered yet.</p>
        </div>
      ) : (
        <>
          <div className="flex flex-col gap-2">
            {executions.map((execution) => (
              <ExecutionRow
                key={execution.id}
                execution={execution}
                workflowId={workflowId}
              />
            ))}
          </div>

          {hasMore && (
            <div className="flex justify-center">
              <button
                onClick={handleLoadMore}
                disabled={loadingMore}
                className="rounded bg-gray-800 px-4 py-2 text-sm text-gray-300 hover:bg-gray-700 hover:text-white disabled:opacity-50 transition-colors duration-100"
              >
                {loadingMore ? 'Loading...' : 'Load More'}
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
