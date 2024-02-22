<script>
	import { config } from '$lib/config';
	import Header from '../Header.svelte';
</script>

<svelte:head>
	<title>Create Account | Snowstorm Server Scanner</title>
</svelte:head>

<Header
	title="Create Account"
	description="Create an account, or you can login with Discord or Minecraft"
/>

<div class="notice">
	<strong style="font-size: 32px;">Create Account</strong><br /><br />
	<form method="post" action="/auth/signup" enctype="application/x-www-form-urlencoded">
		<div>
			<label for="username">Username</label>
			<input type="text" id="username" name="username" placeholder="username" />
		</div>
		<div>
			<label for="password">Password</label>
			<input type="password" id="password" name="password" placeholder="password" />
		</div>
		<br />
		<input type="submit" value="Create Account" />
		<a href="/login"><input type="button" value="Login &UpperRightArrow;" /></a>
	</form>
	{#if config.web.oauth.discord.enabled}
		<a
			href="https://discord.com/oauth2/authorize?client_id={config.web.oauth.discord
				.client_id}&response_type=code&redirect_uri=https%3A%2F%2F{config.web
				.domain}%2Foauth2_discord&scope=identify+guilds.members.read"
			><input type="button" value="Login with Discord" /></a
		>
	{/if}
	{#if config.web.oauth.forgejo.enabled}
		<a
			href="https://git.shrecked.dev/login/oauth/authorize?client_id={config.web.oauth.forgejo
				.client_id}&redirect_uri=https%3A%2F%2F{config.web
				.domain}%2Foauth2_forgejo&response_type=code"
			><input type="button" value="Login with Forgejo" /></a
		>
	{/if}
	<a href="/minecraft_login"><input type="button" value="Login with Minecraft" /></a>
</div>
