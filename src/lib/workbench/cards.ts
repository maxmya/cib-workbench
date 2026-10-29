// SPDX-License-Identifier: MIT

/**
 * Home's cards under `FEATURES.workbenchWide` (FEAT-183).
 *
 * The order a person sets, in onboarding or *Customise Home*, is the order of
 * these Workbench-wide cards only. The repository cards (`working`,
 * `branches`, `commits`, `heat`, `farm`) are drawn in the *Loaded* band while
 * something is loaded, and are not in this list. The ids are `HomeLayout`'s,
 * so Settings › Home, onboarding and *Customise Home* edit one thing.
 */

import type { HomeCard, HomeCardId } from '$lib/home/layout';

export interface WideCard extends HomeCard {
	/** One line under the name, where the order is chosen. */
	description: string;
}

export const WIDE_HOME_CARDS: WideCard[] = [
	{ id: 'needs', label: 'Needs you', wide: true, description: 'Reviews, failed pipelines, issues' },
	{ id: 'requests', label: 'Merge requests', description: 'Open, in what you follow' },
	{ id: 'pipelines', label: 'Pipelines', description: 'Latest on default branches and yours' },
	{ id: 'issues', label: 'Issues', description: 'Assigned to you' },
	{ id: 'calls', label: 'Calls', description: 'Workbench-wide collections' },
	{ id: 'releases', label: 'Releases', wide: true, description: 'Latest of each project' }
];

/** Drawn in the Loaded band, never in the ordered list. */
export const LOADED_CARDS: HomeCardId[] = ['working', 'branches', 'commits', 'heat', 'farm'];
