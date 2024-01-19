<script>
	export let title = 'title';
	export let description = 'description';
</script>

<svelte:head>
	<style>
		.forgejo-icon {
			display: inline-block;
			width: 16px;
			height: 16px;
			background-image: url(/forgejo.svg);
			position: relative;
			top: 2px;
			right: 2px;
		}
	</style>
</svelte:head>

<header>
	<nav>
		<ul>
			<li><a href="/">Home</a></li>
			<li><a href="/search">Search</a></li>
			<li><a href="https://discord.gg/WgaqXEywvA">Discord</a></li>
			<li>
				<a href="https://git.shrecked.dev/Shrecknt/snowstorm">
					<div class="icon forgejo-icon"></div>
					Git
				</a>
			</li>
			<li><a class="current" href="/login">Login</a></li>
		</ul>
	</nav>
	<h1>{title}</h1>
	<p>{description}</p>
</header>
