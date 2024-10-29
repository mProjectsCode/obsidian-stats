<script lang="ts">
	import type { Commit } from '../../../../../src/types.ts';

	interface Props {
		commit: Commit;
	}

	let { commit }: Props = $props();
	let url = 'https://github.com/obsidianmd/obsidian-releases/commit/' + commit.hash;
</script>

<a href={url} target="_blank">{commit.date}</a>
