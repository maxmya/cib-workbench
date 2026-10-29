// SPDX-License-Identifier: MIT

/**
 * First-launch onboarding's steps (2D, FEAT-180), pure, so Back and Next are
 * tested without a window.
 *
 * Four steps, *Welcome · GitLab · Token · Home*, and Welcome has three pages.
 * With an account for CIB GitLab already connected — somebody upgrading from
 * 1.x, or running setup again — GitLab and Token are passed over.
 */

export const STEPS = ['Welcome', 'GitLab', 'Token', 'Home'] as const;
export const WELCOME_PAGES = 3;

export interface Stage {
	/** 1 to 4. */
	step: number;
	/** 0 to 2 on Welcome; 0 elsewhere. */
	page: number;
}

export const FIRST: Stage = { step: 1, page: 0 };

export interface Rules {
	/** An account is connected: steps 2 and 3 are passed over. */
	connected: boolean;
	/** The token step's check passed. */
	checked: boolean;
}

/** The stage after `stage`, or null when Next would finish. */
export function next(stage: Stage, rules: Rules): Stage | null {
	if (stage.step === 1 && stage.page < WELCOME_PAGES - 1) return { step: 1, page: stage.page + 1 };
	if (stage.step === 1) return { step: rules.connected ? 4 : 2, page: 0 };
	if (stage.step === 2) return { step: 3, page: 0 };
	if (stage.step === 3) return rules.checked ? { step: 4, page: 0 } : stage;
	return null;
}

export function back(stage: Stage, rules: Rules): Stage {
	if (stage.step === 1) return { step: 1, page: Math.max(0, stage.page - 1) };
	if (stage.step === 2 || (stage.step === 4 && rules.connected)) return { step: 1, page: WELCOME_PAGES - 1 };
	return { step: stage.step - 1, page: 0 };
}

/** Next is held back only on the token step, until the check passes. */
export function canGoOn(stage: Stage, rules: Rules): boolean {
	return stage.step !== 3 || rules.checked;
}

export type Mark = 'done' | 'current' | 'todo';

/** How each step's mark is drawn. Passed-over steps count as done. */
export function marks(stage: Stage, rules: Rules): Mark[] {
	return STEPS.map((_, index) => {
		const step = index + 1;
		if (step === stage.step) return 'current';
		if (step < stage.step || (rules.connected && (step === 2 || step === 3))) return 'done';
		return 'todo';
	});
}
