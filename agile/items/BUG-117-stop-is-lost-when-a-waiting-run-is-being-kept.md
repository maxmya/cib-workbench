<!-- SPDX-License-Identifier: MIT -->

# BUG-117 — Stop is lost when a waiting run is being kept

**Status:** Fixed — on `fix/BUG-117-stop-lost-while-a-run-saves`.
**Branch:** `fix/BUG-117-stop-lost-while-a-run-saves`.
**Screens:** Recipes.
**Raised by:** CI on `main`, 2026-10-29: `recipes::tests::stop_ends_a_run_that_is_waiting` failed with "the run never ended" (run 40825418564), on code unchanged since 1.0.0.

## Cause

- *Stop* ends a waiting run only if it can take the run at once (`recipe_run_stop`, `try_lock`). Otherwise it raises the stop flag and wakes the run's thread.
- Just after a run stops to ask, its thread keeps the run while it writes the record (`publish`). A *Stop* pressed then could not take it.
- Woken, the thread calls `tick`, which returns at once for a run that is waiting, before it looks at the flag. The thread went back to waiting and the run never ended.

## Fix

- Woken with the flag up and the run waiting where *End* is a choice, the thread ends it, as *Stop* would have.
- `a_stop_that_could_not_take_the_run_still_ends_it` leaves the run as such a *Stop* does, and fails without the fix.
