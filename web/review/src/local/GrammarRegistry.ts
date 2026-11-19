import { cachedWasm, grammarManifest, grammarQuery, grammarUrl } from "./assets";
import { engineInstance, isTrap, type Engine } from "./engine";
import { compileGrammar, linkGrammar } from "./linkGrammar";
import { TransientError } from "./TransientError";

// A download can succeed later and a trap is handled by the worker; any other failure repeats, so it is remembered rather than retried.
const permanent = (error: unknown) => !(error instanceof TransientError) && !isTrap(error);

export class GrammarRegistry {
  private manifest = grammarManifest();
  private grammars = new Map<string, Promise<number>>();
  private languages = new Map<string, Promise<void>>();
  private failedGrammars = new Set<string>();
  private failedLanguages = new Set<string>();

  constructor(private engine: Engine) {}

  // False when the language's grammar could not load, so its files render without syntax; a trap is rethrown for the worker to recover from.
  async ensure(language: string): Promise<boolean> {
    if (language === "plaintext") return true;
    try {
      await this.language(language);
      return true;
    } catch (error) {
      if (isTrap(error)) throw error;
      return false;
    }
  }

  // The engine instance was replaced, so every linked grammar is gone; failures stay remembered.
  reset() {
    this.grammars.clear();
    this.languages.clear();
  }

  disable(language: string): boolean {
    const newlyDisabled = !this.failedLanguages.has(language);
    this.failedLanguages.add(language);
    return newlyDisabled;
  }

  enable(language: string) {
    this.failedLanguages.delete(language);
  }

  private language(language: string): Promise<void> {
    return this.once(this.languages, this.failedLanguages, language, async () => {
      const entry = this.manifest.languages.get(language);
      if (!entry) throw new Error(`No grammar for ${language}`);
      const instance = engineInstance();
      const [index, query] = await Promise.all([
        this.grammar(entry.grammar),
        grammarQuery(language),
        ...entry.requires.map((required) => this.language(required)),
      ]);
      this.ensureInstance(instance);
      this.engine.register_grammar(language, index, query);
    });
  }

  private grammar(grammar: string): Promise<number> {
    return this.once(this.grammars, this.failedGrammars, grammar, async () => {
      const url = grammarUrl(grammar);
      if (!url) throw new Error(`No grammar file for ${grammar}`);
      const instance = engineInstance();
      const compiled = await compileGrammar(await cachedWasm(url));
      this.ensureInstance(instance);
      return linkGrammar(this.engine, compiled, this.manifest.imports);
    });
  }

  private ensureInstance(instance: number) {
    if (instance !== engineInstance())
      throw new TransientError("The engine restarted while a grammar loaded");
  }

  // Concurrent requests share one load.
  private once<T>(
    loads: Map<string, Promise<T>>,
    failed: Set<string>,
    key: string,
    load: () => Promise<T>,
  ): Promise<T> {
    if (failed.has(key)) return Promise.reject(new Error(`${key} failed to load before`));
    let pending = loads.get(key);
    if (!pending) {
      const started = load();
      pending = started;
      loads.set(key, started);
      void started.catch((error: unknown) => {
        if (loads.get(key) === started) loads.delete(key);
        if (permanent(error)) failed.add(key);
      });
    }
    return pending;
  }
}
