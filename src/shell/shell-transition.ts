import { normalizeShellState, SHELL_STATE_DIMENSIONS, type NotchShellState } from "../app/state/shell-state.ts";

interface TransitionOptions {
  resize: (state: NotchShellState) => Promise<void>;
  show: (state: NotchShellState) => void;
  duration: () => number;
}

/** Serializes native resizes; newer requests cancel pending visual shrink delays. */
export class ShellTransition {
  private visual: NotchShellState = "compact";
  private native: NotchShellState = "compact";
  private target: NotchShellState = "compact";
  private revision = 0;
  private pending: Promise<void> | null = null;
  private cancelDelay: (() => void) | null = null;
  private disposed = false;
  private options: TransitionOptions;

  constructor(options: TransitionOptions) { this.options = options; }

  request(state: NotchShellState): Promise<void> {
    this.target = normalizeShellState(state);
    this.revision++;
    this.cancelDelay?.();
    if (!this.pending) {
      this.pending = Promise.resolve().then(() => this.run()).finally(() => { this.pending = null; });
    }
    return this.pending;
  }

  private async run(): Promise<void> {
    while (!this.disposed && (this.visual !== this.target || this.native !== this.target)) {
      const target = this.target;
      const revision = this.revision;
      const next = SHELL_STATE_DIMENSIONS[target];
      const previous = SHELL_STATE_DIMENSIONS[this.native];
      if (next.width > previous.width || next.height > previous.height) {
        await this.options.resize(target);
        this.native = target;
        if (this.disposed || revision !== this.revision) continue;
      }
      if (this.visual !== target) {
        this.visual = target;
        this.options.show(target);
      }
      if (this.native !== target) {
        if (next.width < previous.width || next.height < previous.height) {
          await this.delay(this.options.duration());
        }
        if (this.disposed || revision !== this.revision) continue;
        await this.options.resize(target);
        this.native = target;
      }
    }
  }

  private delay(ms: number): Promise<void> {
    if (ms <= 0) return Promise.resolve();
    return new Promise((resolve) => {
      const finish = () => { clearTimeout(timer); this.cancelDelay = null; resolve(); };
      const timer = setTimeout(finish, ms);
      this.cancelDelay = finish;
    });
  }

  destroy(): void {
    this.disposed = true;
    this.cancelDelay?.();
  }
}
