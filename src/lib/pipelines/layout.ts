// SPDX-License-Identifier: MIT
import { PIPELINE_CARD_HEIGHT as H, PIPELINE_CARD_WIDTH as W, PIPELINE_COLUMN_PITCH as X, PIPELINE_ROW_PITCH as Y } from '$lib/metrics';
import type { Job } from './types';

export interface Card { key: string; job: Job; x: number; y: number; subtitle: string; count: number }
export interface Edge { key: string; from: string; to: string; path: string; skipped: boolean; x1: number; x2: number; y1: number; y2: number }
export interface Label { column: number; x: number; text: string }

/** Room above the first row for the column labels. */
const TOP = 32;

/** How round a line's turns are. */
const BEND = 6;

/**
 * A line through `points`, square turns rounded off. Each turn is cut back
 * by up to `BEND` either side and the cut joined by a curve through the
 * corner, never more than half a segment so a short jog stays a jog.
 */
export function rounded(points: [number, number][]): string {
	const kept = points.filter((point, index) => index === 0 || point[0] !== points[index - 1][0] || point[1] !== points[index - 1][1]);
	if (kept.length < 2) return '';
	let d = `M${kept[0][0]} ${kept[0][1]}`;
	for (let i = 1; i < kept.length - 1; i++) {
		const [px, py] = kept[i - 1], [x, y] = kept[i], [nx, ny] = kept[i + 1];
		const before = Math.min(BEND, Math.hypot(x - px, y - py) / 2);
		const after = Math.min(BEND, Math.hypot(nx - x, ny - y) / 2);
		const ax = x - Math.sign(x - px) * before, ay = y - Math.sign(y - py) * before;
		const bx = x + Math.sign(nx - x) * after, by = y + Math.sign(ny - y) * after;
		d += ` L${ax} ${ay} Q${x} ${y},${bx} ${by}`;
	}
	const [lx, ly] = kept[kept.length - 1];
	return `${d} L${lx} ${ly}`;
}

const groupName = (job: Job) => job.stage || `Depth ${job.depth + 1}`;

/** Over 200 jobs, one card per stage (or depth) stands for its members. */
function collapsed(jobs: Job[]): Job[] {
	return [...new Set(jobs.map(groupName))].map((name) => {
		const members = jobs.filter((job) => groupName(job) === name);
		const state = members.some((job) => job.state === 'running')
			? 'running'
			: members.every((job) => job.state === 'skipped')
				? 'skipped'
				: 'host';
		return { ...members[0], id: `stage:${name}`, name, needs: [], matrix: [], implicit: false, reason: `${members.length} jobs`, state };
	});
}

function subtitle(job: Job, value: unknown, collapse: boolean): string {
	if (value) return Object.values(value as Record<string, unknown>).join(' · ');
	if (collapse) return job.reason;
	return job.runsOn.join(', ') || job.environment || job.condition || job.steps[0]?.command || job.reason;
}

/**
 * Place every card once. Columns follow `needs` depth, or the GitLab stage in
 * Stages; empty columns close up. Within a column a card sits near the cards
 * it needs, so edges cross less. GitLab's stage order (`implicit`) is shown by
 * the columns and not drawn as edges: every job to every job of the stage
 * before is a tangle, not information.
 */
