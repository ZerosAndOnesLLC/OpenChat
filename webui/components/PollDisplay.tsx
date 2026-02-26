'use client';

import { useState } from 'react';
import { useAuth } from '@/lib/auth';
import { apiClient } from '@/lib/api';
import { toastManager } from '@/lib/toast';
import type { Poll } from '@/lib/types';

interface PollDisplayProps {
  poll: Poll;
  messageId: string;
}

export default function PollDisplay({ poll, messageId }: PollDisplayProps) {
  const { user } = useAuth();
  const [voting, setVoting] = useState(false);
  const [localPoll, setLocalPoll] = useState(poll);
  const [closing, setClosing] = useState(false);

  // Update from parent when poll data changes
  if (poll.total_votes !== localPoll.total_votes || poll.closed !== localPoll.closed) {
    setLocalPoll(poll);
  }

  const hasVoted = localPoll.user_votes && localPoll.user_votes.length > 0;
  const maxVotes = Math.max(...localPoll.options.map((o) => o.votes), 1);

  const handleVote = async (optionIndex: number) => {
    if (localPoll.closed || voting) return;

    try {
      setVoting(true);
      const updated = await apiClient.votePoll(localPoll.id, optionIndex);
      setLocalPoll(updated);
    } catch (error) {
      toastManager.error('Failed to vote');
    } finally {
      setVoting(false);
    }
  };

  const handleRemoveVote = async () => {
    if (localPoll.closed || voting) return;
    try {
      setVoting(true);
      await apiClient.removePollVote(localPoll.id);
      // Refetch to get updated results
      const updated = await apiClient.getPoll(localPoll.id);
      setLocalPoll(updated);
    } catch (error) {
      toastManager.error('Failed to remove vote');
    } finally {
      setVoting(false);
    }
  };

  const handleClose = async () => {
    try {
      setClosing(true);
      const updated = await apiClient.closePoll(localPoll.id);
      setLocalPoll(updated);
    } catch (error) {
      toastManager.error('Failed to close poll');
    } finally {
      setClosing(false);
    }
  };

  const isCreator = user && user.id === localPoll.created_by;

  return (
    <div className="mt-2 p-3 bg-gray-800/50 rounded-lg border border-gray-700 max-w-md">
      <div className="flex items-center gap-2 mb-2">
        <span className="text-lg">&#x1F4CA;</span>
        <span className="text-sm font-semibold text-white">{localPoll.question}</span>
      </div>

      <div className="text-xs text-gray-500 mb-2">
        {localPoll.poll_type === 'single' ? 'Single choice' : 'Multiple choice'}
        {localPoll.anonymous && ' \u00b7 Anonymous'}
        {localPoll.closed && ' \u00b7 Closed'}
      </div>

      <div className="space-y-1.5">
        {localPoll.options.map((option) => {
          const isSelected = localPoll.user_votes?.includes(option.index);
          const percentage = localPoll.total_votes > 0
            ? Math.round((option.votes / localPoll.total_votes) * 100)
            : 0;

          return (
            <button
              key={option.index}
              onClick={() => handleVote(option.index)}
              disabled={localPoll.closed || voting}
              className={`w-full text-left relative rounded overflow-hidden transition-colors ${
                localPoll.closed ? 'cursor-default' : 'cursor-pointer hover:bg-gray-700/50'
              } ${isSelected ? 'ring-1 ring-blue-500' : ''}`}
            >
              {/* Progress bar background */}
              <div
                className={`absolute inset-0 ${isSelected ? 'bg-blue-900/40' : 'bg-gray-700/30'}`}
                style={{ width: `${percentage}%` }}
              />
              <div className="relative flex items-center justify-between px-3 py-1.5">
                <div className="flex items-center gap-2">
                  {isSelected && (
                    <svg className="h-3.5 w-3.5 text-blue-400 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  )}
                  <span className="text-sm text-gray-200">{option.text}</span>
                </div>
                <span className="text-xs text-gray-400 ml-2 flex-shrink-0">
                  {option.votes} {option.votes === 1 ? 'vote' : 'votes'} ({percentage}%)
                </span>
              </div>
            </button>
          );
        })}
      </div>

      <div className="mt-2 flex items-center justify-between">
        <span className="text-xs text-gray-500">
          {localPoll.total_votes} total {localPoll.total_votes === 1 ? 'vote' : 'votes'}
        </span>
        <div className="flex items-center gap-2">
          {hasVoted && !localPoll.closed && (
            <button
              onClick={handleRemoveVote}
              disabled={voting}
              className="text-xs text-gray-400 hover:text-white transition-colors"
            >
              Remove vote
            </button>
          )}
          {isCreator && !localPoll.closed && (
            <button
              onClick={handleClose}
              disabled={closing}
              className="text-xs text-red-400 hover:text-red-300 transition-colors"
            >
              {closing ? 'Closing...' : 'Close poll'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
