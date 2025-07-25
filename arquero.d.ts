import 'arquero';

declare module 'arquero' {
	export interface ColumnTable {
		distinctArray(columnName: string): string[];
		normalize(column: string): ColumnTable;
		normalizeBy(column: string, group: string): ColumnTable;
		imputeAll(columns: string[], nullable: string[]): ColumnTable;
	}
}
