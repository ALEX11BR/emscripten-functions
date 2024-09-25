PROJECT = simple-game

.PHONY: build-web
build-web: out out/index.html
	cargo build --release --target=wasm32-unknown-emscripten
	cp target/wasm32-unknown-emscripten/release/$(PROJECT).js target/wasm32-unknown-emscripten/release/$(subst -,_,$(PROJECT)).wasm out

out:
	mkdir out

out/index.html: out shell.html
	sed 's|{{{ SCRIPT }}}|<script src="$(PROJECT).js"></script>|' shell.html > out/index.html

.PHONY: clean
clean:
	rm -r out
