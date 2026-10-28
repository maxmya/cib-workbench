// SPDX-License-Identifier: MIT

/** The Hooks screen's typed command boundary (FEAT-137). */
import { invoke } from '@tauri-apps/api/core';
import type { Builder, Definition, Run, Stage, Tried } from './builder-types';

export const load = (): Promise<Builder> => invoke('hooks_builder');
export const save = (definition: Definition, install: boolean, imported: boolean): Promise<void> => invoke('hooks_save', { definition, install, imported });
export const importManager = (): Promise<Definition> => invoke('hooks_import');
export const matches = (paths: string[]): Promise<string[]> => invoke('hooks_matches', { paths });
export const runs = (): Promise<Run[]> => invoke('hooks_runs');
export const run = (id: string): Promise<Run> => invoke('hooks_run', { id });
export const log = (id: string, block: string): Promise<string> => invoke('hooks_log', { id, block });
export const dryRun = (definition: Definition, stage: Stage): Promise<Tried> => invoke('hooks_dry_run', { definition, stage });
export const tryMessage = (definition: Definition, message: string): Promise<Tried> => invoke('hooks_try_message', { definition, message });
export const stop = (id: string): Promise<void> => invoke('hooks_stop', { id });
export const undoRestage = (id: string): Promise<void> => invoke('hooks_undo_restage', { id });

/** What a manager runs, read, explained and edited (FEAT-171). */
export interface Askable { id: string; name: string }
export interface Explanation { agent: string; text: string }
export const agents = (): Promise<Askable[]> => invoke('hooks_agents');
export const explain = (path: string, agent: string | null): Promise<Explanation> => invoke('hooks_explain', { path, agent });
export const write = (path: string, text: string): Promise<void> => invoke('hooks_write', { path, text });
/** Add a command to the hook Husky runs (FEAT-176); answers the file it went in. */
export const add = (hook: string, command: string): Promise<string> => invoke('hooks_add', { hook, command });
