<script lang="ts" context="module">
	export type Duration = { secs: number; nanos: number };
	export type Ipv4Addr = string;
	export type Ipv4AddrRange = { first: Ipv4Addr; last: Ipv4Addr };

	export type Autocomplete = {
		autocomplete_data:
			| {
					type: 'Username';
					data: { username: string };
			  }
			| {
					type: 'Uuid';
					data: { uuid: string };
			  };
	};
	export type DataEntry =
		| { type: 'queue'; data: [ScanningMode, Duration][] }
		| { type: 'autocomplete'; data: AutocompleteResults };
	export type ScanningMode =
		| { type: 'Paused'; data: {} }
		| { type: 'Discovery'; data: {} }
		| { type: 'DiscoveryTopPorts'; data: {} }
		| { type: 'Range'; data: { range: Ipv4AddrRange } }
		| { type: 'RangeTopPorts'; data: { range: Ipv4AddrRange } }
		| { type: 'AllPorts'; data: { ip: Ipv4Addr } }
		| { type: 'Rescan'; data: { ips: [Ipv4Addr, number][] } }
		| { type: 'Auto'; data: {} };
	export type AutocompleteResults = {
		type: 'Username' | 'Uuid';
		data: {
			players: [string, string][];
		};
	};

	export type WebActions =
		| { type: 'QueueAction'; data: {} }
		| { type: 'GetModesQueue'; data: {} }
		| { type: 'Autocomplete'; data: Autocomplete };
	export type ActionResponse = {
		success: boolean;
		msg: string;
		data?: DataEntry;
	};
</script>
