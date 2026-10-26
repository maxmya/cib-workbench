// SPDX-License-Identifier: MIT

/**
 * Typed wrappers around Calls' Tauri commands (FEAT-160).
 *
 * The only Calls module that calls `invoke`, as `$lib/recipes/api.ts` is for
 * recipes. Nothing here reaches a network: a send is made in Rust and the
 * answer arrives as an event.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
	BroughtIn,
	Call,
	CallsSettings,
	Choices,
	Collection,
	Found,
	Environment,
	Environments,
	HistoryEntry,
	Loaded,
	Pasted,
	PersonState,
	Private,
	Quoting,
	ShareAsk,
	Shared,
	SharePlan,
	WsdlView
} from './types';

/**
 * Where the collections on screen live (FEAT-170): beside the open
 * repository's code, or Workbench-wide, in application data, there with any
 * repository open or none.
 */
export type Scope = 'repository' | 'global';

/**
 * The scope every command below reads and writes. Set by the store when the
 * screen switches, so no caller can write one scope's collection into the
 * other's folder by passing the wrong word.
 */
let scope: Scope = 'repository';

export function setScope(next: Scope): void {
	scope = next;
}

export function currentScope(): Scope {
	return scope;
}

/** A piece of an event stream, as it arrived. */
export const CHUNK_EVENT = 'calls-chunk';
/** A send ended: answered, failed or cancelled. */
export const DONE_EVENT = 'calls-done';

export function load(): Promise<Loaded> {
	return invoke('calls_load', { scope });
}

/** Save a collection; answers with the file stem it was written under. */
export function saveCollection(collection: Collection, stem: string | null, was: string | null): Promise<string> {
	return invoke('calls_save_collection', { scope, collection, stem, was });
}

export function deleteCollection(stem: string): Promise<void> {
	return invoke('calls_delete_collection', { scope, stem });
}

export function saveState(person: PersonState): Promise<void> {
	return invoke('calls_save_state', { scope, person });
}

/**
 * Build and start sending `call`. Rejects at once, with nothing sent, when a
 * variable has no value; otherwise the answer is a `calls-done` event.
 */
export function send(ask: { sendId: string; stem: string | null; call: Call; environment: string | null }): Promise<void> {
	return invoke('calls_send', { scope, ask });
}

export function cancel(sendId: string): Promise<void> {
	return invoke('calls_cancel', { sendId });
}

/** Write a response's bytes, as they arrived, to `path`. */
export function saveBody(sendId: string, path: string): Promise<void> {
	return invoke('calls_save_body', { sendId, path });
}

export function history(): Promise<HistoryEntry[]> {
	return invoke('calls_history', { scope });
}

export function historyBody(id: string): Promise<string | null> {
	return invoke('calls_history_body', { scope, id });
}

export function clearHistory(): Promise<void> {
	return invoke('calls_history_clear', { scope });
}

// --- Settings › Calls (FEAT-165) ---

/** Settings › Calls as a new install has them: what *Reset* goes back to. */
export function defaultSettings(): Promise<CallsSettings> {
	return invoke('calls_default_settings');
}

/** The keys the open repository's `settings.json` overrides. */
export function overridden(): Promise<string[]> {
	return invoke('calls_overridden');
}

// --- SOAP (FEAT-163) ---

/** A WSDL in the repository, by its path from the working tree. */
export function wsdl(path: string): Promise<WsdlView> {
	return invoke('calls_wsdl', { path });
}

/** An operation's envelope made again from the WSDL, keeping typed values; and its action. */
export function rebuild(ask: {
	wsdl: string;
	service: string;
	port: string | null;
	operation: string;
	version: string | null;
	envelope: string | null;
}): Promise<[string, string]> {
	return invoke('calls_rebuild', { ask });
}

export function importWsdlUrl(url: string): Promise<Found> {
	return invoke('calls_import_wsdl_url', { scope, url });
}

// --- Environments, secrets and kept values (FEAT-161) ---

export function environments(): Promise<Environments> {
	return invoke('calls_environments', { scope });
}

/**
 * Save an environment. `was` is its stem before a rename; `copyFrom` the
 * environment it duplicates, whose keychain values it is given.
 */
export function saveEnvironment(
	environment: Environment,
	was: string | null,
	copyFrom: string | null = null
): Promise<Environments> {
	return invoke('calls_save_environment', { scope, environment, was, copyFrom });
}

export function deleteEnvironment(stem: string): Promise<Environments> {
	return invoke('calls_delete_environment', { scope, stem });
}

export function saveShared(shared: Shared): Promise<Environments> {
	return invoke('calls_save_shared', { scope, shared });
}

/** An environment's kept values, and which secrets have a value; `null` is the shared variables. */
export function privateOf(environment: string | null): Promise<Private> {
	return invoke('calls_private', { scope, environment });
}

/** One secret's value, while the eye is held. */
export function reveal(environment: string | null, name: string): Promise<string | null> {
	return invoke('calls_reveal', { scope, environment, name });
}

/** Set a secret's value, or remove it with `null`. */
export function setPrivate(environment: string | null, name: string, value: string | null): Promise<void> {
	return invoke('calls_set_private', { scope, environment, name, value });
}

export function keep(ask: {
	environment: string | null;
	var: string;
	value: string;
	callId: string;
	call: string;
	method: string;
	from: string;
}): Promise<Environments> {
	return invoke('calls_keep', { scope, ask });
}

export function forgetKept(environment: string | null, name: string): Promise<Environments> {
	return invoke('calls_forget_kept', { scope, environment, name });
}

// --- Import (FEAT-162) ---

/** A scan's progress. */
export const SCAN_EVENT = 'calls-scan';
/** A scan ended. */
export const SCAN_DONE_EVENT = 'calls-scan-done';

/** Scan the working tree; the answer is a `calls-scan-done` event. */
export function scan(scanId: string): Promise<void> {
	return invoke('calls_scan', { scanId });
}

export function cancelScan(scanId: string): Promise<void> {
	return invoke('calls_scan_cancel', { scanId });
}

/** What the last scan found, marked against the collections as they are now. */
export function found(): Promise<Found | null> {
	return invoke('calls_found', { scope });
}

/** The collection files in the repository, read without the code: opened in place. */
export function foundFiles(): Promise<Found> {
	return invoke('calls_found_files');
}

export function importFile(path: string): Promise<Found> {
	return invoke('calls_import_file', { scope, path });
}

export function importText(text: string): Promise<Found> {
	return invoke('calls_import_text', { scope, text });
}

export function bringIn(ask: {
	ids: string[];
	into: string | null;
	name: string | null;
	choices: Choices;
	serverAddress: boolean;
}): Promise<BroughtIn> {
	return invoke('calls_bring_in', { scope, ask });
}

// --- cURL (FEAT-166) ---

/** The call as one `curl` command; secrets stay `{{name}}` unless `secrets`. */
export function curl(ask: {
	stem: string | null;
	call: Call;
	environment: string | null;
	quoting: Quoting;
	secrets: boolean;
}): Promise<string> {
	return invoke('calls_curl', { scope, ask });
}

export function curlParse(text: string): Promise<Pasted> {
	return invoke('calls_curl_parse', { text });
}

// --- Share (FEAT-164) ---

/** What a share would write, with sizes. */
export function sharePlan(ask: ShareAsk): Promise<SharePlan> {
	return invoke('calls_share_plan', { scope, ask });
}

/** Write the file or zip to `ask.path`, or commit; answers with the path or the commit's id. */
export function share(ask: ShareAsk): Promise<string> {
	return invoke('calls_share', { scope, ask });
}
