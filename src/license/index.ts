import fs from 'node:fs/promises';
import {parse} from "yaml";


export interface LicenseData {
    title: string;
    "spdx-id": string;
    description: string;
    how: string;
    using: string[];
    permissions: string[];
    conditions: string[];
    limitations: string[];
}

export interface LicenseDescription {
    description: string;
    label: string;
    tag: string;
}

export interface Licenses {
    licenses: LicenseData[];
    permissions: string[];
    conditions: string[];
    limitations: string[];
    descriptions: {
        permissions: LicenseDescription[];
        conditions: LicenseDescription[];
        limitations: LicenseDescription[];
    }
}

export async function processLicences() {
    const dir = await fs.readdir('choosealicense.com/_licenses');
    const licenses = await Promise.all(dir.map(async (file) => {
        const data = await fs.readFile(`choosealicense.com/_licenses/${file}`, 'utf-8');
        const frontMatter = data.split('---')[1];
        const licenseData: LicenseData = parse(frontMatter);
        return licenseData;
    }));

    const allPermissions = new Set<string>();
    const allConditions = new Set<string>();
    const allLimitations = new Set<string>();

    for (const license of licenses) {
        license.permissions.forEach(x => allPermissions.add(x));
        license.conditions.forEach(x => allConditions.add(x));
        license.limitations.forEach(x => allLimitations.add(x));
    }

    await Bun.write(Bun.file('licenses.json'), JSON.stringify({
        licenses: licenses,
        permissions: Array.from(allPermissions),
        conditions: Array.from(allConditions),
        limitations: Array.from(allLimitations),
        descriptions: parse(await fs.readFile('choosealicense.com/_data/rules.yml', 'utf-8'))
    } satisfies Licenses, null, 4));
}