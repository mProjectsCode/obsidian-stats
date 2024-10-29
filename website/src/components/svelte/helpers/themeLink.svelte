<script lang="ts">
	import slug from 'slug';

	interface Props {
		id: string;
		name: string;
	}

	let { id, name }: Props = $props();
</script>

<a href={'/obsidian-stats/themes/' + slug(id)}>{name}</a>
