<script lang="ts">
	import { onMount } from 'svelte';
	import Header from '../Header.svelte';

	let ws: WebSocket;
	export function getQueue() {
		ws.send(JSON.stringify({ GetModesQueue: {} }));
	}

	onMount(async () => {
		type ActionResponse = {
			success: boolean;
			msg: string;
			queue?: QueueEntry[];
		};
		type QueueEntry = [ModeType, Duration];
		type ModeType = { Auto: {} } | { Discovery: {} };
		type Duration = { secs: number; nanos: number };

		const wsUrl = 'wss://' + window.location.host + '/ws';
		ws = new WebSocket(wsUrl);

		ws.addEventListener('open', () => {
			ws.send(JSON.stringify({ GetModesQueue: {} }));
		});
		ws.addEventListener('message', (message) => {
			const obj = JSON.parse(message.data) as ActionResponse;
			if (obj.queue) {
				let queueItems: { type: string; duration: Duration }[] = [];
				for (let item of obj.queue) {
					const type = Object.keys(item[0])[0];
					const duration = item[1];
					queueItems.push({ type, duration });
				}
				console.log(queueItems);
			}
		});
	});
</script>

<svelte:head>
	<title>Admin Panel | Snowstorm Server Scanner</title>
</svelte:head>

<Header title="Admin Panel" description="Enqueue and manage tasks" />

<button on:click={getQueue}>get queue</button><br />
