<script lang="ts">
	import { onMount } from 'svelte';
	import Header from '../Header.svelte';
	import type { ActionResponse, WebActions } from '../ApiTypes.svelte';

	let ws: WebSocket;
	export function getQueue() {
		ws.send(
			JSON.stringify({
				type: 'GetModesQueue',
				data: {}
			} as WebActions)
		);
	}

	export let usernameBox: HTMLInputElement;
	export let autocompleteResultsBox: HTMLTextAreaElement;
	export function autocomplete() {
		let username = usernameBox.value ?? '';
		ws.send(
			JSON.stringify({
				type: 'Autocomplete',
				data: {
					autocomplete_data: {
						type: 'Username',
						data: {
							username: username
						}
					}
				}
			} as WebActions)
		);
	}

	onMount(async () => {
		const wsUrl = 'wss://' + window.location.host + '/ws';
		ws = new WebSocket(wsUrl);

		ws.addEventListener('open', () => {
			autocomplete();
			ws.send(
				JSON.stringify({
					type: 'QueueAction',
					data: {
						action: {
							type: 'Enqueue',
							data: {
								mode: {
									type: 'Auto',
									data: {}
								},
								duration: {
									secs: 100000000,
									nanos: 0
								}
							}
						}
					}
				} as WebActions)
			);
			ws.send(
				JSON.stringify({
					type: 'GetModesQueue',
					data: {}
				} as WebActions)
			);
		});
		ws.addEventListener('message', (message) => {
			const obj = JSON.parse(message.data) as ActionResponse;
			console.log(obj);
			if (!obj.success) {
				console.log('An error occurred!', obj.msg);
				return;
			}
			if (obj.data?.type == 'autocomplete') {
				let rawPlayers = obj.data.data.data.players;
				let players = rawPlayers.map((player) => `${player[0]} - ${player[1]}`).join('\n');
				autocompleteResultsBox.value = players;
			}
		});
	});
</script>

<svelte:head>
	<title>Admin Panel | Snowstorm Server Scanner</title>
</svelte:head>

<Header title="Admin Panel" description="Enqueue and manage tasks" />

<button on:click={getQueue}>get queue</button><br />
<input type="text" bind:this={usernameBox} on:input={autocomplete} /><br />
<button on:click={autocomplete}>autocomplete</button><br />
<textarea bind:this={autocompleteResultsBox} style="resize:none;width:100%;" rows="16" readonly
></textarea><br />