export function layout(jobs: Job[], stages: string[], mode: 'needs' | 'stages', expanded: Set<string>, collapse: boolean) {
	const groups = collapse ? collapsed(jobs) : jobs;
	const wanted = (job: Job) => {
		const stage = stages.indexOf(job.stage);
		return mode === 'stages' && stage >= 0 ? stage : job.depth;
	};
	const used = [...new Set(groups.map(wanted))].sort((a, b) => a - b);
	const columnOf = new Map(used.map((value, index) => [value, index]));

	const cards: Card[] = [];
	const byId = new Map<string, Card[]>();
	const rowOf = new Map<string, number>();
	for (const [column] of used.entries()) {
		const members = groups
			.map((job, order) => ({ job, order }))
			.filter(({ job }) => columnOf.get(wanted(job)) === column)
			.map(({ job, order }) => {
				const parents = job.implicit ? [] : job.needs.map((id) => rowOf.get(id)).filter((row): row is number => row !== undefined);
				const weight = parents.length ? parents.reduce((a, b) => a + b, 0) / parents.length : Number.POSITIVE_INFINITY;
				return { job, order, weight };
			})
			.sort((a, b) => a.weight - b.weight || a.order - b.order);
		let row = 0;
		for (const { job } of members) {
			const values = expanded.has(job.id) && job.matrix.length ? job.matrix : [null];
			const placed: Card[] = [];
			values.forEach((value, index) => {
				const card: Card = {
					key: `${job.id}/${index}`,
					job,
					x: column * X,
					y: TOP + row * Y,
					subtitle: subtitle(job, value, collapse),
					count: job.matrix.length
				};
				row++;
				cards.push(card);
				placed.push(card);
			});
			rowOf.set(job.id, (placed[0].y - TOP) / Y);
			byId.set(job.id, placed);
		}
	}

	/*
	 * Lines run square, in the free space between cards (BUG-110). A curve
	 * from card to card cut straight through every card in the columns it
	 * jumped, and one job needed by twenty fanned twenty curves over the
	 * screen. Now a line leaves its card into the gutter on the card's right,
	 * runs down that gutter, crosses any columns between along the gap above
	 * its target's row — no card sits in a gap between rows — and comes down
	 * the gutter before its target. A job's lines share their way out, so
	 * twenty are one trunk with twenty branches.
	 *
	 * Each gutter's left half is for lines leaving and its right half for
	 * lines arriving; a card's lane in it follows its row, so neighbours'
	 * trunks stand side by side rather than on top of each other.
	 */
	const half = (X - W) / 2;
	const LANES = 4;
	const step = (half - 6) / LANES;
	const lane = (card: Card) => Math.round((card.y - TOP) / Y) % LANES;
	const out = (card: Card) => card.x + W + 4 + lane(card) * step;
	const into = (card: Card) => Math.max(4, card.x - half + 2 + lane(card) * step);
	const edges: Edge[] = [];
	for (const card of cards) {
		if (card.job.implicit) continue;
		for (const name of card.job.needs) {
			for (const start of byId.get(name) ?? []) {
				const x1 = start.x + W, y1 = start.y + H / 2, y2 = card.y + H / 2;
				const trunk = out(start);
				let x2 = card.x;
				let points: [number, number][];
				let low = Math.min(y1, y2), high = Math.max(y1, y2);
				if (card.x === start.x) {
					// The same column: out on the right and back in on the right.
					x2 = card.x + W;
					points = [[x1, y1], [trunk, y1], [trunk, y2], [x2, y2]];
				} else if (card.x - start.x === X) {
					// The next column: down the gutter between them.
					points = [[x1, y1], [trunk, y1], [trunk, y2], [x2, y2]];
				} else {
					// Columns between, or behind: along the gap above the target's row.
					const gap = card.y - (Y - H) / 2, drop = into(card);
					points = [[x1, y1], [trunk, y1], [trunk, gap], [drop, gap], [drop, y2], [x2, y2]];
					low = Math.min(low, gap);
					high = Math.max(high, gap);
				}
				edges.push({
					key: `${start.key}>${card.key}`,
					from: start.job.id,
					to: card.job.id,
					path: rounded(points),
					skipped: card.job.state === 'skipped',
					x1: Math.min(x1, x2), x2: Math.max(x1, x2), y1: low, y2: high
				});
			}
		}
	}

	const labels: Label[] = used.map((value, column) => {
		const names = new Set(groups.filter((job) => columnOf.get(wanted(job)) === column).map((job) => job.stage));
		const text = mode === 'stages' ? (stages[value] ?? '') : names.size === 1 ? [...names][0] : '';
		return { column, x: column * X, text };
	});
	const width = Math.max(W, ...cards.map((c) => c.x + W));
	const height = Math.max(160, ...cards.map((c) => c.y + H + 20));
	return { cards, edges, width, height, labels };
}
