<script lang="ts">
	import Header from '../Header.svelte';
	import { onMount } from 'svelte';

	export let discordIdElem: HTMLInputElement;

	onMount(() => {
		function getCookie(name: string) {
			const value = `; ${document.cookie}`;
			const parts = value.split(`; ${name}=`);
			if (parts.length === 2)
				return decodeURIComponent((parts.pop() as string).split(';').shift() as string);
		}

		const discordId = getCookie('Discord-Id');

		if (discordId !== undefined) {
			discordIdElem.value = discordId;
		} else {
			discordIdElem.value = 'Missing Cookie';
		}
	});
</script>

<svelte:head>
	<title>Discord Login | Snowstorm Server Scanner</title>
</svelte:head>

<Header
	title="Create Account with Discord"
	description="Create an account which will automatically be linked to your Discord account"
/>

<div class="notice">
	<strong style="font-size: 32px;">Create Account</strong><br /><br />
	<form method="post" action="/auth/discord" enctype="application/x-www-form-urlencoded">
		<div>
			<label for="username">Username</label>
			<input type="text" id="username" name="username" placeholder="username" />
		</div>
		<div>
			<label for="password">Password</label>
			<input type="password" id="password" name="password" placeholder="password" />
		</div>
		<div>
			<label for="discordId">Discord ID</label>
			<input
				type="text"
				id="discordId"
				bind:this={discordIdElem}
				value="pls enable js :("
				disabled
			/>
		</div>
		<br />
		<input type="submit" value="Create Account" />
		<a href="/login"><input type="button" value="Login &UpperRightArrow;" /></a>
	</form>
</div>
