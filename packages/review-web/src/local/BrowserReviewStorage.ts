const emptyReview = '{"reviewed":{},"notes":[]}';

export class BrowserReviewStorage {
  private database: Promise<IDBDatabase>;

  constructor(databaseName: string) {
    this.database = new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open(databaseName, 1);
      request.onupgradeneeded = () => request.result.createObjectStore("data");
      request.onsuccess = () => {
        request.result.onversionchange = () => request.result.close();
        resolve(request.result);
      };
      request.onerror = () => reject(request.error);
      request.onblocked = () => reject(new Error("Close other review tabs to upgrade browser storage"));
    });
    void this.database.catch(() => {});
  }

  async readStored(key: string): Promise<string | undefined> {
    const db = await this.database;
    return new Promise((resolve, reject) => {
      const transaction = db.transaction("data", "readonly");
      const request = transaction.objectStore("data").get(key);
      transaction.oncomplete = () => resolve(request.result);
      transaction.onabort = () => reject(transaction.error);
    });
  }

  async readReview(repository: string): Promise<string> {
    return await this.readStored(`review:${repository}`) ?? emptyReview;
  }

  async writeStored(key: string, value: string): Promise<void> {
    const db = await this.database;
    return new Promise((resolve, reject) => {
      const transaction = db.transaction("data", "readwrite");
      transaction.objectStore("data").put(value, key);
      transaction.oncomplete = () => resolve();
      transaction.onabort = () => reject(transaction.error);
    });
  }

  // Rust runs synchronously inside the callback, so overlapping tabs mutate the latest committed snapshot.
  async updateReview(repository: string, mutate: (state: string) => string, backup = false): Promise<string> {
    const db = await this.database;
    return new Promise((resolve, reject) => {
      const transaction = db.transaction("data", "readwrite");
      const store = transaction.objectStore("data");
      const key = `review:${repository}`;
      const request = store.get(key);
      let result: string;
      let failure: unknown;
      request.onsuccess = () => {
        try {
          const previous = request.result ?? emptyReview;
          result = mutate(previous);
          if (backup) store.put(JSON.stringify({ previous, imported: result }), `review-backup:${repository}`);
          store.put(result, key);
        } catch (error) { failure = error; transaction.abort(); }
      };
      transaction.oncomplete = () => resolve(result);
      transaction.onabort = () => reject(failure ?? transaction.error);
    });
  }
}
