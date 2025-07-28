<script lang="ts">
	interface Props {
		id: string;
	}

	let { id }: Props = $props();
</script>

<a href={'/obsidian-stats/plugins/' + id}>{id}</a>
