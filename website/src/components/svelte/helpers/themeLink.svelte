<script lang="ts">
	import slug from 'slug';

	export let id: string;
	export let name: string;
</script>

<a href={'/obsidian-stats/themes/' + slug(id)}>{name}</a>
