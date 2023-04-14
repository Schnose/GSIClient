// HTML elements
const mapName = document.querySelector(".map-name");
const tpWr = document.querySelector(".tp-wr");
const proWr = document.querySelector(".pro-wr");
const tpPb = document.querySelector("#tp-pb");
const proPb = document.querySelector("#pro-pb");

// 100.53 => 01:40.530
function formatTime(seconds) {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const remainingSeconds = (seconds % 60).toFixed(3);

	let timeString = `${minutes.toString().padStart(2, "0")}:${remainingSeconds.toString().padStart(6, "0")}`;

	if (hours > 0) {
		timeString = `${hours.toString().padStart(2, "0")}:${timeString}`;
	}

	return timeString;
}

// validate map names
function isKZMap(mapName) {
	if (!mapName) {
		return false;
	}

	const prefixes = [
		"bkz_",
		"kz_",
		"kzpro_",
		"skz_",
		"vnl_",
		"xc_",
	];

	for (const prefix of prefixes) {
		if (mapName.startsWith(prefix)) {
			return true;
		}
	}

	return false;
}

let gameInfo = null;

// Setup WebSocket connection
const url = new URL("/gsi", window.location.href);
url.protocol = url.protocol.replace("http", "ws");

const ws = new WebSocket(url.href);
ws.onmessage = async (ev) => {
	console.log({ ev });

	if (!ev) {
		return;
	}

	try {
		gameInfo = JSON.parse(ev.data);
	} catch (err) {
		console.error("Failed to deserialize JSON: ", err);
	}

	console.log({ gameInfo });

	if (!gameInfo) {
		return;
	}

	mapName.innerHTML = `${gameInfo.map_name}`;

	if (gameInfo?.mode) {
		let mode;
		switch (gameInfo.mode) {
			case "kz_timer": {
				mode = "KZT";
				break;
			}
			case "kz_simple": {
				mode = "SKZ";
				break;
			}
			case "kz_vanilla": {
				mode = "VNL";
				break;
			}
			default: {
				mode = "unknown mode";
			}
		}
		mapName.innerHTML = `[${mode}] ${mapName.innerHTML}`;
	}

	if (gameInfo?.map_tier) {
		mapName.innerHTML += ` (T${gameInfo.map_tier})`;
	} else {
		mapName.innerHTML += " (not global)";
	}
};

setInterval(async () => {
	if (!gameInfo) {
		return;
	}

	const shouldFetchRecords = gameInfo.player_name
		&& gameInfo.mode
		&& isKZMap(gameInfo.map_name);

	const [tp_wr, pro_wr] = shouldFetchRecords
		? await fetch(
			`http://localhost:9999/wrs?steam_id=${gameInfo.steam_id}&map_identifier=${gameInfo.map_name}&mode=${gameInfo.mode}`
		)
			.then((res) => res.json())
			.catch(console.error)
		: [null, null];

	const [tp_pb, pro_pb] = shouldFetchRecords
		? await fetch(
			`http://localhost:9999/pbs?steam_id=${gameInfo.steam_id}&map_identifier=${gameInfo.map_name}&mode=${gameInfo.mode}`
		)
			.then((res) => res.json())
			.catch(console.error)
		: [null, null];

	if (tp_wr) {
		tpWr.innerHTML = `${formatTime(tp_wr.time)} by ${tp_wr.player_name}`;

		if (tp_pb && tp_pb.time - tp_wr.time != 0) {
			tpPb.innerHTML = `(+${formatTime(tp_pb.time - tp_wr.time)})`;
		} else {
			tpPb.innerHTML = "";
		}

	} else {
		tpWr.innerHTML = "no WR";
		tpPb.innerHTML = "";
	}

	if (pro_wr) {
		proWr.innerHTML = `${formatTime(pro_wr.time)} by ${pro_wr.player_name}`;

		if (pro_pb && pro_pb.time - pro_wr.time != 0) {
			proPb.innerHTML = `(+${formatTime(pro_pb.time - pro_wr.time)})`;
		} else {
			proPb.innerHTML = "";
		}

	} else {
		proWr.innerHTML = "no WR";
		proPb.innerHTML = "";
	}
}, 3000);
