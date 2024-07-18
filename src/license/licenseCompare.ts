import { promises as fs } from 'fs';
import { parse } from 'yaml';
import { LicenseData } from '.';
import dice from 'fast-dice-coefficient';

export class LicenseComparer {
    licenses: {
        name: string;
        text: string;
    }[] = [];

    async init() {
        const dir = await fs.readdir('choosealicense.com/_licenses');
        this.licenses = await Promise.all(dir.map(async (file) => {
            const data = await fs.readFile(`choosealicense.com/_licenses/${file}`, 'utf-8');
            const frontmatter = parse(data.split('---')[1]) as LicenseData;
            const text = data.split('---')[2];
            return { name: frontmatter["spdx-id"], text: text.toLowerCase().replaceAll(/\s+/g, '') };
        }));
    }

    /**
     * Returns the spdx-id of the best matching license or undefined if no match is found.
     */
    compare(license: string): string | undefined {
        const processedLicense = license.toLowerCase().replaceAll(/\s+/g, '');

        const exactMatch = this.licenses.find(x => x.text === processedLicense);
        if (exactMatch) {
            return exactMatch.name;
        }

        const scores = this.licenses.map(x => {
            return {
                name: x.name,
                score: dice(x.text, processedLicense)
            }
        });

        scores.sort((a, b) => b.score - a.score);
        const bestScore = scores[0];

        if (bestScore.score > 0.95) {
            return bestScore.name;
        }

        return undefined;
    }
}