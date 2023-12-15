// taken from https://gitlab.com/Fryuni/blog/-/blob/afd3ddec74c45151ceed47e67e5e010a00d58663/src/utils/zod.ts

import { z } from 'astro/zod';

export { z };

export function parse<T extends z.ZodType>(schema: T, data: unknown): z.infer<T> {
	const result = schema.safeParse(data);

	if (result.success) {
		return result.data;
	}

	let message = 'Invalid data:';

	const { fieldErrors } = result.error.flatten();

	for (const [path, messages] of Object.entries(fieldErrors)) {
		if (messages === undefined) continue;

		for (const errorMessage of messages) {
			message += `\n  - ${path}: ${errorMessage}`;
		}
	}

	throw new Error(message);
}
