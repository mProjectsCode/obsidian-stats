import { ColumnTable, escape, op, table } from 'arquero';
import { Struct } from 'arquero/dist/types/op/op-api';

Object.assign(
	ColumnTable.prototype,
	{
		distinctArray(this: ColumnTable, columnName: string) {
			return this.rollup({ values: op.array_agg_distinct(columnName) }).get('values', 0);
		},
		normalize(this: ColumnTable, column: string) {
			const sum = this.rollup({ sum: op.sum(column) }).get('sum', 0);
			return this.derive({ downloads: escape((d: Struct) => d[column] / sum) });
		},
		normalizeBy(this: ColumnTable, column: string, group: string) {
			const sum = this
				.groupby(group)
				.rollup({ sum: op.sum(column) })
				.orderby(group);
			return this
				.join(sum, group)
				.derive({ [column]: escape((d: Struct) => d[column] / d['sum']) })
				.select(...this.columnNames())
				.impute({ [column]: 0 })
				.orderby(group);
		},
		imputeAll(this: ColumnTable, columns: string[], nullable: string[]) {
			const distinctValues = columns.map(column => table({ [column]: this.distinctArray(column) }));
			const allCombinations = distinctValues.reduce((acc, curr) => acc.cross(curr));

			return allCombinations
				.join_left(
					this,
					[columns],
				)
				.impute(nullable.reduce((acc, curr) => ({ ...acc, [curr]: 0 }), {}))
				.orderby(...columns);
		},
	}
);




// addTableMethod(
// 	'distinctArray',
// 	(table: ColumnTable, columnName: string) => {
// 		return table.rollup({ values: op.array_agg_distinct(columnName) }).get('values', 0);
// 	},
// 	{ override: true },
// );

// addVerb(
// 	'normalize',
// 	(table: ColumnTable, column: string) => {
// 		const sum = table.rollup({ sum: op.sum(column) }).get('sum', 0);
// 		return table.derive({ downloads: escape((d: Struct) => d[column] / sum) });
// 	},
// 	[{ name: 'column', type: 'Expr' }],
// 	{ override: true },
// );

// // Normalize by group
// addVerb(
// 	'normalizeBy',
// 	(table: ColumnTable, column: string, group: string) => {
// 		const sum = table
// 			.groupby(group)
// 			.rollup({ sum: op.sum(column) })
// 			.orderby(group);
// 		return table
// 			.join(sum, [group])
// 			.derive({ [column]: escape((d: Struct) => d[column] / d['sum']) })
// 			.select(...table.columnNames())
// 			.impute({ [column]: 0 })
// 			.orderby(group);
// 	},
// 	[
// 		{ name: 'column', type: 'Expr' },
// 		{ name: 'group', type: 'Expr' },
// 	],
// 	{ override: true },
// );

// addVerb(
// 	'imputeAll',
// 	(tab: ColumnTable, columns: string[], nullable: string[]) => {
// 		// @ts-ignore
// 		const distinctValues = columns.map(column => table({ [column]: tab.distinctArray(column) }));
// 		const allCombinations = distinctValues.reduce((acc, curr) => acc.cross(curr));

// 		return allCombinations
// 			.join_left(
// 				tab,
// 				columns.map(_ => columns),
// 			)
// 			.impute(nullable.reduce((acc, curr) => ({ ...acc, [curr]: 0 }), {}))
// 			.orderby(...columns);
// 	},
// 	[
// 		{ name: 'columns', type: 'ExprList' },
// 		{ name: 'nullable', type: 'ExprList' },
// 	],
// 	{ override: true },
// );

// | Distribution | OS | TYPE | IS | COMMENTS |
// | ---- | ---- | ---- | ---- | ---- |
// | obsidian-x.y.z.asar.gz | N/A | N/A | N/A | File downloaded by built-in updater |
// | Obsidian-x.y.z-universal.dmg | MacOS |  | x86/ARM |  |
// | Obsidian-x.y.z.AppImage | Linux |  | x86 |  |
// | Obsidian-x.y.z-arm64.AppImage | Linux |  | ARM |  |
// | obsidian-x.y.z-arm64.tar.gz | Linux |  | ARM |  |
// | obsidian_x.y.z_amd64.deb | Linux | Debian | x86 |  |
// | obsidian_x.y.z_amd64.snap | Linux | Snap | x86 |  |
// | obsidian-x.y.z.tar.gz | Linux |  | x86 |  |
// | obsidian-x.y.z-32.exe | Windows |  | x86-32 | Legacy 32bit |
// | obsidian-x.y.z-allusers.exe | Windows |  | x86 | Installed for all users |
// | obsidian-x.y.z.exe | Windows |  | x86 | Regular windows installer |
// | obsidian_x.y.z_arm64.exe | Windows |  | ARM |  |

export function determineAssetOS(assetName: string) {
	if (assetName.endsWith('.asar.gz')) {
		return null;
	} else if (assetName.endsWith('.dmg')) {
		return 'macos';
	} else if (assetName.endsWith('.exe')) {
		return 'windows';
	} else {
		return 'linux';
	}
}

export function determineAssetType(assetName: string) {
	const versionRegex = /\d+\.\d+\.\d+/;
	return assetName.slice(assetName.search(versionRegex) + versionRegex.exec(assetName)![0].length + 1);
}

export function determineAssetIS(assetName: string) {
	if (assetName.endsWith('.dmg')) return 'both';
	return assetName.includes('arm64') ? 'arm64' : 'x86';
}

export function determineAssetArchitecture(assetName: string) {
	if (assetName.includes('32')) return '32bit';
	else return '64bit';
}

export function getMajorVersions(versions: string[]) {
	return [...new Set(versions.map(x => x.split('.').slice(0, 2).join('.')))].map(x => versions.find(y => y.startsWith(x))!);
}
