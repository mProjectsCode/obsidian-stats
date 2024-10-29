<script lang="ts">
	interface Props {
		repo: string;
	}

	let { repo }: Props = $props();
</script>

<a href={'https://github.com/' + repo} target="_blank">{repo}</a>
