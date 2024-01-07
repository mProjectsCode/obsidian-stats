<script lang="ts">
	import type { Commit } from '../../../../../src/types.ts';

	export let commit: Commit;
	let url = 'https://github.com/obsidianmd/obsidian-releases/commit/' + commit.hash;
</script>

<a href={url} target="_blank">{commit.date}</a>
