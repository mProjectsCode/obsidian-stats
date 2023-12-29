<script lang="ts">
	import slug from 'slug';

	export let id: string;
</script>

<a href={'/obsidian-stats/plugins/' + slug(id)}>{id}</a>
