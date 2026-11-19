// A failure worth retrying later, such as a download that failed or returned something other than WebAssembly.
export class TransientError extends Error {}
