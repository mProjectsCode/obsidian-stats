// Run src/plugin/index.ts, src/theme/index.ts, src/release/index.ts with try/catch

import { buildPluginStats } from "./plugin";
import { buildThemeStats } from "./theme";
import { buildReleaseStats } from "./release";

async function main() {
    try {
        await buildPluginStats();
    } catch (e) {
        console.error(e);
    }

    try {
        await buildThemeStats();
    } catch (e) {
        console.error(e);
    }

    try {
        await buildReleaseStats();
    } catch (e) {
        console.error(e);
    }
}

main();
