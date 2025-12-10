'use client';

import { useState, useEffect, useMemo } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import {
  ChevronDown,
  ChevronRight,
  ChevronLeft,
  Settings,
} from 'lucide-react';

export interface SettingsItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  component: React.ReactNode;
}

export interface SettingsCategory {
  id: string;
  label: string;
  icon: React.ReactNode;
  items: SettingsItem[];
}

interface SettingsLayoutProps {
  categories: SettingsCategory[];
  singleItems?: SettingsItem[];
  defaultSection?: string;
}

function getInitialSection(
  sectionParam: string | null,
  defaultSection: string | undefined,
  categories: SettingsCategory[],
  singleItems: SettingsItem[]
): string {
  if (sectionParam) return sectionParam;
  if (defaultSection) return defaultSection;
  if (categories.length > 0 && categories[0].items.length > 0) {
    return categories[0].items[0].id;
  }
  if (singleItems.length > 0) {
    return singleItems[0].id;
  }
  return '';
}

function getInitialExpandedCategories(categories: SettingsCategory[]): Set<string> {
  return new Set(categories.map(cat => cat.id));
}

export default function SettingsLayout({ categories, singleItems = [], defaultSection }: SettingsLayoutProps) {
  const router = useRouter();
  const searchParams = useSearchParams();

  // Get section from URL or use default
  const sectionParam = searchParams.get('section');

  // Compute initial values once
  const initialSection = useMemo(
    () => getInitialSection(sectionParam, defaultSection, categories, singleItems),
    [sectionParam, defaultSection, categories, singleItems]
  );

  const initialExpanded = useMemo(
    () => getInitialExpandedCategories(categories),
    [categories]
  );

  const [activeSection, setActiveSection] = useState<string>(initialSection);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(initialExpanded);
  const [showMobileNav, setShowMobileNav] = useState(true);

  // Update URL when section changes
  useEffect(() => {
    if (activeSection) {
      const params = new URLSearchParams(searchParams.toString());
      params.set('section', activeSection);
      router.replace(`/settings/?${params.toString()}`, { scroll: false });
    }
  }, [activeSection, router, searchParams]);

  const toggleCategory = (categoryId: string) => {
    setExpandedCategories(prev => {
      const newSet = new Set(prev);
      if (newSet.has(categoryId)) {
        newSet.delete(categoryId);
      } else {
        newSet.add(categoryId);
      }
      return newSet;
    });
  };

  const handleSelectItem = (itemId: string) => {
    setActiveSection(itemId);
    // On mobile, hide nav and show detail
    setShowMobileNav(false);
  };

  const handleBackToNav = () => {
    setShowMobileNav(true);
  };

  // Find the active component
  const getActiveComponent = () => {
    for (const category of categories) {
      for (const item of category.items) {
        if (item.id === activeSection) {
          return { component: item.component, label: item.label };
        }
      }
    }
    for (const item of singleItems) {
      if (item.id === activeSection) {
        return { component: item.component, label: item.label };
      }
    }
    return null;
  };

  const activeContent = getActiveComponent();

  return (
    <div className="flex h-full bg-gray-950">
      {/* Navigation Sidebar */}
      <nav
        className={`
          w-full md:w-72 lg:w-80 flex-shrink-0 border-r border-gray-800 bg-gray-900/50
          overflow-y-auto
          ${showMobileNav ? 'block' : 'hidden md:block'}
        `}
      >
        <div className="p-4">
          {/* Categories */}
          {categories.map((category) => (
            <div key={category.id} className="mb-2">
              {/* Category Header */}
              <button
                onClick={() => toggleCategory(category.id)}
                className="w-full flex items-center gap-3 px-3 py-2.5 text-left text-gray-300 hover:text-white hover:bg-gray-800/50 rounded-lg transition-colors"
              >
                <span className="text-gray-500">{category.icon}</span>
                <span className="flex-1 font-medium text-sm">{category.label}</span>
                {expandedCategories.has(category.id) ? (
                  <ChevronDown className="w-4 h-4 text-gray-500" />
                ) : (
                  <ChevronRight className="w-4 h-4 text-gray-500" />
                )}
              </button>

              {/* Category Items */}
              {expandedCategories.has(category.id) && (
                <div className="ml-4 mt-1 space-y-1">
                  {category.items.map((item) => (
                    <button
                      key={item.id}
                      onClick={() => handleSelectItem(item.id)}
                      className={`
                        w-full flex items-center gap-3 px-3 py-2 text-left rounded-lg transition-all
                        ${activeSection === item.id
                          ? 'bg-blue-600/20 text-blue-400 border-l-2 border-blue-500'
                          : 'text-gray-400 hover:text-white hover:bg-gray-800/50 border-l-2 border-transparent'
                        }
                      `}
                    >
                      <span className={activeSection === item.id ? 'text-blue-400' : 'text-gray-500'}>
                        {item.icon}
                      </span>
                      <span className="text-sm">{item.label}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))}

          {/* Single Items (no category) */}
          {singleItems.length > 0 && (
            <div className="mt-4 pt-4 border-t border-gray-800">
              {singleItems.map((item) => (
                <button
                  key={item.id}
                  onClick={() => handleSelectItem(item.id)}
                  className={`
                    w-full flex items-center gap-3 px-3 py-2.5 text-left rounded-lg transition-all
                    ${activeSection === item.id
                      ? 'bg-blue-600/20 text-blue-400'
                      : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                    }
                  `}
                >
                  <span className={activeSection === item.id ? 'text-blue-400' : 'text-gray-500'}>
                    {item.icon}
                  </span>
                  <span className="text-sm font-medium">{item.label}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      </nav>

      {/* Detail Panel */}
      <main
        className={`
          flex-1 overflow-y-auto
          ${showMobileNav ? 'hidden md:block' : 'block'}
        `}
      >
        {/* Mobile Back Button */}
        <div className="md:hidden sticky top-0 z-10 bg-gray-900 border-b border-gray-800 px-4 py-3">
          <button
            onClick={handleBackToNav}
            className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors"
          >
            <ChevronLeft className="w-5 h-5" />
            <span className="text-sm font-medium">Back to Settings</span>
          </button>
        </div>

        {/* Content */}
        <div className="p-6 lg:p-8">
          {activeContent ? (
            <div className="max-w-3xl">
              <h2 className="text-xl font-semibold text-white mb-6">{activeContent.label}</h2>
              {activeContent.component}
            </div>
          ) : (
            <div className="flex items-center justify-center h-64 text-gray-500">
              <div className="text-center">
                <Settings className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>Select a setting from the menu</p>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
