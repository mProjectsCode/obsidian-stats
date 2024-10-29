<script lang="ts">
	import slug from 'slug';

	interface Props {
		id: string;
	}

	let { id }: Props = $props();
</script>

<a href={'/obsidian-stats/plugins/' + slug(id)}>{id}</a>
