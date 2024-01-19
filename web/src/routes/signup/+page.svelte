<script>
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
	<a
		href="https://discord.com/oauth2/authorize?client_id=1077019155410202757&response_type=code&redirect_uri=https%3A%2F%2Fsnowstorm.shrecked.dev%2Foauth2_discord&scope=identify+guilds.members.read"
		><input type="button" value="Login with Discord" /></a
	>
	<a
		href="https://git.shrecked.dev/login/oauth/authorize?client_id=2938b5c8-f959-479b-8afe-f6d528470f99&redirect_uri=https%3A%2F%2Fsnowstorm.shrecked.dev%2Foauth2_forgejo&response_type=code"
		><input type="button" value="Login with Forgejo" /></a
	>
	<a href="/minecraft_login"><input type="button" value="Login with Minecraft" /></a>
</div>
