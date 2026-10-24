import {existsSync} from 'node:fs';
import {homedir} from 'node:os';
import {join} from 'node:path';
// Fixed destinations for the maintainer-side publisher and feedback relay.
export const FORK = 'bluecmd/fieldcraft';
export const UPSTREAM = 'tangency-labs/fieldcraft';
export const QUEUE = 'pr-queue';

// Use the checksum-verified maintainer upgrade without changing system packages.
export const GH = existsSync(join(homedir(), '.local/bin/gh')) ? join(homedir(), '.local/bin/gh') : '/usr/bin/gh';
