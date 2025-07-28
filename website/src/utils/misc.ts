export enum ItemType {
	Plugin = 'plugin',
	Theme = 'theme',
}

export function typeToString(type: ItemType, plural: boolean = false, capitalize: boolean = false): string {
	let str;
	if (type === ItemType.Plugin) {
		str = 'Plugin';
	} else if (type === ItemType.Theme) {
		str = 'Theme';
	} else {
		throw new Error(`Unknown type: ${type}`);
	}

	str = capitalize ? str.charAt(0).toUpperCase() + str.slice(1) : str;
	str = plural ? str + 's' : str;
	return str;
}
