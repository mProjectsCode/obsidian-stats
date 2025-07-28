<script lang="ts">
	interface Props {
		id: string;
		name: string;
	}

	let { id, name }: Props = $props();
</script>

<a href={'/obsidian-stats/themes/' + id}>{name}</a>
