import { buildPluginStats } from "./plugin";
import { buildThemeStats } from "./theme";
import { buildReleaseStats } from "./release";

export async function buildStats() {
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

await buildStats();
