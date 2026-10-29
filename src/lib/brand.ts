// SPDX-License-Identifier: MIT

/**
 * Everything a user sees that differs between Spagitty and Workbench.
 *
 * Workbench is CIB's build of Spagitty. It keeps Spagitty's internal names
 * (crates, modules, the `.spagitty/` folder, settings ids) so that Spagitty's
 * `main` keeps merging in; only what a person reads changes, and it reads it
 * from here. `crates/spagitty-core/src/brand.rs` holds the same values for
 * the backend. `docs/workbench-sync.md` says how to keep both true after a
 * merge.
 *
 * The wording is the approved wording in `docs/branding.md` (Workbench
 * section). Do not write new copy here.
 */

import cibLogoColor from '../../assets/brand/cib/cib-logo-color.png';
import cibLogoReversed from '../../assets/brand/cib/cib-logo-reversed.png';
import cibLogoReversedTagline from '../../assets/brand/cib/cib-logo-reversed-tagline.png';
import cibGlobe from '../../assets/brand/cib/cib-globe-round.png';
import workbenchIcon from '../../assets/brand/cib/workbench-icon-64.png';

/** The name in every label, title and message. */
export const PRODUCT_NAME = 'Workbench';
/** The name in full, where the organisation is not otherwise on screen. */
export const PRODUCT_LONG = 'CIB Workbench';
export const ORG = 'CIB';
/** The SPDX identifier of Workbench's own code. */
export const LICENCE = 'MIT';
/**
 * The Spagitty release this build is based on, and the Spagitty commit its
 * tree was cut from. Updated by every sync (`docs/workbench-sync.md`).
 */
export const BASED_ON = {
	name: 'Spagitty',
	version: '4.0.1',
	commit: '86c1a2ed2e4130a7983d7c1146d75ad12f3c540b'
};

/** Where Workbench behaves differently from Spagitty: what it leaves out, and what it adds. */
export const FEATURES = {
	/** Workbench updates itself from its own GitHub releases (FEAT-187). */
	updateCheck: true,
	/** The author's own section for the delight layer. */
	godMode: false,
	/**
	 * Workbench-wide (2.0.0): onboarding, Home across the CIB GitLab projects a
	 * person follows, and Load repository in place of the tab strip
	 * (FEAT-180 to FEAT-184). Off is Spagitty's window, tabs and all.
	 */
	workbenchWide: true
};

/**
 * CIB's GitLab, built in (FEAT-181): the one host the connect screen offers,
 * and the only one the token page link is built from.
 */
export const GITLAB = {
	/** Lower case, no scheme. GitLab Enterprise Edition 16.6.2. */
	host: 'gitlab.apps.ocp-nonprod-01.hodomain.local',
	/** Confirmed signed in on 2026-10-28. `/-/user_settings/` is a 404 on 16.6.2. */
	tokenPath: '/-/profile/personal_access_tokens',
	tokenName: 'Workbench',
	scopes: ['api', 'read_repository', 'write_repository']
} as const;

/**
 * GitLab's token page, with the name and scopes in the query. 16.6.2 keeps the
 * form behind *Add new token* and does not fill it in, so onboarding says what
 * to do there; a GitLab that honours the query fills it in.
 */
export function tokenPageUrl(
	gitlab: { host: string; tokenPath: string; tokenName: string; scopes: readonly string[] } = GITLAB
): string {
	const name = encodeURIComponent(gitlab.tokenName);
	return `https://${gitlab.host}${gitlab.tokenPath}?name=${name}&scopes=${gitlab.scopes.join(',')}`;
}

/** The launch screen's line. */
export const SPLASH_LINE = 'Git, reviews and agents in one place';

/** The internal-use notice: the launch screen and Settings → About. */
export const NOTICE = {
	badge: 'INTERNAL USE ONLY',
	lead: 'In development and provided as is. Use at your own risk.',
	trademark:
		'CIB and the CIB logo are trademarks of Commercial International Bank (Egypt) S.A.E.',
	/** Follows the product name and version: "Workbench 1.0.0 is based on …". */
	licence: 'is based on Spagitty and released under the MIT licence.'
};

/**
 * CIB's own files. Never redraw, recolour or crop them, and pick the logo by
 * the theme's mode rather than with a CSS filter.
 */
export const BRAND_ASSETS = {
	/** For light grounds. */
	logoColor: cibLogoColor,
	/** For dark grounds. */
	logoReversed: cibLogoReversed,
	/** For dark grounds, with "The bank to trust". */
	logoReversedTagline: cibLogoReversedTagline,
	/** The wordmark's "o". */
	globe: cibGlobe,
	/** The app icon, small: `tools/make-workbench-icon.py` draws it. */
	icon: workbenchIcon
};
