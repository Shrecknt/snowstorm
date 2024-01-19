<script lang="ts">
	import Header from '../Header.svelte';
	import { onMount } from 'svelte';

	export let forgejoIdElem: HTMLInputElement;

	onMount(() => {
		function getCookie(name: string) {
			const value = `; ${document.cookie}`;
			const parts = value.split(`; ${name}=`);
			if (parts.length === 2)
				return decodeURIComponent((parts.pop() as string).split(';').shift() as string);
		}

		const forgejoId = getCookie('Forgejo-Id');

		if (forgejoId !== undefined) {
			forgejoIdElem.value = forgejoId;
		} else {
			forgejoIdElem.value = 'Missing Cookie';
		}
	});
</script>

<svelte:head>
	<title>Forgejo Login | Snowstorm Server Scanner</title>
</svelte:head>

<Header
	title="Create Account with Forgejo"
	description="Create an account which will automatically be linked to your Forgejo account"
/>

<div class="notice">
	<strong style="font-size: 32px;">Create Account</strong><br /><br />
	<form method="post" action="/auth/forgejo" enctype="application/x-www-form-urlencoded">
		<div>
			<label for="username">Username</label>
			<input type="text" id="username" name="username" placeholder="username" />
		</div>
		<div>
			<label for="password">Password</label>
			<input type="password" id="password" name="password" placeholder="password" />
		</div>
		<div>
			<label for="forgejoId">Forgejo ID</label>
			<input
				type="text"
				id="forgejoId"
				bind:this={forgejoIdElem}
				value="pls enable js :("
				disabled
			/>
		</div>
		<br />
		<input type="submit" value="Create Account" />
		<a href="/login"><input type="button" value="Login &UpperRightArrow;" /></a>
	</form>
</div>
