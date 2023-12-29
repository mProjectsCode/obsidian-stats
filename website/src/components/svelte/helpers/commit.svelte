<script lang="ts">
	import type { Commit } from '../../../../../src/types.ts';
	import { dateToString } from '../../../../../src/utils.ts';

	export let commit: Commit;
	let url = 'https://github.com/obsidianmd/obsidian-releases/commit/' + commit.hash;
	let date = new Date(commit.date);
	let dateStr = dateToString(date);
</script>

<a href={url} target="_blank">{dateStr}</a>
