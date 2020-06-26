clean:
	cargo clean
	rm musl_*

docker-release:
	./build_linux_musl.sh --release
	docker build -t codenamer .

docker-debug:
	./build-linux-musl.sh
	docker build -t codenamer .
