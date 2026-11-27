import { readFile } from 'node:fs/promises';
import type { Config } from './config';

export class Indexer {
  run() {}
}

export function scanRepository() {
  return new Indexer();
}

const helper = () => readFile;
