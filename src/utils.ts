export class UserError extends Error {}

export function dateToString(date: Date): string {
	return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

export function stringToBase64(str: string): string {
	return Buffer.from(str).toString('base64');
}

export function base64ToString(str: string): string {
	return Buffer.from(str, 'base64').toString();
}