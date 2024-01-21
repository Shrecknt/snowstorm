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

	onMount(async () => {
		const wsUrl = 'wss://' + window.location.host + '/ws';
		ws = new WebSocket(wsUrl);

		ws.addEventListener('open', () => {
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
		});
	});
</script>

<svelte:head>
	<title>Admin Panel | Snowstorm Server Scanner</title>
</svelte:head>

<Header title="Admin Panel" description="Enqueue and manage tasks" />

<button on:click={getQueue}>get queue</button><br />
