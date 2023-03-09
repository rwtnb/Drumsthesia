build-debug:
	wasm-pack build --debug --target web --out-dir web

build-release:
	wasm-pack build --release --target web --out-dir web

serve:
	cd web && python3 -m http.server
