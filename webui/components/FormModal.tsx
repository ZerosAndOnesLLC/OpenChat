'use client';

import { useState } from 'react';
import type { WorkflowForm, FormFieldDefinition, FormFieldType } from '@/lib/types';

interface FormModalProps {
  form: WorkflowForm;
  onSubmit: (data: Record<string, unknown>) => void;
  onClose: () => void;
}

export default function FormModal({ form, onSubmit, onClose }: FormModalProps) {
  const [formData, setFormData] = useState<Record<string, unknown>>(() => {
    const initial: Record<string, unknown> = {};
    for (const field of form.fields) {
      if (field.type === 'multi_select') {
        initial[field.name] = [];
      } else {
        initial[field.name] = '';
      }
    }
    return initial;
  });

  const inputClass =
    'w-full rounded-lg border border-gray-700 bg-gray-900 px-3 py-2 text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500';
  const labelClass = 'mb-1 block text-sm font-medium text-gray-300';

  function handleChange(name: string, value: unknown) {
    setFormData((prev) => ({ ...prev, [name]: value }));
  }

  function handleMultiSelectToggle(name: string, option: string) {
    setFormData((prev) => {
      const current = (prev[name] as string[]) || [];
      const next = current.includes(option)
        ? current.filter((v) => v !== option)
        : [...current, option];
      return { ...prev, [name]: next };
    });
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    for (const field of form.fields) {
      if (!field.required) continue;
      const value = formData[field.name];
      if (field.type === 'multi_select') {
        if (!Array.isArray(value) || value.length === 0) return;
      } else {
        if (value === '' || value === undefined || value === null) return;
      }
    }

    onSubmit(formData);
  }

  function renderField(field: FormFieldDefinition) {
    switch (field.type as FormFieldType) {
      case 'text':
        return (
          <input
            type="text"
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            placeholder={field.placeholder || ''}
            onChange={(e) => handleChange(field.name, e.target.value)}
          />
        );

      case 'textarea':
        return (
          <textarea
            rows={4}
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            placeholder={field.placeholder || ''}
            onChange={(e) => handleChange(field.name, e.target.value)}
          />
        );

      case 'select':
        return (
          <select
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            onChange={(e) => handleChange(field.name, e.target.value)}
          >
            <option value="">{field.placeholder || 'Select an option'}</option>
            {field.options?.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        );

      case 'multi_select':
        return (
          <div className="space-y-2">
            {field.options?.map((option) => {
              const checked = ((formData[field.name] as string[]) || []).includes(option);
              return (
                <label key={option} className="flex items-center gap-2 text-sm text-gray-300">
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={() => handleMultiSelectToggle(field.name, option)}
                    className="rounded border-gray-600 bg-gray-800 text-blue-600 focus:ring-blue-500"
                  />
                  {option}
                </label>
              );
            })}
          </div>
        );

      case 'date':
        return (
          <input
            type="date"
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            onChange={(e) => handleChange(field.name, e.target.value)}
          />
        );

      case 'user_picker':
        return (
          <input
            type="text"
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            placeholder="Enter user ID"
            onChange={(e) => handleChange(field.name, e.target.value)}
          />
        );

      case 'channel_picker':
        return (
          <input
            type="text"
            className={inputClass}
            value={(formData[field.name] as string) || ''}
            placeholder="Enter channel ID"
            onChange={(e) => handleChange(field.name, e.target.value)}
          />
        );

      default:
        return null;
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-lg rounded-xl border border-gray-700 bg-gray-900 shadow-2xl">
        <div className="flex items-center justify-between border-b border-gray-800 px-6 py-4">
          <h2 className="text-lg font-semibold text-white">{form.title}</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white"
            aria-label="Close"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4 px-6 py-4">
          {form.fields.map((field) => (
            <div key={field.name}>
              <label className={labelClass}>
                {field.label}
                {field.required && <span className="ml-1 text-red-400">*</span>}
              </label>
              {renderField(field)}
            </div>
          ))}

          <div className="flex justify-end gap-3 border-t border-gray-800 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg border border-gray-600 px-4 py-2 text-sm font-medium text-gray-300 hover:bg-gray-800"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
            >
              Submit
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
