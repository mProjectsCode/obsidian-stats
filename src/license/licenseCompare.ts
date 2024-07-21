import { promises as fs } from 'fs';
import { parse } from 'yaml';
import { LicenseData } from '.';
import dice from 'fast-dice-coefficient';

const COPYRIGHT_REGEXP = /^(?:copyright)\s*?(?:&copy;|\(c\)|&#(?:169|xa9;)|©)\s*[0-9]{4}.*$/;

export class LicenseComparer {
    licenses: {
        name: string;
        text: string;
    }[] = [];

    async init() {
        const dir = await fs.readdir('choosealicense.com/_licenses');

        const regexp = new RegExp(COPYRIGHT_REGEXP, 'gmi');

        this.licenses = await Promise.all(dir.map(async (file) => {
            const data = await fs.readFile(`choosealicense.com/_licenses/${file}`, 'utf-8');
            const frontmatter = parse(data.split('---')[1]) as LicenseData;
            const text = data
                .split('---')[2]
                .toLowerCase()
                .replaceAll(regexp, '') // remove copyright notices
                .replaceAll(/\s+/g, '');
            return { name: frontmatter["spdx-id"], text: text };
        }));
    }

    /**
     * Returns the spdx-id of the best matching license or undefined if no match is found.
     */
    compare(license: string): string | undefined {
        const lowerCaseLicense = license.toLowerCase();

        // fast paths for common licenses
        if (lowerCaseLicense.startsWith('mit license')) {
            return 'MIT';
        }
        if (lowerCaseLicense.startsWith('GNUGENERALPUBLICLICENSEVersion3,')) {
            return 'GPL-3.0';
        }
        if (lowerCaseLicense.startsWith('GNULESSERGENERALPUBLICLICENSEVersion3,')) {
            return 'LGPL-3.0';
        }
        if (lowerCaseLicense.startsWith('GNUAFFEROGENERALPUBLICLICENSEVersion3,')) {
            return 'AGPL-3.0';
        }

        // we test if the license contains only a copyright notice like "Copyright (c) 2024 Moritz Jung"
        if (new RegExp(COPYRIGHT_REGEXP, 'gi').test(lowerCaseLicense)) {
            console.log('explicitly unlicensed', license);
            // if so we assume that the author reserves all rights
            return 'explicitly unlicensed';
        }

        const processedLicense = license
            .toLowerCase()
            .replaceAll(new RegExp(COPYRIGHT_REGEXP, 'gmi'), '') // remove copyright notices
            .replaceAll(/\s+/g, '');

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